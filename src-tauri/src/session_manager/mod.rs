//! SESSION-001: 工作时段管理模块
//!
//! 负责检测、创建和管理用户的工作时段（Session）。
//! 时段是指连续工作的一个时间段，两次截图间隔超过阈值时自动创建新时段。

use chrono::{DateTime, Local, Utc};
use rusqlite::{params, OptionalExtension};
use serde::{Deserialize, Serialize};
use tauri::command;

use crate::memory_storage::{get_settings_sync, DB_CONNECTION};

/// Default prompt for session batch analysis
const DEFAULT_SESSION_ANALYSIS_PROMPT: &str = r#"你是一个工作分析助手。用户在一段时间内连续工作了 N 分钟，期间截取了多张屏幕截图。

请分析这些截图，理解用户在这段时间内的工作内容，返回以下 JSON 格式：

{
  "per_screenshot_analysis": [
    {
      "timestamp": "2026-03-22T10:05:00Z",
      "current_focus": "正在编写 Rust 代码",
      "active_software": "VS Code",
      "context_keywords": ["Rust", "Tauri", "异步"],
      "tags": ["开发"]
    }
  ],
  "session_summary": "用户在这段时间主要进行 Rust 后端开发，实现了工作时段管理功能...",
  "context_for_next": "正在开发 session_manager 模块，下一步需要实现 analyze_session 函数..."
}

注意：
1. per_screenshot_analysis 数组长度必须与输入截图数量一致
2. session_summary 应概括整个时段的工作内容
3. context_for_next 用于帮助下一时段理解连续性工作
4. tags 从以下列表选择 1-3 个最相关的: ["开发", "会议", "写作", "学习", "研究", "沟通", "规划", "文档", "测试", "设计"]

上一时段上下文（如有）：
{previous_context}

返回纯 JSON，不要添加任何其他文字。"#;

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
    pub screenshot_count: Option<i64>,    // 时段内的截图数量
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
///
/// # Returns
/// * `Some(session)` - 存在活跃时段
/// * `None` - 没有活跃时段
pub fn get_current_session() -> Result<Option<Session>, String> {
    let db = DB_CONNECTION
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;
    let conn = db.as_ref().ok_or("Database not initialized")?;

    let today = get_today_date();
    get_active_session_with_conn(conn, &today)
}

/// 获取今日所有时段
///
/// # Returns
/// * `Vec<Session>` - 今日所有时段列表（按开始时间升序）
#[command]
pub async fn get_today_sessions() -> Result<Vec<Session>, String> {
    get_today_sessions_sync()
}

/// 同步版本：获取今日所有时段
pub fn get_today_sessions_sync() -> Result<Vec<Session>, String> {
    let db = DB_CONNECTION
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;
    let conn = db.as_ref().ok_or("Database not initialized")?;

    let today = get_today_date();
    get_sessions_by_date_with_conn(conn, &today)
}

/// 结束当前活跃时段
///
/// 通常在应用退出时调用
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

// ── 内部辅助函数 ───────────────────────────────────────────────────────────

/// 从 RFC3339 时间戳中提取日期部分
fn extract_date_from_timestamp(timestamp: &str) -> String {
    timestamp.split('T').next().unwrap_or(timestamp).to_string()
}

/// 计算两个时间戳之间的分钟数差
fn calc_gap_minutes(start: &str, end: &str) -> Result<i64, String> {
    let start_time: DateTime<Utc> = start
        .parse()
        .map_err(|e: chrono::ParseError| format!("Invalid start timestamp: {}", e))?;
    let end_time: DateTime<Utc> = end
        .parse()
        .map_err(|e: chrono::ParseError| format!("Invalid end timestamp: {}", e))?;

    let duration = end_time.signed_duration_since(start_time);
    Ok(duration.num_minutes())
}

/// 使用已有连接获取活跃时段
fn get_active_session_with_conn(
    conn: &rusqlite::Connection,
    date: &str,
) -> Result<Option<Session>, String> {
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
    conn.execute(
        "INSERT INTO sessions (date, start_time, status) VALUES (?1, ?2, 'active')",
        params![date, start_time],
    )
    .map_err(|e| format!("Failed to create session: {}", e))?;

    Ok(conn.last_insert_rowid())
}

