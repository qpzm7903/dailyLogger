//! Session service - Business logic for session management
//!
//! This module contains business logic functions for session operations.
//! Commands should delegate to these service functions rather than implementing logic directly.
//!
//! SESSION-001: Session management - detect, create and manage work sessions
//! SESSION-002: Batch analysis of session screenshots via Vision API
//! SESSION-003: User summary editing for sessions

use chrono::Local;

use crate::memory_storage::{get_settings_sync, SessionScreenshot, DB_CONNECTION};
use crate::services::vision_api;

use rusqlite::{params, OptionalExtension};

// ── Data Types ────────────────────────────────────────────────────────────────

// Re-export session types from session_manager (canonical definitions)
pub use crate::session_manager::{
    ScreenshotAnalysis, Session, SessionAnalysisResponse, SessionStatus,
};

// ── Session CRUD Operations ───────────────────────────────────────────────────

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
pub fn detect_or_create_session(current_timestamp: &str) -> Result<i64, String> {
    let db = DB_CONNECTION
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;
    let conn = db.as_ref().ok_or("Database not initialized")?;

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
pub fn get_current_session() -> Result<Option<Session>, String> {
    let db = DB_CONNECTION
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;
    let conn = db.as_ref().ok_or("Database not initialized")?;

    let today = get_today_date();
    get_active_session_with_conn(conn, &today)
}

/// 获取今日所有时段
pub fn get_today_sessions_service() -> Result<Vec<Session>, String> {
    let db = DB_CONNECTION
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;
    let conn = db.as_ref().ok_or("Database not initialized")?;

    let today = get_today_date();
    get_sessions_by_date_with_conn(conn, &today)
}

/// 结束当前活跃时段
pub fn end_current_session() -> Result<(), String> {
    let db = DB_CONNECTION
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;
    let conn = db.as_ref().ok_or("Database not initialized")?;

    let today = get_today_date();
    if let Some(active_session) = get_active_session_with_conn(conn, &today)? {
        end_session_with_conn(conn, active_session.id)?;
    }

    Ok(())
}

// ── Internal Helpers ──────────────────────────────────────────────────────────

/// 从 RFC3339 时间戳中提取日期部分
fn extract_date_from_timestamp(timestamp: &str) -> String {
    crate::extract_date(timestamp)
}

/// 计算两个时间戳之间的分钟数差
fn calc_gap_minutes(start: &str, end: &str) -> Result<i64, String> {
    crate::calc_gap_minutes(start, end)
}

/// 使用已有连接获取活跃时段
fn get_active_session_with_conn(
    conn: &rusqlite::Connection,
    date: &str,
) -> Result<Option<Session>, String> {
    use rusqlite::params;

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
        Err(e) => Err(format!("Failed to query active session: {}", e)),
    }
}

