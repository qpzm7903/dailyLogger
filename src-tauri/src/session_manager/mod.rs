//! SESSION-001: 工作时段管理模块
//!
//! 负责检测、创建和管理用户的工作时段（Session）。
//! 时段是指连续工作的一个时间段，两次截图间隔超过阈值时自动创建新时段。

use chrono::Local;
use rusqlite::{params, OptionalExtension};
use serde::{Deserialize, Serialize};

use crate::errors::{AppError, AppResult};
use crate::memory_storage::{get_settings_sync, DB_CONNECTION};

/// SESSION-002: Per-screenshot analysis result from AI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenshotAnalysis {
    pub timestamp: String,
    pub current_focus: String,
    pub active_software: String,
    pub context_keywords: Vec<String>,
    pub tags: Vec<String>,
}

/// SESSION-002: Session batch analysis response from AI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionAnalysisResponse {
    pub per_screenshot_analysis: Vec<ScreenshotAnalysis>,
    pub session_summary: String,
    pub context_for_next: String,
}

/// Re-export SessionScreenshot from memory_storage for convenience
pub use crate::memory_storage::SessionScreenshot;

/// 工作时段状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum SessionStatus {
    #[default]
    Active, // 正在进行中
    Ended,    // 已结束
    Analyzed, // 已分析
}

impl From<String> for SessionStatus {
    fn from(s: String) -> Self {
        match s.to_lowercase().as_str() {
            "active" => SessionStatus::Active,
            "ended" => SessionStatus::Ended,
            "analyzed" => SessionStatus::Analyzed,
            _ => SessionStatus::Active,
        }
    }
}

impl From<SessionStatus> for String {
    fn from(status: SessionStatus) -> Self {
        match status {
            SessionStatus::Active => "active".to_string(),
            SessionStatus::Ended => "ended".to_string(),
            SessionStatus::Analyzed => "analyzed".to_string(),
        }
    }
}

/// 工作时段结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: i64,
    pub date: String,                     // YYYY-MM-DD
    pub start_time: String,               // RFC3339
    pub end_time: Option<String>,         // RFC3339, None = ongoing
    pub ai_summary: Option<String>,       // AI 生成的时段摘要
    pub user_summary: Option<String>,     // 用户自写的时段摘要
    pub context_for_next: Option<String>, // 传递给下一时段分析的上下文
    pub status: SessionStatus,
    #[serde(default)]
    pub screenshot_count: Option<i64>, // 时段内的截图数量
}

impl Default for Session {
    fn default() -> Self {
        let now = Local::now();
        Self {
            id: 0,
            date: now.format("%Y-%m-%d").to_string(),
            start_time: now.to_rfc3339(),
            end_time: None,
            ai_summary: None,
            user_summary: None,
            context_for_next: None,
            status: SessionStatus::Active,
            screenshot_count: None,
        }
    }
}

/// 获取时段间隔阈值（分钟）
fn get_session_gap_minutes() -> i64 {
    get_settings_sync()
        .ok()
        .and_then(|s| s.session_gap_minutes)
        .unwrap_or(30) as i64
}

/// 获取今天的日期字符串 (YYYY-MM-DD)
fn get_today_date() -> String {
    Local::now().format("%Y-%m-%d").to_string()
}

/// 检测或创建当前工作时段
///
/// 返回当前活跃的 session_id。如果两次截图间隔超过阈值，
/// 自动结束当前时段并创建新时段。
///
/// # Arguments
/// * `current_timestamp` - 当前时间戳 (RFC3339)
///
/// # Returns
/// * `Ok(session_id)` - 当前时段的 ID
/// * `Err(message)` - 数据库操作失败
pub fn detect_or_create_session(current_timestamp: &str) -> AppResult<i64> {
    let db = DB_CONNECTION.lock()?;
    let conn = db
        .as_ref()
        .ok_or_else(|| AppError::database("Database not initialized"))?;

    let today = extract_date_from_timestamp(current_timestamp);

    // 检查是否有活跃时段
    if let Some(active_session) = get_active_session_with_conn(conn, &today)? {
        // 检查最后一条记录的时间
        let last_record_time = get_last_record_timestamp_with_conn(conn, active_session.id)?;

        if let Some(last_time) = last_record_time {
            let gap_minutes = calc_gap_minutes(&last_time, current_timestamp)?;
            let gap_threshold = get_session_gap_minutes();

            if gap_minutes > gap_threshold {
                // 间隔超过阈值 → 结束当前时段，创建新时段
                end_session_with_conn(conn, active_session.id)?;
                return create_new_session_with_conn(conn, &today, current_timestamp);
            }
        }

        return Ok(active_session.id);
    }

    // 无活跃时段 → 创建新时段
    create_new_session_with_conn(conn, &today, current_timestamp)
}