/// 使用已有连接结束时段
fn end_session_with_conn(conn: &rusqlite::Connection, session_id: i64) -> Result<(), String> {
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
///
/// Returns the `context_for_next` field from the previous session (same date, earlier time).
/// This context is passed to the AI to help understand the continuity of work.
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

// ── SESSION-002: 批量分析实现 ─────────────────────────────────────────────────

/// API configuration for session analysis
struct ApiConfig {
    api_base_url: String,
    api_key: String,
    model_name: String,
    custom_headers: Vec<crate::memory_storage::CustomHeader>,
}

/// Load API configuration from settings
fn load_api_config() -> Result<ApiConfig, String> {
    let settings = get_settings_sync()?;

    let api_base_url = settings.api_base_url.ok_or("API Base URL not configured")?;
    let api_key = settings.api_key.clone().unwrap_or_default();
    let model_name = settings
        .model_name
        .clone()
        .unwrap_or_else(|| "gpt-4o".to_string());

    // Parse custom headers
    let custom_headers = if let Some(ref headers_json) = settings.custom_headers {
        if !headers_json.is_empty() {
            serde_json::from_str::<Vec<crate::memory_storage::CustomHeader>>(headers_json)
                .unwrap_or_default()
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    Ok(ApiConfig {
        api_base_url,
        api_key,
        model_name,
        custom_headers,
    })
}

/// Read and encode screenshot as base64
fn encode_screenshot(path: &str) -> Result<String, String> {
    let bytes =
        std::fs::read(path).map_err(|e| format!("Failed to read screenshot {}: {}", path, e))?;
    Ok(base64::Engine::encode(
        &base64::engine::general_purpose::STANDARD,
        &bytes,
    ))
}

/// Build multi-image request for Vision API
fn build_multi_image_request(
    screenshots: &[SessionScreenshot],
    previous_context: Option<&str>,
    config: &ApiConfig,
) -> Result<serde_json::Value, String> {
    let prompt = DEFAULT_SESSION_ANALYSIS_PROMPT
        .replace("{previous_context}", previous_context.unwrap_or("无"));

    let mut content: Vec<serde_json::Value> = vec![serde_json::json!({
        "type": "text",
        "text": prompt
    })];

    for screenshot in screenshots {
        let base64_image = encode_screenshot(&screenshot.screenshot_path)?;
        content.push(serde_json::json!({
            "type": "image_url",
            "image_url": {
                "url": format!("data:image/png;base64,{}", base64_image)
            }
        }));
    }

    Ok(serde_json::json!({
        "model": config.model_name,
        "messages": [{
            "role": "user",
            "content": content
        }],
        "max_tokens": 4000
    }))
}

/// Call Vision API for batch analysis
async fn call_vision_api_batch(
    request: &serde_json::Value,
    config: &ApiConfig,
) -> Result<SessionAnalysisResponse, String> {
    let endpoint = format!("{}/chat/completions", config.api_base_url);
    let client = crate::create_http_client(&endpoint, 180)?;

    let masked_key = crate::mask_api_key(&config.api_key);

    tracing::info!(
        "{}",
        serde_json::json!({
            "event": "session_analysis_request",
            "endpoint": endpoint,
            "model": config.model_name,
            "api_key_masked": masked_key,
        })
    );

    let start = std::time::Instant::now();
    let mut request_builder = client
        .post(&endpoint)
        .header("Content-Type", "application/json")
        .json(request);

    // Check if custom headers contain auth
    let has_custom_auth = config
        .custom_headers
        .iter()
        .any(|h| h.key.to_lowercase() == "authorization" || h.key.to_lowercase() == "api-key");

    if !config.api_key.is_empty() && !has_custom_auth {
        request_builder =
            request_builder.header("Authorization", format!("Bearer {}", config.api_key));
    }

    for header in &config.custom_headers {
        request_builder = request_builder.header(&header.key, &header.value);
    }

    let response = request_builder.send().await.map_err(|e| {
        tracing::error!("Session analysis API call failed: {}", e);
        format!("API request failed: {}", e)
    })?;

    let elapsed_ms = start.elapsed().as_millis();

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        tracing::error!(
            "{}",
            serde_json::json!({
                "event": "session_analysis_error",
                "status": status.as_u16(),
                "response_body": body,
                "elapsed_ms": elapsed_ms,
            })
        );
        return Err(format!("API error ({}): {}", status, body));
    }

    let response_json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    let content = response_json["choices"][0]["message"]["content"]
        .as_str()
        .ok_or("No content in response")?;

    tracing::info!(
        "{}",
        serde_json::json!({
            "event": "session_analysis_response",
            "elapsed_ms": elapsed_ms,
            "usage": response_json.get("usage"),
        })
    );

    // Strip markdown code fences if present
    let content = content.trim();
    let content = if let Some(inner) = content
        .strip_prefix("```json")
        .or_else(|| content.strip_prefix("```"))
    {
        inner.trim_end_matches("```").trim()
    } else {
        content
    };

    let analysis: SessionAnalysisResponse = serde_json::from_str(content)
        .map_err(|e| format!("Failed to parse analysis JSON: {}. Content: {}", e, content))?;

    Ok(analysis)
}

/// SESSION-002: Analyze a session's screenshots in batch
///
/// Collects all pending screenshots in a session, sends them to the Vision API
/// together with context from the previous session, and stores the analysis results.
///
/// # Arguments
/// * `session_id` - The ID of the session to analyze
///
/// # Returns
/// * `Ok(())` - Analysis successful
/// * `Err(String)` - Analysis failed
#[command]
pub async fn analyze_session(session_id: i64) -> Result<(), String> {
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

    // 3. Load API config
    let config = load_api_config()?;

    // 4. Build multi-image request
    let request = build_multi_image_request(&screenshots, previous_context.as_deref(), &config)?;

    // 5. Call Vision API
    let response = call_vision_api_batch(&request, &config).await?;

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

/// SESSION-002: Get screenshots for a session (Tauri command)
#[command]
pub async fn get_session_screenshots(session_id: i64) -> Result<Vec<SessionScreenshot>, String> {
    crate::memory_storage::get_records_by_session_id(session_id)
}

// ── SESSION-003: User summary editing ────────────────────────────────────────

/// SESSION-003: Update user summary for a session
pub fn update_session_user_summary_sync(
    session_id: i64,
    user_summary: Option<&str>,
) -> Result<(), String> {
    let db = crate::memory_storage::DB_CONNECTION
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

/// SESSION-003: Update user summary for a session (Tauri command)
#[command]
pub async fn update_session_user_summary(
    session_id: i64,
    user_summary: Option<String>,
) -> Result<(), String> {
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