/// 使用已有连接获取指定时段的最后一条记录时间戳
fn get_last_record_timestamp_with_conn(
    conn: &rusqlite::Connection,
    session_id: i64,
) -> Result<Option<String>, String> {
    use rusqlite::params;

    let result = conn.query_row(
        "SELECT timestamp FROM records WHERE session_id = ?1 ORDER BY timestamp DESC LIMIT 1",
        params![session_id],
        |row| row.get::<_, String>(0),
    );

    match result {
        Ok(timestamp) => Ok(Some(timestamp)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(format!("Failed to query last record timestamp: {}", e)),
    }
}

/// 使用已有连接创建新时段
fn create_new_session_with_conn(
    conn: &rusqlite::Connection,
    date: &str,
    start_time: &str,
) -> Result<i64, String> {
    use rusqlite::params;

    conn.execute(
        "INSERT INTO sessions (date, start_time, status) VALUES (?1, ?2, 'active')",
        params![date, start_time],
    )
    .map_err(|e| format!("Failed to create session: {}", e))?;

    Ok(conn.last_insert_rowid())
}

/// 使用已有连接结束时段
fn end_session_with_conn(conn: &rusqlite::Connection, session_id: i64) -> Result<(), String> {
    use rusqlite::params;

    let end_time = Local::now().to_rfc3339();

    conn.execute(
        "UPDATE sessions SET end_time = ?1, status = 'ended' WHERE id = ?2",
        params![end_time, session_id],
    )
    .map_err(|e| format!("Failed to end session: {}", e))?;

    Ok(())
}

/// 使用已有连接获取指定日期的所有时段
fn get_sessions_by_date_with_conn(
    conn: &rusqlite::Connection,
    date: &str,
) -> Result<Vec<Session>, String> {
    use rusqlite::params;

    let mut stmt = conn
        .prepare(
            "SELECT s.id, s.date, s.start_time, s.end_time, s.ai_summary, s.user_summary, s.context_for_next, s.status,
                    (SELECT COUNT(*) FROM records WHERE session_id = s.id AND screenshot_path IS NOT NULL) as screenshot_count
             FROM sessions s
             WHERE s.date = ?1
             ORDER BY s.start_time ASC",
        )
        .map_err(|e| format!("Failed to prepare query: {}", e))?;

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
        })
        .map_err(|e| format!("Failed to query sessions: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to collect sessions: {}", e))?;

    Ok(sessions)
}

/// SESSION-002: Get previous session's context for continuous analysis
pub fn get_previous_session_context(session_id: i64) -> Result<Option<String>, String> {
    let db = DB_CONNECTION
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;
    let conn = db.as_ref().ok_or("Database not initialized")?;

    // Get current session's date and start_time
    let current_session: Option<(String, String)> = conn
        .query_row(
            "SELECT date, start_time FROM sessions WHERE id = ?1",
            params![session_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()
        .map_err(|e| format!("Failed to query session: {}", e))?;

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
        .optional()
        .map_err(|e| format!("Failed to query previous session: {}", e))?;

    Ok(result)
}

// ── SESSION-002: Batch Analysis ──────────────────────────────────────────────

/// SESSION-002: Analyze a session's screenshots in batch
///
/// Collects all pending screenshots in a session, sends them to the Vision API
/// together with context from the previous session, and stores the analysis results.
pub async fn analyze_session_service(session_id: i64) -> Result<(), String> {
    // 1. Get screenshots for this session
    let screenshots = crate::memory_storage::get_records_by_session_id(session_id)?;

    if screenshots.is_empty() {
        return Err("No pending screenshots in session".to_string());
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
    let request =
        vision_api::build_multi_image_request(&screenshots, previous_context.as_deref(), &config)?;

    // 5. Call Vision API (with retry logic for transient errors)
    let response = vision_api::call_vision_api_batch_with_retry(&request, &config).await?;

    // 6. Validate response
    if response.per_screenshot_analysis.len() != screenshots.len() {
        return Err(format!(
            "Analysis count mismatch: expected {}, got {}",
            screenshots.len(),
            response.per_screenshot_analysis.len()
        ));
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

// ── SESSION-003: User Summary Editing ────────────────────────────────────────

/// Update user summary for a session
pub fn update_session_user_summary_service(
    session_id: i64,
    user_summary: Option<&str>,
) -> Result<(), String> {
    use rusqlite::params;

    let db = DB_CONNECTION
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;
    let conn = db.as_ref().ok_or("Database not initialized")?;

    let rows_affected = conn
        .execute(
            "UPDATE sessions SET user_summary = ?1 WHERE id = ?2",
            params![user_summary, session_id],
        )
        .map_err(|e| format!("Failed to update session user summary: {}", e))?;

    if rows_affected == 0 {
        return Err(format!("Session with id {} not found", session_id));
    }

    tracing::info!("Updated user summary for session {}", session_id);
    Ok(())
}

/// Get screenshots for a session
pub fn get_session_screenshots_service(session_id: i64) -> Result<Vec<SessionScreenshot>, String> {
    crate::memory_storage::get_records_by_session_id(session_id)
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