/// 获取当前活跃时段
///
/// # Returns
/// * `Some(session)` - 存在活跃时段
/// * `None` - 没有活跃时段
pub fn get_current_session() -> AppResult<Option<Session>> {
    let db = DB_CONNECTION.lock()?;
    let conn = db
        .as_ref()
        .ok_or_else(|| AppError::database("Database not initialized"))?;

    let today = get_today_date();
    get_active_session_with_conn(conn, &today)
}

/// 获取今日所有时段
///
/// # Returns
/// * `Vec<Session>` - 今日所有时段列表（按开始时间升序）
///
/// Note: This function is kept for backward compatibility. The command is now in session_commands.rs.
pub async fn get_today_sessions() -> AppResult<Vec<Session>> {
    get_today_sessions_sync()
}

/// 同步版本：获取今日所有时段
pub fn get_today_sessions_sync() -> AppResult<Vec<Session>> {
    let db = DB_CONNECTION.lock()?;
    let conn = db
        .as_ref()
        .ok_or_else(|| AppError::database("Database not initialized"))?;

    let today = get_today_date();
    get_sessions_by_date_with_conn(conn, &today)
}

/// 结束当前活跃时段
///
/// 通常在应用退出时调用
pub fn end_current_session() -> AppResult<()> {
    let db = DB_CONNECTION.lock()?;
    let conn = db
        .as_ref()
        .ok_or_else(|| AppError::database("Database not initialized"))?;

    let today = get_today_date();
    if let Some(active_session) = get_active_session_with_conn(conn, &today)? {
        end_session_with_conn(conn, active_session.id)?;
    }

    Ok(())
}

// ── 内部辅助函数 ───────────────────────────────────────────────────────────

/// 从 RFC3339 时间戳中提取日期部分
fn extract_date_from_timestamp(timestamp: &str) -> String {
    crate::extract_date(timestamp)
}

/// 计算两个时间戳之间的分钟数差
fn calc_gap_minutes(start: &str, end: &str) -> AppResult<i64> {
    crate::calc_gap_minutes(start, end).map_err(AppError::from)
}

/// 使用已有连接获取活跃时段
fn get_active_session_with_conn(
    conn: &rusqlite::Connection,
    date: &str,
) -> AppResult<Option<Session>> {
    let result = conn.query_row(
        "SELECT s.id, s.date, s.start_time, s.end_time, s.ai_summary, s.user_summary, s.context_for_next, s.status,
                (SELECT COUNT(*) FROM records WHERE session_id = s.id AND screenshot_path IS NOT NULL) as screenshot_count
         FROM sessions s
         WHERE s.date = ?1 AND s.status = 'active'
         ORDER BY s.start_time DESC
         LIMIT 1",
        params![date],
        |row| {
            Ok(Session {
                id: row.get(0)?,
                date: row.get(1)?,
                start_time: row.get(2)?,
                end_time: row.get(3)?,
                ai_summary: row.get(4)?,
                user_summary: row.get(5)?,
                context_for_next: row.get(6)?,
                status: SessionStatus::from(row.get::<_, String>(7)?),
                screenshot_count: row.get(8)?,
            })
        },
    );

    match result {
        Ok(session) => Ok(Some(session)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(AppError::database(format!(
            "Failed to query active session: {}",
            e
        ))),
    }
}

/// 使用已有连接获取指定时段的最后一条记录时间戳
fn get_last_record_timestamp_with_conn(
    conn: &rusqlite::Connection,
    session_id: i64,
) -> AppResult<Option<String>> {
    let result = conn.query_row(
        "SELECT timestamp FROM records WHERE session_id = ?1 ORDER BY timestamp DESC LIMIT 1",
        params![session_id],
        |row| row.get::<_, String>(0),
    );

    match result {
        Ok(timestamp) => Ok(Some(timestamp)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(AppError::database(format!(
            "Failed to query last record timestamp: {}",
            e
        ))),
    }
}

/// 使用已有连接创建新时段
fn create_new_session_with_conn(
    conn: &rusqlite::Connection,
    date: &str,
    start_time: &str,
) -> AppResult<i64> {
    conn.execute(
        "INSERT INTO sessions (date, start_time, status) VALUES (?1, ?2, 'active')",
        params![date, start_time],
    )?;

    Ok(conn.last_insert_rowid())
}

/// 使用已有连接结束时段
fn end_session_with_conn(conn: &rusqlite::Connection, session_id: i64) -> AppResult<()> {
    let end_time = Local::now().to_rfc3339();

    conn.execute(
        "UPDATE sessions SET end_time = ?1, status = 'ended' WHERE id = ?2",
        params![end_time, session_id],
    )?;

    Ok(())
}

/// 使用已有连接获取指定日期的所有时段
fn get_sessions_by_date_with_conn(
    conn: &rusqlite::Connection,
    date: &str,
) -> AppResult<Vec<Session>> {
    let mut stmt = conn.prepare(
        "SELECT s.id, s.date, s.start_time, s.end_time, s.ai_summary, s.user_summary, s.context_for_next, s.status,
                (SELECT COUNT(*) FROM records WHERE session_id = s.id AND screenshot_path IS NOT NULL) as screenshot_count
         FROM sessions s
         WHERE s.date = ?1
         ORDER BY s.start_time ASC",
    )?;

    let sessions = stmt
        .query_map(params![date], |row| {
            Ok(Session {
                id: row.get(0)?,
                date: row.get(1)?,
                start_time: row.get(2)?,
                end_time: row.get(3)?,
                ai_summary: row.get(4)?,
                user_summary: row.get(5)?,
                context_for_next: row.get(6)?,
                status: SessionStatus::from(row.get::<_, String>(7)?),
                screenshot_count: row.get(8)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(sessions)
}

/// SESSION-002: Get previous session's context for continuous analysis
///
/// Returns the `context_for_next` field from the previous session (same date, earlier time).
/// This context is passed to the AI to help understand the continuity of work.
pub fn get_previous_session_context(session_id: i64) -> AppResult<Option<String>> {
    let db = DB_CONNECTION.lock()?;
    let conn = db
        .as_ref()
        .ok_or_else(|| AppError::database("Database not initialized"))?;

    // Get current session's date and start_time
    let current_session: Option<(String, String)> = conn
        .query_row(
            "SELECT date, start_time FROM sessions WHERE id = ?1",
            params![session_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()?;

    let Some((date, start_time)) = current_session else {
        return Ok(None);
    };

    // Find previous session on the same date
    let result = conn
        .query_row(
            "SELECT context_for_next FROM sessions
             WHERE date = ?1 AND start_time < ?2 AND context_for_next IS NOT NULL AND context_for_next != ''
             ORDER BY start_time DESC
             LIMIT 1",
            params![date, start_time],
            |row| row.get::<_, String>(0),
        )
        .optional()?;

    Ok(result)
}

// ── SESSION-002: 批量分析实现 ─────────────────────────────────────────────────

/// SESSION-002: Analyze a session's screenshots in batch
///
/// Collects all pending screenshots in a session, sends them to the Vision API
/// together with context from the previous session, and stores the analysis results.
///
/// Note: This function is kept for backward compatibility. The command is now in session_commands.rs.
pub async fn analyze_session(session_id: i64) -> AppResult<()> {
    // 1. Get screenshots for this session
    let screenshots = crate::memory_storage::get_records_by_session_id(session_id)?;

    if screenshots.is_empty() {
        return Err(AppError::validation("No pending screenshots in session"));
    }

    tracing::info!(
        "{}",
        serde_json::json!({
            "event": "analyze_session_start",
            "session_id": session_id,
            "screenshot_count": screenshots.len(),
        })
    );

    // 2. Get previous session context
    let previous_context = get_previous_session_context(session_id)?;

    // 3. Load API config (uses vision-capable model)
    let config = crate::synthesis::load_vision_api_config()?;

    // 4. Build multi-image request
    let request = crate::services::vision_api::build_multi_image_request(
        &screenshots,
        previous_context.as_deref(),
        &config,
    )?;

    // 5. Call Vision API (with retry logic for transient errors)
    let response =
        crate::services::vision_api::call_vision_api_batch_with_retry(&request, &config).await?;

    // 6. Validate response
    if response.per_screenshot_analysis.len() != screenshots.len() {
        return Err(AppError::validation(format!(
            "Analysis count mismatch: expected {}, got {}",
            screenshots.len(),
            response.per_screenshot_analysis.len()
        )));
    }

    // 7. Store results
    for (screenshot, analysis) in screenshots
        .iter()
        .zip(response.per_screenshot_analysis.iter())
    {
        let content = serde_json::json!({
            "current_focus": analysis.current_focus,
            "active_software": analysis.active_software,
            "context_keywords": analysis.context_keywords,
            "tags": analysis.tags,
        })
        .to_string();

        crate::memory_storage::update_record_analysis(screenshot.record_id, &content)?;
    }

    // 8. Update session with summary
    crate::memory_storage::update_session_analysis(
        session_id,
        &response.session_summary,
        &response.context_for_next,
    )?;

    tracing::info!(
        "{}",
        serde_json::json!({
            "event": "analyze_session_complete",
            "session_id": session_id,
            "screenshot_count": screenshots.len(),
        })
    );

    Ok(())
}

/// SESSION-002: Get screenshots for a session
/// Note: This function is kept for backward compatibility. The command is now in session_commands.rs.
pub async fn get_session_screenshots(session_id: i64) -> AppResult<Vec<SessionScreenshot>> {
    crate::memory_storage::get_records_by_session_id(session_id)
}

// ── SESSION-003: User summary editing ────────────────────────────────────────

/// SESSION-003: Update user summary for a session
pub fn update_session_user_summary_sync(
    session_id: i64,
    user_summary: Option<&str>,
) -> AppResult<()> {
    let db = crate::memory_storage::DB_CONNECTION.lock()?;
    let conn = db
        .as_ref()
        .ok_or_else(|| AppError::database("Database not initialized"))?;

    let rows_affected = conn.execute(
        "UPDATE sessions SET user_summary = ?1 WHERE id = ?2",
        params![user_summary, session_id],
    )?;

    if rows_affected == 0 {
        return Err(AppError::database(format!(
            "Session with id {} not found",
            session_id
        )));
    }

    tracing::info!("Updated user summary for session {}", session_id);
    Ok(())
}

/// SESSION-003: Update user summary for a session (Tauri command)
/// Note: This function is kept for backward compatibility. The command is now in session_commands.rs.
pub async fn update_session_user_summary(
    session_id: i64,
    user_summary: Option<String>,
) -> AppResult<()> {
    update_session_user_summary_sync(session_id, user_summary.as_deref())
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_date_from_timestamp() {
        assert_eq!(
            extract_date_from_timestamp("2026-03-22T10:30:00Z"),
            "2026-03-22"
        );
        assert_eq!(
            extract_date_from_timestamp("2026-03-22T10:30:00+08:00"),
            "2026-03-22"
        );
    }

    #[test]
    fn test_calc_gap_minutes() {
        let gap = calc_gap_minutes("2026-03-22T10:00:00Z", "2026-03-22T10:30:00Z").unwrap();
        assert_eq!(gap, 30);

        let gap = calc_gap_minutes("2026-03-22T10:00:00Z", "2026-03-22T11:15:00Z").unwrap();
        assert_eq!(gap, 75);
    }

    #[test]
    fn test_session_status_conversion() {
        assert_eq!(
            SessionStatus::from("active".to_string()),
            SessionStatus::Active
        );
        assert_eq!(
            SessionStatus::from("ended".to_string()),
            SessionStatus::Ended
        );
        assert_eq!(
            SessionStatus::from("analyzed".to_string()),
            SessionStatus::Analyzed
        );
        assert_eq!(
            SessionStatus::from("unknown".to_string()),
            SessionStatus::Active
        );

        assert_eq!(String::from(SessionStatus::Active), "active");
        assert_eq!(String::from(SessionStatus::Ended), "ended");
        assert_eq!(String::from(SessionStatus::Analyzed), "analyzed");
    }

    #[test]
    fn test_session_default() {
        let session = Session::default();
        assert!(session.id == 0);
        assert!(session.end_time.is_none());
        assert!(session.ai_summary.is_none());
        assert!(session.user_summary.is_none());
        assert!(session.context_for_next.is_none());
        assert_eq!(session.status, SessionStatus::Active);
    }
}
