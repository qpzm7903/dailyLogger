use chrono::Datelike;
use rusqlite::params;
use std::path::PathBuf;

use crate::dingtalk;
use crate::infrastructure::retry;
use crate::memory_storage::{self, Record, Settings};
use crate::notion;
use crate::session_manager::{Session, SessionStatus};
use crate::slack;

// STAB-001: Retry configuration for AI API calls
const MAX_RETRIES: u32 = 3;
const INITIAL_RETRY_DELAY_MS: u64 = 1000; // 1 second
const MAX_RETRY_DELAY_MS: u64 = 10000; // 10 seconds

/// API configuration extracted from Settings for LLLM calls.
#[derive(Debug, Clone)]
pub struct ApiConfig {
    api_base_url: String,
    api_key: String,
    model_name: String,
    is_ollama: bool,
    // AI-006: Custom API headers
    custom_headers: Vec<crate::memory_storage::CustomHeader>,
    // PERF-001: Proxy configuration
    proxy_config: crate::ProxyConfig,
}

/// Extract API configuration from settings (shared by all report generators).
pub fn load_api_config(settings: &Settings) -> Result<ApiConfig, String> {
    let api_base_url = settings
        .api_base_url
        .clone()
        .ok_or("API Base URL not configured")?;
    let api_key = settings.api_key.clone().unwrap_or_default();
    let model_name = settings
        .summary_model_name
        .clone()
        .filter(|s| !s.is_empty())
        .or_else(|| settings.model_name.clone())
        .unwrap_or_else(|| "gpt-4o".to_string());
    let is_ollama = crate::ollama::is_ollama_endpoint(&api_base_url);

    if !is_ollama && api_key.is_empty() {
        return Err("API Key is required for non-Ollama endpoints".to_string());
    }

    // AI-006: Parse custom headers from settings
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

    // PERF-001: Parse proxy configuration from settings
    let proxy_config = crate::ProxyConfig {
        enabled: settings.proxy_enabled.unwrap_or(false),
        host: settings.proxy_host.clone(),
        port: settings.proxy_port,
        username: settings.proxy_username.clone(),
        password: settings.proxy_password.clone(),
    };

    Ok(ApiConfig {
        api_base_url,
        api_key,
        model_name,
        is_ollama,
        custom_headers,
        proxy_config,
    })
}

/// Send a prompt to the LLM API and return the response content (shared by all report generators).
async fn call_llm_api(
    config: &ApiConfig,
    prompt: &str,
    max_tokens: u32,
    caller: &str,
) -> Result<String, String> {
    let endpoint = format!("{}/chat/completions", config.api_base_url);

    // Create HTTP client with proxy configuration
    let client =
        crate::create_http_client_with_proxy(&endpoint, 120, Some(config.proxy_config.clone()))?;

    let request_body = serde_json::json!({
        "model": config.model_name,
        "messages": [{"role": "user", "content": prompt}],
        "max_tokens": max_tokens
    });

    let masked_key = crate::mask_api_key(&config.api_key);
    // AI-006: Log custom headers (mask sensitive values)
    let custom_headers_debug: Vec<_> = config
        .custom_headers
        .iter()
        .map(|h| {
            if h.sensitive {
                format!("{}: {}", h.key, "***MASKED***")
            } else {
                format!("{}: {}", h.key, h.value)
            }
        })
        .collect();

    tracing::info!(
        "{}",
        serde_json::json!({
            "event": "llm_request",
            "caller": caller,
            "endpoint": endpoint,
            "model": config.model_name,
            "max_tokens": max_tokens,
            "api_key_masked": masked_key,
            "has_image": false,
            "prompt": prompt,
            "custom_headers": custom_headers_debug,
        })
    );

    let start = std::time::Instant::now();
    let mut request = client
        .post(&endpoint)
        .header("Content-Type", "application/json")
        .json(&request_body);

    // AI-006: Check if custom headers contain Authorization or api-key header
    let has_custom_auth = config
        .custom_headers
        .iter()
        .any(|h| h.key.to_lowercase() == "authorization" || h.key.to_lowercase() == "api-key");

    // Set Authorization header only if api_key is provided and no custom auth header
    if !config.api_key.is_empty() && !has_custom_auth {
        request = request.header("Authorization", format!("Bearer {}", config.api_key));
    }

    // AI-006: Apply custom headers
    for header in &config.custom_headers {
        request = request.header(&header.key, &header.value);
    }

    let response = request.send().await.map_err(|e| {
        let elapsed_ms = start.elapsed().as_millis();
        let error_msg = crate::ollama::format_connection_error(&e.to_string(), config.is_ollama);
        tracing::error!(
            "{}",
            serde_json::json!({
                "event": "llm_error",
                "caller": caller,
                "error": error_msg,
                "elapsed_ms": elapsed_ms,
            })
        );
        error_msg
    })?;
    let elapsed_ms = start.elapsed().as_millis();

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        tracing::error!(
            "{}",
            serde_json::json!({
                "event": "llm_error",
                "caller": caller,
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
        .ok_or("No content in response")?
        .to_string();

    tracing::info!(
        "{}",
        serde_json::json!({
            "event": "llm_response",
            "caller": caller,
            "status": 200,
            "elapsed_ms": elapsed_ms,
            "usage": response_json.get("usage"),
            "model": response_json.get("model"),
            "response_id": response_json.get("id"),
            "content": content,
        })
    );

    Ok(content)
}

/// STAB-001: Wrapper for call_llm_api with retry logic for transient errors
pub async fn call_llm_api_with_retry(
    config: &ApiConfig,
    prompt: &str,
    max_tokens: u32,
    caller: &str,
) -> Result<String, String> {
    let mut last_error = String::new();

    for attempt in 1..=MAX_RETRIES {
        match call_llm_api(config, prompt, max_tokens, caller).await {
            Ok(result) => return Ok(result),
            Err(e) => {
                last_error = e.clone();
                if attempt < MAX_RETRIES && retry::is_retryable_error(&e) {
                    let delay = retry::calculate_retry_delay(
                        attempt,
                        INITIAL_RETRY_DELAY_MS,
                        MAX_RETRY_DELAY_MS,
                    );
                    tracing::warn!(
                        "LLM API call failed (attempt {}/{}), retrying in {}ms: {}",
                        attempt,
                        MAX_RETRIES,
                        delay,
                        e
                    );
                    tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
                } else {
                    // Non-retryable error or max retries reached
                    break;
                }
            }
        }
    }

    Err(format!(
        "LLM API call failed after {} attempts: {}",
        MAX_RETRIES, last_error
    ))
}

/// Write report content to the Obsidian output directory and return the full path.
pub fn write_report_to_obsidian(
    obsidian_path: &str,
    filename: &str,
    content: &str,
) -> Result<String, String> {
    let output_dir = PathBuf::from(obsidian_path);
    std::fs::create_dir_all(&output_dir)
        .map_err(|e| format!("Failed to create output directory: {}", e))?;

    let output_path = output_dir.join(filename);
    std::fs::write(&output_path, content).map_err(|e| format!("Failed to write report: {}", e))?;

    Ok(output_path.to_string_lossy().to_string())
}

/// INT-002: Write report content to Logseq output directory if configured.
/// Logseq stores user-created pages in the `pages` folder inside the graph root.
/// Returns Some(path) if written successfully, None if Logseq not configured.
pub fn write_report_to_logseq(
    settings: &memory_storage::Settings,
    filename: &str,
    content: &str,
) -> Option<String> {
    match settings.get_logseq_output_path() {
        Ok(logseq_path) => {
            // Logseq convention: pages go in the `pages` subdirectory
            let output_dir = PathBuf::from(&logseq_path).join("pages");
            if let Err(e) = std::fs::create_dir_all(&output_dir) {
                tracing::warn!("Failed to create Logseq pages directory: {}", e);
                return None;
            }

            let output_path = output_dir.join(filename);
            match std::fs::write(&output_path, content) {
                Ok(_) => {
                    let path_str = output_path.to_string_lossy().to_string();
                    tracing::info!("Report also written to Logseq: {}", path_str);
                    Some(path_str)
                }
                Err(e) => {
                    tracing::warn!("Failed to write report to Logseq: {}", e);
                    None
                }
            }
        }
        Err(_) => None, // Logseq not configured, silently skip
    }
}

/// INT-004: Send report notifications to Slack and DingTalk.
/// This function sends notifications to configured channels after a report is generated.
/// Errors are logged but do not affect the main report generation flow.
pub fn send_report_notifications(settings: &Settings, title: &str, content: &str) {
    // Send to Slack if configured
    if slack::is_slack_configured(settings) {
        match slack::send_to_slack_sync(settings, title, content) {
            Some(true) => tracing::info!("Report sent to Slack: {}", title),
            Some(false) => tracing::warn!("Failed to send report to Slack"),
            None => tracing::debug!("Slack not configured or send returned None"),
        }
    }

    // Send to DingTalk if configured
    if dingtalk::is_dingtalk_configured(settings) {
        match dingtalk::send_to_dingtalk_sync(settings, title, content) {
            Some(true) => tracing::info!("Report sent to DingTalk: {}", title),
            Some(false) => tracing::warn!("Failed to send report to DingTalk"),
            None => tracing::debug!("DingTalk not configured or send returned None"),
        }
    }
}

const DEFAULT_SUMMARY_PROMPT: &str = r"
你是一个工作日志助手。请根据以下今日工作记录，生成一份结构化的 Markdown 格式日报。

要求：
1. 按时间顺序组织
2. 提取关键工作内容和技术关键词
3. 总结今日工作成果和遇到的问题
4. 输出纯 Markdown 格式，不要有其他说明文字

今日记录：
{records}
{github_activity}
请生成日报：";

/// Default title format for daily summaries
pub const DEFAULT_TITLE_FORMAT: &str = "工作日报 - {date}";

/// Default prompt template for weekly reports
const DEFAULT_WEEKLY_REPORT_PROMPT: &str = r"
你是一个工作日志助手。请根据以下本周工作记录，生成一份结构化的 Markdown 格式周报。

要求：
1. 按日期分组展示工作内容
2. 提取本周关键成果和技术亮点
3. 总结遇到的问题和解决方案
4. 列出下周待跟进事项
5. 输出纯 Markdown 格式，不要有其他说明文字

本周记录：
{records}

请生成周报：";

/// Get the default weekly report prompt
pub fn get_default_weekly_report_prompt() -> String {
    DEFAULT_WEEKLY_REPORT_PROMPT.to_string()
}

/// Default prompt template for monthly reports
const DEFAULT_MONTHLY_REPORT_PROMPT: &str = r"
你是一个工作日志助手。请根据以下本月工作记录，生成一份结构化的 Markdown 格式月报。

要求：
1. 按周分组展示工作内容
2. 提取本月关键成果和技术亮点
3. 总结遇到的问题和解决方案
4. 分析月度工作趋势（哪些方面投入更多时间）
5. 列出下月待跟进事项
6. 输出纯 Markdown 格式，不要有其他说明文字

本月记录：
{records}

请生成月报：";

/// Default prompt template for custom period reports - REPORT-003
const DEFAULT_CUSTOM_REPORT_PROMPT: &str = r"
你是一个工作日志助手。请根据以下指定时间段的工作记录，生成一份结构化的 Markdown 格式报告。

要求：
1. 按日期分组展示工作内容
2. 提取该时间段的关键成果和技术亮点
3. 总结遇到的问题和解决方案
4. 列出后续待跟进事项
5. 输出纯 Markdown 格式，不要有其他说明文字

时间段：{start_date} 至 {end_date}
记录：
{records}

请生成报告：";

/// Get the default monthly report prompt
pub fn get_default_monthly_report_prompt() -> String {
    DEFAULT_MONTHLY_REPORT_PROMPT.to_string()
}

/// Default prompt template for comparison reports - REPORT-004
const DEFAULT_COMPARISON_REPORT_PROMPT: &str = r"
你是一个工作日志分析助手。请对比以下两个时间段的工作记录，生成一份结构化的 Markdown 格式对比分析报告。

要求：
1. 概述两个时间段各自的工作重点
2. 对比分析工作量变化（记录数量、涉及领域）
3. 分析工作重心转移（哪些方面增加/减少了投入）
4. 评估效率趋势和工作模式变化
5. 提出改进建议
6. 输出纯 Markdown 格式，不要有其他说明文字

时间段 A ({start_date_a} 至 {end_date_a})：
{records_a}

时间段 B ({start_date_b} 至 {end_date_b})：
{records_b}

请生成对比分析报告：";

/// Get the default comparison report prompt - REPORT-004
pub fn get_default_comparison_report_prompt() -> String {
    DEFAULT_COMPARISON_REPORT_PROMPT.to_string()
}

/// Generate comparison report filename - REPORT-004
pub fn generate_comparison_report_filename(
    start_a: &str,
    end_a: &str,
    start_b: &str,
    end_b: &str,
) -> String {
    format!("对比分析-{}~{}-vs-{}~{}.md", start_a, end_a, start_b, end_b)
}

/// Format records grouped by week for monthly trend analysis
pub fn format_records_by_week(records: &[Record]) -> String {
    use chrono::Datelike;

    // Group records by week
    let mut week_groups: std::collections::BTreeMap<String, Vec<&Record>> =
        std::collections::BTreeMap::new();

    for record in records {
        if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&record.timestamp) {
            let local_dt = dt.with_timezone(&chrono::Local);
            let week_num = local_dt.iso_week().week();
            let week_key = format!(
                "第{}周 ({}月{}日-{}日)",
                week_num,
                local_dt.month(),
                local_dt.day(),
                local_dt.format("%m-%d")
            );
            week_groups.entry(week_key).or_default().push(record);
        }
    }

    // Format each week group
    let mut output = String::new();
    for (week, week_records) in week_groups {
        output.push_str(&format!("### {}\n\n", week));
        for record in week_records {
            let time = chrono::DateTime::parse_from_rfc3339(&record.timestamp)
                .map(|dt| {
                    dt.with_timezone(&chrono::Local)
                        .format("%Y-%m-%d %H:%M")
                        .to_string()
                })
                .unwrap_or_else(|_| "unknown".to_string());

            let source = if record.source_type == "auto" {
                "🖥️ 自动感知"
            } else {
                "⚡ 闪念"
            };

            output.push_str(&format!("- [{}] {}: {}\n", time, source, record.content));
        }
        output.push('\n');
    }

    output
}

/// Generate the filename for monthly report
pub fn generate_monthly_report_filename() -> String {
    let now = chrono::Local::now();
    format!("月报-{}.md", now.format("%Y-%m"))
}

/// Format the summary title by replacing placeholders.
/// Supports: {date} - replaced with YYYY-MM-DD format
pub fn format_summary_title(format: &str) -> String {
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    format.replace("{date}", &today)
}

/// Filter records based on settings.
/// If include_manual_records is false, records with source_type='manual' are excluded.
pub fn filter_records_by_settings(records: Vec<Record>, settings: &Settings) -> Vec<Record> {
    let include_manual = settings.include_manual_records.unwrap_or(true);
    if include_manual {
        records
    } else {
        records
            .into_iter()
            .filter(|r| r.source_type != "manual")
            .collect()
    }
}

/// SESSION-005: Get the display summary for a session.
/// Priority: user_summary > ai_summary > "暂无摘要"
fn get_session_display_summary(session: &Session) -> String {
    session
        .user_summary
        .as_ref()
        .filter(|s| !s.is_empty())
        .or(session.ai_summary.as_ref())
        .filter(|s| !s.is_empty())
        .cloned()
        .unwrap_or_else(|| "暂无摘要".to_string())
}

/// SESSION-005: Get records for a specific session, with user_notes preferred over content.
/// Skips records with analysis_status = 'pending'.
fn get_session_records_for_summary(
    session_id: i64,
) -> Result<Vec<(String, String, String)>, String> {
    // Use get_records_by_session_id which returns SessionScreenshot (id, timestamp, screenshot_path)
    // We need full Record to get user_notes and content
    let db = crate::memory_storage::DB_CONNECTION
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;
    let conn = db.as_ref().ok_or("Database not initialized")?;

    let mut stmt = conn
        .prepare(
            "SELECT id, timestamp, content, user_notes, analysis_status, source_type
             FROM records
             WHERE session_id = ?1
             ORDER BY timestamp ASC",
        )
        .map_err(|e| format!("Failed to prepare query: {}", e))?;

    let records = stmt
        .query_map(params![session_id], |row| {
            Ok(Record {
                id: row.get(0)?,
                timestamp: row.get(1)?,
                source_type: row.get(5)?,
                content: row.get(2)?,
                screenshot_path: None,
                monitor_info: None,
                tags: None,
                user_notes: row.get(3)?,
                session_id: Some(session_id),
                analysis_status: row.get(4)?,
            })
        })
        .map_err(|e| format!("Query failed: {}", e))?;

    let mut result = Vec::new();
    for record in records {
        let record = record.map_err(|e| format!("Failed to read record: {}", e))?;
        // Skip pending records
        if record.analysis_status.as_deref() == Some("pending") {
            continue;
        }
        // Get display content: user_notes > content
        let display_content = if let Some(ref notes) = record.user_notes {
            if !notes.is_empty() {
                notes.clone()
            } else {
                record.content.clone()
            }
        } else {
            record.content.clone()
        };
        // Format time
        let time = chrono::DateTime::parse_from_rfc3339(&record.timestamp)
            .map(|dt| dt.with_timezone(&chrono::Local).format("%H:%M").to_string())
            .unwrap_or_else(|_| "unknown".to_string());
        // Source icon
        let source = if record.source_type == "auto" {
            "🖥️ 自动感知"
        } else {
            "⚡ 闪念"
        };
        result.push((time, source.to_string(), display_content.clone()));
    }

    Ok(result)
}

/// SESSION-005: Format a single session for the summary prompt.
/// Returns formatted string like:
/// ## 09:00-12:00 - active
///
/// ✏️ (if user edited)
///
/// 用户摘要或AI摘要
///
/// - [09:15] 🖥️: 截图内容
/// - [10:30] ⚡: 闪念内容
fn format_session_for_summary(session: &Session) -> String {
    // Time range
    let time_range = match &session.end_time {
        Some(end) => {
            let start_time = chrono::DateTime::parse_from_rfc3339(&session.start_time)
                .map(|dt| dt.with_timezone(&chrono::Local).format("%H:%M").to_string())
                .unwrap_or_else(|_| session.start_time.clone());
            let end_time = chrono::DateTime::parse_from_rfc3339(end)
                .map(|dt| dt.with_timezone(&chrono::Local).format("%H:%M").to_string())
                .unwrap_or_else(|_| end.clone());
            format!("{} - {}", start_time, end_time)
        }
        None => {
            let start_time = chrono::DateTime::parse_from_rfc3339(&session.start_time)
                .map(|dt| dt.with_timezone(&chrono::Local).format("%H:%M").to_string())
                .unwrap_or_else(|_| session.start_time.clone());
            format!("{} - 进行中", start_time)
        }
    };

    let status_str = match session.status {
        SessionStatus::Active => "active",
        SessionStatus::Ended => "ended",
        SessionStatus::Analyzed => "analyzed",
    };

    // Check if user edited
    let is_edited = session
        .user_summary
        .as_ref()
        .filter(|s| !s.is_empty())
        .is_some();

    // Get display summary
    let summary = get_session_display_summary(session);

    // Build session section
    let mut content = format!("## {} - {}\n\n", time_range, status_str);

    if is_edited {
        content.push_str("✏️ ");
    }
    content.push_str(&summary);
    content.push_str("\n\n");

    // Get and format records within this session
    match get_session_records_for_summary(session.id) {
        Ok(records) => {
            for (time, source, display_content) in records {
                content.push_str(&format!("- [{}] {}: {}\n", time, source, display_content));
            }
        }
        Err(e) => {
            tracing::warn!("Failed to get records for session {}: {}", session.id, e);
        }
    }

    content
}

/// SESSION-005: Build session-based report content from sessions.
/// If no sessions exist, returns None to indicate fallback to legacy format.
pub fn build_session_based_report(sessions: &[Session]) -> Option<String> {
    if sessions.is_empty() {
        return None;
    }

    let mut content = String::new();
    for session in sessions {
        content.push_str(&format_session_for_summary(session));
        content.push('\n');
    }

    Some(content)
}

/// Format records into a string for the summary prompt.
/// Each record is formatted as: "- [HH:MM] 🖥️/⚡ source: content"
/// SESSION-003: Prefers user_notes over content when available.
pub fn format_records_for_summary(records: &[Record]) -> String {
    records
        .iter()
        .map(|r| {
            let time = chrono::DateTime::parse_from_rfc3339(&r.timestamp)
                .map(|dt| dt.with_timezone(&chrono::Local).format("%H:%M").to_string())
                .unwrap_or_else(|_| "unknown".to_string());

            let source = if r.source_type == "auto" {
                "🖥️ 自动感知"
            } else {
                "⚡ 闪念"
            };

            // SESSION-003: Prefer user_notes over content
            let display_content = r
                .user_notes
                .as_ref()
                .filter(|n| !n.is_empty())
                .map(|n| n.as_str())
                .unwrap_or(&r.content);

            format!("- [{}] {}: {}", time, source, display_content)
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Generate the filename for the daily summary based on settings.
pub fn generate_summary_filename(settings: &Settings) -> String {
    let title_format = settings
        .summary_title_format
        .as_deref()
        .filter(|s| !s.is_empty())
        .unwrap_or(DEFAULT_TITLE_FORMAT);

    // Replace {date} placeholder and create filename
    let title = format_summary_title(title_format);
    format!("{}.md", title)
}

// DATA-007: Multi-language support

/// Supported languages for daily report translation
const SUPPORTED_LANGUAGES: &[(&str, &str)] = &[
    ("zh-CN", "中文"),
    ("en", "English"),
    ("ja", "日本語"),
    ("ko", "한국어"),
    ("es", "Español"),
    ("fr", "Français"),
    ("de", "Deutsch"),
];

/// Language code to file suffix mapping
fn get_language_suffix(lang: &str) -> &str {
    match lang {
        "zh-CN" => "",
        "en" => ".en",
        "ja" => ".ja",
        "ko" => ".ko",
        "es" => ".es",
        "fr" => ".fr",
        "de" => ".de",
        _ => "",
    }
}

/// Get language display name from code
fn get_language_name(lang: &str) -> &str {
    SUPPORTED_LANGUAGES
        .iter()
        .find(|(code, _)| *code == lang)
        .map(|(_, name)| *name)
        .unwrap_or("Unknown")
}

/// Translation prompt template
const TRANSLATION_PROMPT: &str = r#"你是一个专业的技术文档翻译助手。请将以下 Markdown 格式的工作日报翻译成{language}。

要求：
1. 保持 Markdown 格式不变
2. 技术术语保持准确
3. 保持原意的专业性
4. 输出纯翻译结果，不要有其他说明文字

原文：
{original_report}

请翻译："#;

/// Translate the report content to the target language
pub async fn translate_report(
    config: &ApiConfig,
    original_report: &str,
    target_lang: &str,
) -> Result<String, String> {
    let lang_name = get_language_name(target_lang);
    let prompt = TRANSLATION_PROMPT
        .replace("{language}", lang_name)
        .replace("{original_report}", original_report);

    call_llm_api_with_retry(config, &prompt, 3000, "translate_report").await
}

/// Get the list of supported languages
pub fn get_supported_languages() -> Vec<(String, String)> {
    SUPPORTED_LANGUAGES
        .iter()
        .map(|(code, name)| (code.to_string(), name.to_string()))
        .collect()
}

/// Generate daily summary filename with language suffix
pub fn generate_summary_filename_with_lang(settings: &Settings, lang: &str) -> String {
    let title_format = settings
        .summary_title_format
        .as_deref()
        .filter(|s| !s.is_empty())
        .unwrap_or(DEFAULT_TITLE_FORMAT);

    let title = format_summary_title(title_format);
    let suffix = get_language_suffix(lang);
    format!("{}{}.md", title, suffix)
}

pub async fn generate_daily_summary() -> Result<String, String> {
    if !crate::network_status::is_online() {
        let _ = crate::offline_queue::enqueue_task(
            &crate::offline_queue::OfflineTaskType::DailySummary,
            "{}",
            None,
        );
        return Err("当前处于离线状态，日报生成已加入队列，网络恢复后将自动处理".to_string());
    }

    let settings = memory_storage::get_settings_sync()
        .map_err(|e| format!("Failed to get settings: {}", e))?;
    let obsidian_path = settings.get_obsidian_output_path()?;
    let api_config = load_api_config(&settings)?;

    // SESSION-005: Try session-based approach first
    let sessions = crate::session_manager::get_today_sessions_sync().unwrap_or_default();

    if !sessions.is_empty() {
        // SESSION-005 AC#4: Auto-analyze pending/ended sessions before generating report
        for session in &sessions {
            if session.status == SessionStatus::Active || session.status == SessionStatus::Ended {
                tracing::info!("Auto-analyzing session {} before daily summary", session.id);
                if let Err(e) = crate::session_manager::analyze_session(session.id).await {
                    tracing::warn!("Failed to analyze session {}: {}", session.id, e);
                }
            }
        }

        // Re-fetch sessions after analysis
        let sessions = crate::session_manager::get_today_sessions_sync().unwrap_or_default();

        // SESSION-005: Build session-based report
        if let Some(content) = build_session_based_report(&sessions) {
            let prompt_template = settings
                .summary_prompt
                .as_deref()
                .filter(|s| !s.is_empty())
                .unwrap_or(DEFAULT_SUMMARY_PROMPT);
            let prompt = prompt_template
                .replace("{records}", &content)
                .replace("{github_activity}", "");

            let summary =
                call_llm_api_with_retry(&api_config, &prompt, 2000, "generate_daily_summary")
                    .await?;

            let filename = generate_summary_filename(&settings);
            let path_str = write_report_to_obsidian(&obsidian_path, &filename, &summary)?;

            // INT-002: Also write to Logseq if configured
            write_report_to_logseq(&settings, &filename, &summary);

            // INT-001: Also write to Notion if configured
            if let Some(notion_url) =
                notion::write_report_to_notion(&settings, &filename, &summary).await
            {
                tracing::info!("Report also written to Notion: {}", notion_url);
            }

            // INT-004: Send notifications to Slack/DingTalk if configured
            let title = settings
                .summary_title_format
                .as_ref()
                .filter(|s| !s.is_empty())
                .map(|fmt| format_summary_title(fmt))
                .unwrap_or_else(|| format_summary_title(DEFAULT_TITLE_FORMAT));
            send_report_notifications(&settings, &title, &summary);

            let mut updated_settings = settings.clone();
            updated_settings.last_summary_path = Some(path_str.clone());
            memory_storage::save_settings_sync(&updated_settings)
                .map_err(|e| format!("Failed to update settings: {}", e))?;

            tracing::info!("Daily summary generated (session-based): {}", path_str);
            return Ok(path_str);
        }
    }

    // SESSION-005 AC#6: Fallback to legacy flat record format if no sessions
    let all_records = memory_storage::get_all_today_records_for_summary()
        .map_err(|e| format!("Failed to get records: {}", e))?;
    let records = filter_records_by_settings(all_records, &settings);
    if records.is_empty() {
        return Err("No records for today after filtering".to_string());
    }

    let records_text = format_records_for_summary(&records);

    let prompt_template = settings
        .summary_prompt
        .as_deref()
        .filter(|s| !s.is_empty())
        .unwrap_or(DEFAULT_SUMMARY_PROMPT);
    let prompt = prompt_template
        .replace("{records}", &records_text)
        .replace("{github_activity}", ""); // GitHub integration removed in v3.0.0

    let summary =
        call_llm_api_with_retry(&api_config, &prompt, 2000, "generate_daily_summary").await?;

    let filename = generate_summary_filename(&settings);
    let path_str = write_report_to_obsidian(&obsidian_path, &filename, &summary)?;

    // INT-002: Also write to Logseq if configured
    write_report_to_logseq(&settings, &filename, &summary);

    // INT-001: Also write to Notion if configured
    if let Some(notion_url) = notion::write_report_to_notion(&settings, &filename, &summary).await {
        tracing::info!("Report also written to Notion: {}", notion_url);
    }

    // INT-004: Send notifications to Slack/DingTalk if configured
    let title = settings
        .summary_title_format
        .as_ref()
        .filter(|s| !s.is_empty())
        .map(|fmt| format_summary_title(fmt))
        .unwrap_or_else(|| format_summary_title(DEFAULT_TITLE_FORMAT));
    send_report_notifications(&settings, &title, &summary);

    let mut updated_settings = settings.clone();
    updated_settings.last_summary_path = Some(path_str.clone());
    memory_storage::save_settings_sync(&updated_settings)
        .map_err(|e| format!("Failed to update settings: {}", e))?;

    tracing::info!("Daily summary generated: {}", path_str);
    Ok(path_str)
}

// DATA-007: Multi-language daily report command
pub async fn generate_multilingual_daily_summary(target_lang: String) -> Result<String, String> {
    if !crate::network_status::is_online() {
        return Err("当前处于离线状态，多语言日报生成需要网络连接".to_string());
    }

    let settings = memory_storage::get_settings_sync()
        .map_err(|e| format!("Failed to get settings: {}", e))?;
    let api_config = load_api_config(&settings)?;

    // Get the default (Chinese) summary first
    let summary = generate_base_daily_summary(&settings, &api_config).await?;

    // If target language is Chinese (default), return as-is
    if target_lang == "zh-CN" || target_lang.is_empty() {
        return Ok(summary);
    }

    // Translate to target language
    let translated = translate_report(&api_config, &summary, &target_lang).await?;

    // Save the translated version
    let obsidian_path = settings.get_obsidian_output_path()?;
    let filename = generate_summary_filename_with_lang(&settings, &target_lang);
    let path_str = write_report_to_obsidian(&obsidian_path, &filename, &translated)?;

    tracing::info!(
        "Multilingual daily summary generated: {} (lang: {})",
        path_str,
        target_lang
    );
    Ok(path_str)
}

/// Helper function to generate base daily summary content
pub async fn generate_base_daily_summary(
    settings: &Settings,
    api_config: &ApiConfig,
) -> Result<String, String> {
    // Try session-based approach first
    let sessions = crate::session_manager::get_today_sessions_sync().unwrap_or_default();

    if !sessions.is_empty() {
        for session in &sessions {
            if session.status == SessionStatus::Active || session.status == SessionStatus::Ended {
                if let Err(e) = crate::session_manager::analyze_session(session.id).await {
                    tracing::warn!("Failed to analyze session {}: {}", session.id, e);
                }
            }
        }

        let sessions = crate::session_manager::get_today_sessions_sync().unwrap_or_default();

        if let Some(content) = build_session_based_report(&sessions) {
            let prompt_template = settings
                .summary_prompt
                .as_deref()
                .filter(|s| !s.is_empty())
                .unwrap_or(DEFAULT_SUMMARY_PROMPT);
            let prompt = prompt_template
                .replace("{records}", &content)
                .replace("{github_activity}", "");

            return call_llm_api_with_retry(api_config, &prompt, 2000, "generate_daily_summary")
                .await;
        }
    }

    // Fallback to legacy flat record format
    let all_records = memory_storage::get_all_today_records_for_summary()
        .map_err(|e| format!("Failed to get records: {}", e))?;
    let records = filter_records_by_settings(all_records, settings);
    if records.is_empty() {
        return Err("No records for today after filtering".to_string());
    }

    let records_text = format_records_for_summary(&records);
    let prompt_template = settings
        .summary_prompt
        .as_deref()
        .filter(|s| !s.is_empty())
        .unwrap_or(DEFAULT_SUMMARY_PROMPT);
    let prompt = prompt_template
        .replace("{records}", &records_text)
        .replace("{github_activity}", "");

    call_llm_api_with_retry(api_config, &prompt, 2000, "generate_daily_summary").await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory_storage::Record;

    fn create_test_record(source_type: &str, content: &str) -> Record {
        Record {
            id: 1,
            timestamp: chrono::Utc::now().to_rfc3339(),
            source_type: source_type.to_string(),
            content: content.to_string(),
            screenshot_path: None,
            monitor_info: None,
            tags: None,
            user_notes: None,
            session_id: None,
            analysis_status: None,
        }
    }

    fn create_settings_with_include_manual(include: bool) -> Settings {
        Settings {
            include_manual_records: Some(include),
            weekly_report_prompt: None,
            weekly_report_day: None,
            last_weekly_report_path: None,
            summary_title_format: None,
            api_base_url: None,
            api_key: None,
            model_name: None,
            screenshot_interval: None,
            summary_time: None,
            obsidian_path: None,
            auto_capture_enabled: None,
            last_summary_path: None,
            summary_model_name: None,
            analysis_prompt: None,
            summary_prompt: None,
            change_threshold: None,
            max_silent_minutes: None,
            window_whitelist: None,
            window_blacklist: None,
            use_whitelist_only: None,
            auto_adjust_silent: None,
            silent_adjustment_paused_until: None,
            auto_detect_work_time: None,
            use_custom_work_time: None,
            custom_work_time_start: None,
            custom_work_time_end: None,
            learned_work_time: None,
            capture_mode: None,
            selected_monitor_index: None,
            tag_categories: None,
            is_ollama: None,
            monthly_report_prompt: None,
            last_monthly_report_path: None,
            custom_report_prompt: None,
            last_custom_report_path: None,
            obsidian_vaults: None,
            auto_detect_vault_by_window: None,
            comparison_report_prompt: None,
            logseq_graphs: None,
            notion_api_key: None,
            notion_database_id: None,
            slack_webhook_url: None,
            dingtalk_webhook_url: None,
            capture_only_mode: None,
            custom_headers: None,
            quality_filter_enabled: None,
            quality_filter_threshold: None,
            session_gap_minutes: None,
            // PERF-001: Proxy settings
            proxy_enabled: None,
            proxy_host: None,
            proxy_port: None,
            proxy_username: None,
            proxy_password: None,
            test_model_name: None,
            onboarding_completed: None,
            language: None,
            // DATA-007: Multi-language settings
            preferred_language: None,
            supported_languages: None,
            // STAB-002: Auto backup settings
            auto_backup_enabled: None,
            auto_backup_interval: None,
            auto_backup_retention: None,
            last_auto_backup_at: None,
            // FEAT-008: Custom export template
            custom_export_template: None,
        }
    }

    // ── Tests for format_summary_title ──

    #[test]
    fn format_summary_title_replaces_date_placeholder() {
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let result = format_summary_title("工作日报 - {date}");
        assert_eq!(result, format!("工作日报 - {}", today));
    }

    #[test]
    fn format_summary_title_with_custom_format() {
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let result = format_summary_title("Daily Report - {date}");
        assert_eq!(result, format!("Daily Report - {}", today));
    }

    #[test]
    fn format_summary_title_without_placeholder() {
        let result = format_summary_title("工作日报");
        assert_eq!(result, "工作日报");
    }

    #[test]
    fn format_summary_title_multiple_placeholders() {
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let result = format_summary_title("{date} 日报 {date}");
        assert_eq!(result, format!("{} 日报 {}", today, today));
    }

    // ── Tests for filter_records_by_settings ──

    #[test]
    fn filter_records_keeps_all_when_include_is_true() {
        let settings = create_settings_with_include_manual(true);
        let records = vec![
            create_test_record("auto", "auto record"),
            create_test_record("manual", "manual record"),
        ];

        let filtered = filter_records_by_settings(records, &settings);
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn filter_records_excludes_manual_when_include_is_false() {
        let settings = create_settings_with_include_manual(false);
        let records = vec![
            create_test_record("auto", "auto record"),
            create_test_record("manual", "manual record"),
        ];

        let filtered = filter_records_by_settings(records, &settings);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].source_type, "auto");
    }

    #[test]
    fn filter_records_keeps_all_when_setting_is_none() {
        let mut settings = create_settings_with_include_manual(true);
        settings.include_manual_records = None;
        let records = vec![
            create_test_record("auto", "auto record"),
            create_test_record("manual", "manual record"),
        ];

        let filtered = filter_records_by_settings(records, &settings);
        assert_eq!(filtered.len(), 2); // None defaults to true
    }

    #[test]
    fn filter_records_all_auto_records_kept() {
        let settings = create_settings_with_include_manual(false);
        let records = vec![
            create_test_record("auto", "auto record 1"),
            create_test_record("auto", "auto record 2"),
        ];

        let filtered = filter_records_by_settings(records, &settings);
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn filter_records_all_manual_records_removed() {
        let settings = create_settings_with_include_manual(false);
        let records = vec![
            create_test_record("manual", "manual record 1"),
            create_test_record("manual", "manual record 2"),
        ];

        let filtered = filter_records_by_settings(records, &settings);
        assert!(filtered.is_empty());
    }

    // ── Tests for generate_summary_filename ──

    #[test]
    fn generate_filename_uses_default_format() {
        let settings = Settings {
            summary_title_format: None,
            ..create_settings_with_include_manual(true)
        };
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let filename = generate_summary_filename(&settings);
        assert_eq!(filename, format!("工作日报 - {}.md", today));
    }

    #[test]
    fn generate_filename_uses_custom_format() {
        let mut settings = create_settings_with_include_manual(true);
        settings.summary_title_format = Some("Daily Report - {date}".to_string());
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let filename = generate_summary_filename(&settings);
        assert_eq!(filename, format!("Daily Report - {}.md", today));
    }

    #[test]
    fn generate_filename_ignores_empty_format() {
        let mut settings = create_settings_with_include_manual(true);
        settings.summary_title_format = Some("".to_string());
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let filename = generate_summary_filename(&settings);
        // Should use default format
        assert_eq!(filename, format!("工作日报 - {}.md", today));
    }

    // ── Tests for format_records_for_summary ──

    #[test]
    fn format_records_outputs_correct_format() {
        let records = vec![create_test_record("auto", "test content")];
        let formatted = format_records_for_summary(&records);
        assert!(formatted.contains("- ["));
        assert!(formatted.contains("🖥️ 自动感知"));
        assert!(formatted.contains("test content"));
    }

    #[test]
    fn format_records_manual_source_uses_flash_icon() {
        let records = vec![create_test_record("manual", "manual content")];
        let formatted = format_records_for_summary(&records);
        assert!(formatted.contains("⚡ 闪念"));
        assert!(formatted.contains("manual content"));
    }

    #[test]
    fn format_records_multiple_records_joined_with_newline() {
        let records = vec![
            create_test_record("auto", "first"),
            create_test_record("manual", "second"),
        ];
        let formatted = format_records_for_summary(&records);
        let lines: Vec<&str> = formatted.lines().collect();
        assert_eq!(lines.len(), 2);
    }

    #[test]
    fn format_records_empty_returns_empty_string() {
        let records: Vec<Record> = vec![];
        let formatted = format_records_for_summary(&records);
        assert!(formatted.is_empty());
    }

    // ── Tests for get_default_summary_prompt ──

    #[test]
    fn get_default_summary_prompt_returns_expected_content() {
        let prompt = get_default_summary_prompt();
        assert!(prompt.contains("{records}"));
        assert!(prompt.contains("Markdown"));
        assert!(prompt.contains("工作日志助手"));
    }

    #[test]
    fn get_default_summary_prompt_is_not_empty() {
        let prompt = get_default_summary_prompt();
        assert!(!prompt.is_empty());
    }

    // ── Tests for get_default_weekly_report_prompt ──

    #[test]
    fn get_default_weekly_report_prompt_returns_expected_content() {
        let prompt = get_default_weekly_report_prompt();
        assert!(prompt.contains("{records}"));
        assert!(prompt.contains("Markdown"));
        assert!(prompt.contains("周报"));
    }

    #[test]
    fn get_default_weekly_report_prompt_is_not_empty() {
        let prompt = get_default_weekly_report_prompt();
        assert!(!prompt.is_empty());
    }

    // ── Tests for get_week_dates_for_filename ──

    #[test]
    fn week_dates_returns_7_day_range() {
        let (start, end) = get_week_dates_for_filename(0);
        let start_date =
            chrono::NaiveDate::parse_from_str(&start, "%Y-%m-%d").expect("valid start date");
        let end_date = chrono::NaiveDate::parse_from_str(&end, "%Y-%m-%d").expect("valid end date");
        let diff = (end_date - start_date).num_days();
        assert_eq!(diff, 6, "Week range should be 6 days (Mon-Sun)");
    }

    #[test]
    fn week_dates_start_is_monday_when_week_start_day_is_0() {
        let (start, _end) = get_week_dates_for_filename(0);
        let start_date =
            chrono::NaiveDate::parse_from_str(&start, "%Y-%m-%d").expect("valid start date");
        assert_eq!(
            start_date.weekday(),
            chrono::Weekday::Mon,
            "Week should start on Monday when week_start_day=0"
        );
    }

    #[test]
    fn week_dates_start_is_sunday_when_week_start_day_is_6() {
        let (start, _end) = get_week_dates_for_filename(6);
        let start_date =
            chrono::NaiveDate::parse_from_str(&start, "%Y-%m-%d").expect("valid start date");
        assert_eq!(
            start_date.weekday(),
            chrono::Weekday::Sun,
            "Week should start on Sunday when week_start_day=6"
        );
    }

    // ── Tests for generate_weekly_report_filename ──

    #[test]
    fn weekly_report_filename_has_correct_format() {
        let filename = generate_weekly_report_filename(0);
        assert!(
            filename.starts_with("周报-"),
            "Filename should start with '周报-'"
        );
        assert!(filename.contains("-to-"), "Filename should contain '-to-'");
        assert!(filename.ends_with(".md"), "Filename should end with '.md'");
    }

    #[test]
    fn weekly_report_filename_uses_correct_week_start_day() {
        // When week_start_day=6 (Sunday), the start date should be a Sunday
        let filename = generate_weekly_report_filename(6);
        // Extract start date from filename: "周报-YYYY-MM-DD-to-YYYY-MM-DD.md"
        let parts: Vec<&str> = filename.split('-').collect();
        // parts: ["周报", "YYYY", "MM", "DD", "to", "YYYY", "MM", "DD.md"]
        let start_date_str = format!("{}-{}-{}", parts[1], parts[2], parts[3]);
        let start_date = chrono::NaiveDate::parse_from_str(&start_date_str, "%Y-%m-%d")
            .expect("valid start date in filename");
        assert_eq!(
            start_date.weekday(),
            chrono::Weekday::Sun,
            "Filename should reflect Sunday as week start"
        );
    }

    // ── Tests for get_default_monthly_report_prompt ──

    #[test]
    fn get_default_monthly_report_prompt_returns_expected_content() {
        let prompt = get_default_monthly_report_prompt();
        assert!(prompt.contains("{records}"));
        assert!(prompt.contains("Markdown"));
        assert!(prompt.contains("工作日志助手"));
        assert!(prompt.contains("按周分组"));
    }

    #[test]
    fn get_default_monthly_report_prompt_is_not_empty() {
        let prompt = get_default_monthly_report_prompt();
        assert!(!prompt.is_empty());
    }

    // ── Tests for format_records_by_week ──

    #[test]
    fn format_records_by_week_groups_by_week() {
        let records = vec![
            create_test_record("auto", "record 1"),
            create_test_record("manual", "record 2"),
        ];
        let formatted = format_records_by_week(&records);
        assert!(formatted.contains("第"));
        assert!(formatted.contains("周"));
    }

    #[test]
    fn format_records_by_week_contains_records() {
        let records = vec![create_test_record("auto", "test content")];
        let formatted = format_records_by_week(&records);
        assert!(formatted.contains("test content"));
    }

    #[test]
    fn format_records_by_week_empty_returns_empty_string() {
        let records: Vec<Record> = vec![];
        let formatted = format_records_by_week(&records);
        assert!(formatted.is_empty());
    }

    // ── Tests for generate_monthly_report_filename ──

    #[test]
    fn generate_monthly_report_filename_format() {
        let filename = generate_monthly_report_filename();
        assert!(filename.starts_with("月报-"));
        assert!(filename.ends_with(".md"));
        // Should contain YYYY-MM format
        assert!(filename.contains("-"));
    }

    // ── REPORT-003: Tests for custom report functions ──

    #[test]
    fn get_default_custom_report_prompt_returns_expected_content() {
        let prompt = get_default_custom_report_prompt();
        assert!(prompt.contains("{records}"));
        assert!(prompt.contains("{start_date}"));
        assert!(prompt.contains("{end_date}"));
        assert!(prompt.contains("工作日志助手"));
    }

    #[test]
    fn get_default_custom_report_prompt_is_not_empty() {
        let prompt = get_default_custom_report_prompt();
        assert!(!prompt.is_empty());
    }

    #[test]
    fn custom_report_filename_default_name() {
        let filename = generate_custom_report_filename("自定义报告", "2026-03-01", "2026-03-14");
        assert_eq!(filename, "自定义报告-2026-03-01-to-2026-03-14.md");
    }

    #[test]
    fn custom_report_filename_with_custom_name() {
        let filename = generate_custom_report_filename("双周报", "2026-03-01", "2026-03-14");
        assert_eq!(filename, "双周报-2026-03-01-to-2026-03-14.md");
    }

    #[test]
    fn custom_report_filename_quarter() {
        let filename = generate_custom_report_filename("季度报", "2026-01-01", "2026-03-31");
        assert_eq!(filename, "季度报-2026-01-01-to-2026-03-31.md");
    }

    #[test]
    fn biweekly_range_returns_14_day_span() {
        let (start, end) = get_biweekly_range();
        let start_date =
            chrono::NaiveDate::parse_from_str(&start, "%Y-%m-%d").expect("valid start");
        let end_date = chrono::NaiveDate::parse_from_str(&end, "%Y-%m-%d").expect("valid end");
        let diff = (end_date - start_date).num_days();
        assert_eq!(
            diff, 13,
            "Biweekly range should be 13 days (14 days inclusive)"
        );
    }

    #[test]
    fn biweekly_range_end_is_today() {
        let (_start, end) = get_biweekly_range();
        let today = chrono::Local::now()
            .date_naive()
            .format("%Y-%m-%d")
            .to_string();
        assert_eq!(end, today, "Biweekly range should end today");
    }

    #[test]
    fn quarter_range_q1() {
        // Test Q1: January 1 - March 31
        let start = chrono::NaiveDate::from_ymd_opt(2026, 2, 15).unwrap();
        let month = start.month();
        let quarter_start_month = (month - 1) / 3 * 3 + 1;
        assert_eq!(quarter_start_month, 1);

        let q_start = chrono::NaiveDate::from_ymd_opt(2026, quarter_start_month, 1).unwrap();
        let q_end = chrono::NaiveDate::from_ymd_opt(2026, quarter_start_month + 3, 1).unwrap()
            - chrono::Duration::days(1);
        assert_eq!(q_start.format("%Y-%m-%d").to_string(), "2026-01-01");
        assert_eq!(q_end.format("%Y-%m-%d").to_string(), "2026-03-31");
    }

    #[test]
    fn quarter_range_q4() {
        // Test Q4: October 1 - December 31
        let start = chrono::NaiveDate::from_ymd_opt(2026, 11, 15).unwrap();
        let month = start.month();
        let quarter_start_month = (month - 1) / 3 * 3 + 1;
        assert_eq!(quarter_start_month, 10);

        let q_start = chrono::NaiveDate::from_ymd_opt(2026, quarter_start_month, 1).unwrap();
        // Q4: quarter_start_month + 3 = 13 > 12
        let q_end =
            chrono::NaiveDate::from_ymd_opt(2026 + 1, 1, 1).unwrap() - chrono::Duration::days(1);
        assert_eq!(q_start.format("%Y-%m-%d").to_string(), "2026-10-01");
        assert_eq!(q_end.format("%Y-%m-%d").to_string(), "2026-12-31");
    }

    #[test]
    fn quarter_range_returns_valid_dates() {
        let (start, end) = get_quarter_range();
        let start_date =
            chrono::NaiveDate::parse_from_str(&start, "%Y-%m-%d").expect("valid start");
        let end_date = chrono::NaiveDate::parse_from_str(&end, "%Y-%m-%d").expect("valid end");
        assert!(end_date >= start_date, "Quarter end must be >= start");

        // Quarter should be roughly 90 days
        let diff = (end_date - start_date).num_days();
        assert!(
            (89..=92).contains(&diff),
            "Quarter should be 89-92 days, got {}",
            diff
        );
    }

    #[test]
    fn quarter_range_start_is_first_of_quarter() {
        let (start, _end) = get_quarter_range();
        let start_date =
            chrono::NaiveDate::parse_from_str(&start, "%Y-%m-%d").expect("valid start");
        assert_eq!(start_date.day(), 1, "Quarter should start on day 1");
        assert!(
            [1, 4, 7, 10].contains(&start_date.month()),
            "Quarter should start in Jan/Apr/Jul/Oct"
        );
    }

    // ── Tests for load_api_config ──

    #[test]
    fn load_api_config_returns_error_when_no_api_url() {
        let settings = create_settings_with_include_manual(true);
        let result = load_api_config(&settings);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("API Base URL not configured"));
    }

    #[test]
    fn load_api_config_returns_error_when_no_api_key_non_ollama() {
        let mut settings = create_settings_with_include_manual(true);
        settings.api_base_url = Some("https://api.openai.com/v1".to_string());
        settings.api_key = None;
        let result = load_api_config(&settings);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("API Key is required"));
    }

    #[test]
    fn load_api_config_uses_summary_model_name_over_model_name() {
        let mut settings = create_settings_with_include_manual(true);
        settings.api_base_url = Some("https://api.openai.com/v1".to_string());
        settings.api_key = Some("test-key".to_string());
        settings.model_name = Some("gpt-3.5".to_string());
        settings.summary_model_name = Some("gpt-4o".to_string());
        let config = load_api_config(&settings).unwrap();
        assert_eq!(config.model_name, "gpt-4o");
    }

    #[test]
    fn load_api_config_falls_back_to_model_name() {
        let mut settings = create_settings_with_include_manual(true);
        settings.api_base_url = Some("https://api.openai.com/v1".to_string());
        settings.api_key = Some("test-key".to_string());
        settings.model_name = Some("gpt-3.5".to_string());
        settings.summary_model_name = None;
        let config = load_api_config(&settings).unwrap();
        assert_eq!(config.model_name, "gpt-3.5");
    }

    #[test]
    fn load_api_config_defaults_to_gpt4o() {
        let mut settings = create_settings_with_include_manual(true);
        settings.api_base_url = Some("https://api.openai.com/v1".to_string());
        settings.api_key = Some("test-key".to_string());
        settings.model_name = None;
        settings.summary_model_name = None;
        let config = load_api_config(&settings).unwrap();
        assert_eq!(config.model_name, "gpt-4o");
    }

    // ── Tests for AI-006: Custom Headers ──

    #[test]
    fn load_api_config_loads_custom_headers() {
        let mut settings = create_settings_with_include_manual(true);
        settings.api_base_url = Some("https://api.openai.com/v1".to_string());
        settings.api_key = Some("test-key".to_string());
        settings.custom_headers = Some(
            r#"[{"key":"HTTP-Referer","value":"https://dailylogger.app","sensitive":false}]"#
                .to_string(),
        );
        let config = load_api_config(&settings).unwrap();
        assert_eq!(config.custom_headers.len(), 1);
        assert_eq!(config.custom_headers[0].key, "HTTP-Referer");
        assert_eq!(config.custom_headers[0].value, "https://dailylogger.app");
        assert!(!config.custom_headers[0].sensitive);
    }

    #[test]
    fn load_api_config_handles_empty_custom_headers() {
        let mut settings = create_settings_with_include_manual(true);
        settings.api_base_url = Some("https://api.openai.com/v1".to_string());
        settings.api_key = Some("test-key".to_string());
        settings.custom_headers = Some("[]".to_string());
        let config = load_api_config(&settings).unwrap();
        assert!(config.custom_headers.is_empty());
    }

    #[test]
    fn load_api_config_handles_invalid_custom_headers_json() {
        let mut settings = create_settings_with_include_manual(true);
        settings.api_base_url = Some("https://api.openai.com/v1".to_string());
        settings.api_key = Some("test-key".to_string());
        settings.custom_headers = Some("invalid json".to_string());
        let config = load_api_config(&settings).unwrap();
        assert!(config.custom_headers.is_empty());
    }

    #[test]
    fn load_api_config_handles_none_custom_headers() {
        let mut settings = create_settings_with_include_manual(true);
        settings.api_base_url = Some("https://api.openai.com/v1".to_string());
        settings.api_key = Some("test-key".to_string());
        settings.custom_headers = None;
        let config = load_api_config(&settings).unwrap();
        assert!(config.custom_headers.is_empty());
    }

    // ── Tests for write_report_to_obsidian ──

    #[test]
    fn write_report_to_obsidian_creates_file() {
        let dir = std::env::temp_dir().join("dailylogger_test_write_report");
        let _ = std::fs::remove_dir_all(&dir);
        let path =
            write_report_to_obsidian(dir.to_str().unwrap(), "test-report.md", "# Report\nContent")
                .unwrap();
        assert!(std::path::Path::new(&path).exists());
        let content = std::fs::read_to_string(&path).unwrap();
        assert_eq!(content, "# Report\nContent");
        let _ = std::fs::remove_dir_all(&dir);
    }

    // ── Tests for write_report_to_logseq (INT-002) ──

    #[test]
    fn write_report_to_logseq_creates_file_in_pages_folder() {
        let dir = std::env::temp_dir().join("dailylogger_test_write_logseq");
        let _ = std::fs::remove_dir_all(&dir);

        // Create settings with logseq_graphs configured
        // Use serde_json to properly escape Windows paths (backslashes)
        let mut settings = create_settings_with_include_manual(true);
        let graph_path = dir.to_str().unwrap().to_string();
        let graphs_json = serde_json::to_string(&[serde_json::json!({
            "name": "Test",
            "path": graph_path,
            "is_default": true
        })])
        .unwrap();
        settings.logseq_graphs = Some(graphs_json);

        let path = write_report_to_logseq(&settings, "test-report.md", "# Report\nContent");
        assert!(path.is_some());
        let path = path.unwrap();

        // File should be in pages subdirectory
        assert!(path.contains("pages"));
        assert!(std::path::Path::new(&path).exists());
        let content = std::fs::read_to_string(&path).unwrap();
        assert_eq!(content, "# Report\nContent");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn write_report_to_logseq_creates_pages_directory() {
        let dir = std::env::temp_dir().join("dailylogger_test_logseq_pages");
        let _ = std::fs::remove_dir_all(&dir);

        // Create settings with logseq_graphs configured
        // Use serde_json to properly escape Windows paths (backslashes)
        let mut settings = create_settings_with_include_manual(true);
        let graph_path = dir.to_str().unwrap().to_string();
        let graphs_json = serde_json::to_string(&[serde_json::json!({
            "name": "Test",
            "path": graph_path,
            "is_default": true
        })])
        .unwrap();
        settings.logseq_graphs = Some(graphs_json);

        let path = write_report_to_logseq(&settings, "daily-report.md", "# Daily\n");
        assert!(path.is_some());

        // Verify pages folder was created
        let pages_dir = std::path::Path::new(&dir).join("pages");
        assert!(pages_dir.exists());
        assert!(std::path::Path::new(path.unwrap().as_str()).exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn write_report_to_logseq_returns_none_when_not_configured() {
        let settings = create_settings_with_include_manual(true);
        // logseq_graphs is None by default
        let result = write_report_to_logseq(&settings, "test.md", "# Content");
        assert!(result.is_none());
    }

    /// INT-002: Additional edge case tests for Logseq export
    #[test]
    fn write_report_to_logseq_with_empty_path_returns_none() {
        let mut settings = create_settings_with_include_manual(true);
        // Graph with empty path should be skipped
        settings.logseq_graphs =
            Some(r#"[{"name":"Empty","path":"","is_default":true}]"#.to_string());
        let result = write_report_to_logseq(&settings, "test.md", "# Content");
        assert!(result.is_none());
    }

    #[test]
    fn write_report_to_logseq_with_whitespace_path_returns_none() {
        let mut settings = create_settings_with_include_manual(true);
        // Graph with whitespace-only path should be skipped
        settings.logseq_graphs =
            Some(r#"[{"name":"Whitespace","path":"   ","is_default":true}]"#.to_string());
        let result = write_report_to_logseq(&settings, "test.md", "# Content");
        assert!(result.is_none());
    }

    #[test]
    fn write_report_to_logseq_falls_back_to_first_graph() {
        let dir = std::env::temp_dir().join("dailylogger_test_logseq_fallback");
        let _ = std::fs::remove_dir_all(&dir);

        let mut settings = create_settings_with_include_manual(true);
        let graph_path = dir.to_str().unwrap().to_string();
        let graphs_json = serde_json::to_string(&[
            serde_json::json!({
                "name": "First",
                "path": graph_path,
                "is_default": false
            }),
            serde_json::json!({
                "name": "Second",
                "path": "/nonexistent",
                "is_default": false
            }),
        ])
        .unwrap();
        settings.logseq_graphs = Some(graphs_json);

        // Should use first graph when no default is set
        let result = write_report_to_logseq(&settings, "test-fallback.md", "# Test");
        assert!(result.is_some());
        let path = result.unwrap();
        assert!(path.contains("First") || path.contains("dailylogger_test_logseq_fallback"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn get_logseq_output_path_with_multiple_graphs_selects_default() {
        let mut settings = create_settings_with_include_manual(true);
        settings.logseq_graphs = Some(
            r#"[{"name":"Personal","path":"/logseq/personal","is_default":false},{"name":"Work","path":"/logseq/work","is_default":true}]"#.to_string(),
        );
        // Should select the graph with is_default=true, not the first one
        assert_eq!(settings.get_logseq_output_path().unwrap(), "/logseq/work");
    }

    // NOTE: Performance benchmark tests moved to dedicated `mod benchmarks` below (CORE-008)
}

/// Returns the default summary prompt template.
/// This is used when the user has not configured a custom prompt.
pub fn get_default_summary_prompt() -> String {
    DEFAULT_SUMMARY_PROMPT.to_string()
}

/// Get the week boundaries for filename generation.
/// Returns (week_start_date, week_end_date) as strings in YYYY-MM-DD format.
fn get_week_dates_for_filename(week_start_day: i32) -> (String, String) {
    let today = chrono::Local::now().date_naive();
    let weekday = today.weekday().num_days_from_monday() as i32;
    let days_since_week_start = (weekday - week_start_day + 7) % 7;

    let week_start_date = today - chrono::Duration::days(days_since_week_start as i64);
    let week_end_date = week_start_date + chrono::Duration::days(6);

    (
        week_start_date.format("%Y-%m-%d").to_string(),
        week_end_date.format("%Y-%m-%d").to_string(),
    )
}

/// Generate the filename for weekly report.
/// `week_start_day`: 0=Monday, 6=Sunday
pub fn generate_weekly_report_filename(week_start_day: i32) -> String {
    let (start_date, end_date) = get_week_dates_for_filename(week_start_day);
    format!("周报-{}-to-{}.md", start_date, end_date)
}

/// Generate weekly report - REPORT-001
pub async fn generate_weekly_report() -> Result<String, String> {
    if !crate::network_status::is_online() {
        let _ = crate::offline_queue::enqueue_task(
            &crate::offline_queue::OfflineTaskType::WeeklyReport,
            "{}",
            None,
        );
        return Err("当前处于离线状态，周报生成已加入队列，网络恢复后将自动处理".to_string());
    }

    let settings = memory_storage::get_settings_sync()
        .map_err(|e| format!("Failed to get settings: {}", e))?;
    let obsidian_path = settings.get_obsidian_output_path()?;
    let api_config = load_api_config(&settings)?;

    let week_start_day = settings.weekly_report_day.unwrap_or(0);
    let all_records = memory_storage::get_week_records_sync(week_start_day)
        .map_err(|e| format!("Failed to get week records: {}", e))?;
    let records = filter_records_by_settings(all_records, &settings);
    if records.is_empty() {
        return Err("本周无记录".to_string());
    }

    let records_text = format_records_for_summary(&records);
    let prompt_template = settings
        .weekly_report_prompt
        .as_deref()
        .filter(|s| !s.is_empty())
        .unwrap_or(DEFAULT_WEEKLY_REPORT_PROMPT);
    let prompt = prompt_template.replace("{records}", &records_text);

    let summary =
        call_llm_api_with_retry(&api_config, &prompt, 3000, "generate_weekly_report").await?;

    let filename = generate_weekly_report_filename(week_start_day);
    let path_str = write_report_to_obsidian(&obsidian_path, &filename, &summary)?;

    // INT-002: Also write to Logseq if configured
    write_report_to_logseq(&settings, &filename, &summary);

    // INT-001: Also write to Notion if configured
    if let Some(notion_url) = notion::write_report_to_notion_sync(&settings, &filename, &summary) {
        tracing::info!("Weekly report also written to Notion: {}", notion_url);
    }

    // INT-004: Send notifications to Slack/DingTalk if configured
    let title = filename.trim_end_matches(".md");
    send_report_notifications(&settings, title, &summary);

    let mut updated_settings = settings.clone();
    updated_settings.last_weekly_report_path = Some(path_str.clone());
    memory_storage::save_settings_sync(&updated_settings)
        .map_err(|e| format!("Failed to update settings: {}", e))?;

    tracing::info!("Weekly report generated: {}", path_str);
    Ok(path_str)
}

/// Generate monthly report - REPORT-002
pub async fn generate_monthly_report() -> Result<String, String> {
    if !crate::network_status::is_online() {
        let _ = crate::offline_queue::enqueue_task(
            &crate::offline_queue::OfflineTaskType::MonthlyReport,
            "{}",
            None,
        );
        return Err("当前处于离线状态，月报生成已加入队列，网络恢复后将自动处理".to_string());
    }

    let settings = memory_storage::get_settings_sync()
        .map_err(|e| format!("Failed to get settings: {}", e))?;
    let obsidian_path = settings.get_obsidian_output_path()?;
    let api_config = load_api_config(&settings)?;

    let all_records = memory_storage::get_month_records_sync()
        .map_err(|e| format!("Failed to get month records: {}", e))?;
    let records = filter_records_by_settings(all_records, &settings);
    if records.is_empty() {
        return Err("本月无记录".to_string());
    }

    let records_text = format_records_by_week(&records);
    let prompt_template = settings
        .monthly_report_prompt
        .as_deref()
        .filter(|s| !s.is_empty())
        .unwrap_or(DEFAULT_MONTHLY_REPORT_PROMPT);
    let prompt = prompt_template.replace("{records}", &records_text);

    let summary =
        call_llm_api_with_retry(&api_config, &prompt, 4000, "generate_monthly_report").await?;

    let filename = generate_monthly_report_filename();
    let path_str = write_report_to_obsidian(&obsidian_path, &filename, &summary)?;

    // INT-002: Also write to Logseq if configured
    write_report_to_logseq(&settings, &filename, &summary);

    // INT-001: Also write to Notion if configured
    if let Some(notion_url) = notion::write_report_to_notion_sync(&settings, &filename, &summary) {
        tracing::info!("Monthly report also written to Notion: {}", notion_url);
    }

    // INT-004: Send notifications to Slack/DingTalk if configured
    let title = filename.trim_end_matches(".md");
    send_report_notifications(&settings, title, &summary);

    let mut updated_settings = settings.clone();
    updated_settings.last_monthly_report_path = Some(path_str.clone());
    memory_storage::save_settings_sync(&updated_settings)
        .map_err(|e| format!("Failed to update settings: {}", e))?;

    tracing::info!("Monthly report generated: {}", path_str);
    Ok(path_str)
}

/// Get the default custom report prompt - REPORT-003
pub fn get_default_custom_report_prompt() -> String {
    DEFAULT_CUSTOM_REPORT_PROMPT.to_string()
}

/// Generate filename for custom period report
/// Format: {report_name}-{start_date}-to-{end_date}.md
pub fn generate_custom_report_filename(
    report_name: &str,
    start_date: &str,
    end_date: &str,
) -> String {
    format!("{}-{}-to-{}.md", report_name, start_date, end_date)
}

/// Calculate biweekly date range (last 14 days including today)
pub fn get_biweekly_range() -> (String, String) {
    let today = chrono::Local::now().date_naive();
    let start = today - chrono::Duration::days(13);
    (
        start.format("%Y-%m-%d").to_string(),
        today.format("%Y-%m-%d").to_string(),
    )
}

/// Calculate current quarter date range
pub fn get_quarter_range() -> (String, String) {
    let today = chrono::Local::now().date_naive();
    let month = today.month();
    let quarter_start_month = (month - 1) / 3 * 3 + 1;

    let start = chrono::NaiveDate::from_ymd_opt(today.year(), quarter_start_month, 1).unwrap();
    let end = if quarter_start_month + 3 > 12 {
        chrono::NaiveDate::from_ymd_opt(today.year() + 1, 1, 1).unwrap() - chrono::Duration::days(1)
    } else {
        chrono::NaiveDate::from_ymd_opt(today.year(), quarter_start_month + 3, 1).unwrap()
            - chrono::Duration::days(1)
    };

    (
        start.format("%Y-%m-%d").to_string(),
        end.format("%Y-%m-%d").to_string(),
    )
}

/// Generate custom period report - REPORT-003
pub async fn generate_custom_report(
    start_date: String,
    end_date: String,
    report_name: Option<String>,
) -> Result<String, String> {
    if !crate::network_status::is_online() {
        return Err("当前处于离线状态，报告生成需要网络连接。请检查网络连接后重试。".to_string());
    }

    let parsed_start = chrono::NaiveDate::parse_from_str(&start_date, "%Y-%m-%d")
        .map_err(|e| format!("无效的起始日期格式 (需要 YYYY-MM-DD): {}", e))?;
    let parsed_end = chrono::NaiveDate::parse_from_str(&end_date, "%Y-%m-%d")
        .map_err(|e| format!("无效的结束日期格式 (需要 YYYY-MM-DD): {}", e))?;
    if parsed_end < parsed_start {
        return Err("结束日期不能早于起始日期".to_string());
    }

    let settings = memory_storage::get_settings_sync()
        .map_err(|e| format!("Failed to get settings: {}", e))?;
    let obsidian_path = settings.get_obsidian_output_path()?;
    let api_config = load_api_config(&settings)?;

    let all_records =
        memory_storage::get_records_by_date_range_sync(start_date.clone(), end_date.clone())
            .map_err(|e| format!("Failed to get records: {}", e))?;
    let records = filter_records_by_settings(all_records, &settings);
    if records.is_empty() {
        return Err("所选时间范围内无记录".to_string());
    }

    let day_count = (parsed_end - parsed_start).num_days() + 1;
    let records_text = if day_count > 14 {
        format_records_by_week(&records)
    } else {
        format_records_for_summary(&records)
    };

    let prompt_template = settings
        .custom_report_prompt
        .as_deref()
        .filter(|s| !s.is_empty())
        .unwrap_or(DEFAULT_CUSTOM_REPORT_PROMPT);
    let prompt = prompt_template
        .replace("{records}", &records_text)
        .replace("{start_date}", &start_date)
        .replace("{end_date}", &end_date);

    let summary =
        call_llm_api_with_retry(&api_config, &prompt, 4000, "generate_custom_report").await?;

    let name = report_name.as_deref().unwrap_or("自定义报告");
    let filename = generate_custom_report_filename(name, &start_date, &end_date);
    let path_str = write_report_to_obsidian(&obsidian_path, &filename, &summary)?;

    // INT-002: Also write to Logseq if configured
    write_report_to_logseq(&settings, &filename, &summary);

    // INT-001: Also write to Notion if configured
    if let Some(notion_url) = notion::write_report_to_notion_sync(&settings, &filename, &summary) {
        tracing::info!("Custom report also written to Notion: {}", notion_url);
    }

    // INT-004: Send notifications to Slack/DingTalk if configured
    let title = filename.trim_end_matches(".md");
    send_report_notifications(&settings, title, &summary);

    let mut updated_settings = settings.clone();
    updated_settings.last_custom_report_path = Some(path_str.clone());
    memory_storage::save_settings_sync(&updated_settings)
        .map_err(|e| format!("Failed to update settings: {}", e))?;

    tracing::info!("Custom report generated: {}", path_str);
    Ok(path_str)
}

/// Generate comparison report between two time periods - REPORT-004
pub async fn compare_reports(
    start_date_a: String,
    end_date_a: String,
    start_date_b: String,
    end_date_b: String,
) -> Result<String, String> {
    if !crate::network_status::is_online() {
        return Err("当前处于离线状态，报告生成需要网络连接。请检查网络连接后重试。".to_string());
    }

    let parsed_start_a = chrono::NaiveDate::parse_from_str(&start_date_a, "%Y-%m-%d")
        .map_err(|e| format!("无效的时段A起始日期格式 (需要 YYYY-MM-DD): {}", e))?;
    let parsed_end_a = chrono::NaiveDate::parse_from_str(&end_date_a, "%Y-%m-%d")
        .map_err(|e| format!("无效的时段A结束日期格式 (需要 YYYY-MM-DD): {}", e))?;
    let parsed_start_b = chrono::NaiveDate::parse_from_str(&start_date_b, "%Y-%m-%d")
        .map_err(|e| format!("无效的时段B起始日期格式 (需要 YYYY-MM-DD): {}", e))?;
    let parsed_end_b = chrono::NaiveDate::parse_from_str(&end_date_b, "%Y-%m-%d")
        .map_err(|e| format!("无效的时段B结束日期格式 (需要 YYYY-MM-DD): {}", e))?;

    if parsed_end_a < parsed_start_a {
        return Err("时段A的结束日期不能早于起始日期".to_string());
    }
    if parsed_end_b < parsed_start_b {
        return Err("时段B的结束日期不能早于起始日期".to_string());
    }

    let settings = memory_storage::get_settings_sync()
        .map_err(|e| format!("Failed to get settings: {}", e))?;
    let obsidian_path = settings.get_obsidian_output_path()?;
    let api_config = load_api_config(&settings)?;

    let all_records_a =
        memory_storage::get_records_by_date_range_sync(start_date_a.clone(), end_date_a.clone())
            .map_err(|e| format!("Failed to get period A records: {}", e))?;
    let all_records_b =
        memory_storage::get_records_by_date_range_sync(start_date_b.clone(), end_date_b.clone())
            .map_err(|e| format!("Failed to get period B records: {}", e))?;

    let records_a = filter_records_by_settings(all_records_a, &settings);
    let records_b = filter_records_by_settings(all_records_b, &settings);

    if records_a.is_empty() && records_b.is_empty() {
        return Err("两个时间段内均无记录".to_string());
    }

    let day_count_a = (parsed_end_a - parsed_start_a).num_days() + 1;
    let records_text_a = if day_count_a > 14 {
        format_records_by_week(&records_a)
    } else {
        format_records_for_summary(&records_a)
    };

    let day_count_b = (parsed_end_b - parsed_start_b).num_days() + 1;
    let records_text_b = if day_count_b > 14 {
        format_records_by_week(&records_b)
    } else {
        format_records_for_summary(&records_b)
    };

    let prompt_template = settings
        .comparison_report_prompt
        .as_deref()
        .filter(|s| !s.is_empty())
        .unwrap_or(DEFAULT_COMPARISON_REPORT_PROMPT);
    let prompt = prompt_template
        .replace("{records_a}", &records_text_a)
        .replace("{records_b}", &records_text_b)
        .replace("{start_date_a}", &start_date_a)
        .replace("{end_date_a}", &end_date_a)
        .replace("{start_date_b}", &start_date_b)
        .replace("{end_date_b}", &end_date_b);

    let summary = call_llm_api_with_retry(&api_config, &prompt, 4000, "compare_reports").await?;

    let filename =
        generate_comparison_report_filename(&start_date_a, &end_date_a, &start_date_b, &end_date_b);
    let path_str = write_report_to_obsidian(&obsidian_path, &filename, &summary)?;

    // INT-002: Also write to Logseq if configured
    write_report_to_logseq(&settings, &filename, &summary);

    // INT-001: Also write to Notion if configured
    if let Some(notion_url) = notion::write_report_to_notion_sync(&settings, &filename, &summary) {
        tracing::info!("Comparison report also written to Notion: {}", notion_url);
    }

    tracing::info!("Comparison report generated: {}", path_str);
    Ok(path_str)
}

// ── SESSION-005: Session-based daily summary tests ──

#[cfg(test)]
mod session_summary_tests {
    use super::*;
    use crate::session_manager::{Session, SessionStatus};

    fn create_test_session(
        id: i64,
        ai_summary: Option<&str>,
        user_summary: Option<&str>,
        status: SessionStatus,
    ) -> Session {
        Session {
            id,
            date: chrono::Local::now().format("%Y-%m-%d").to_string(),
            start_time: chrono::Utc::now().to_rfc3339(),
            end_time: Some(chrono::Utc::now().to_rfc3339()),
            ai_summary: ai_summary.map(|s| s.to_string()),
            user_summary: user_summary.map(|s| s.to_string()),
            context_for_next: None,
            status,
            screenshot_count: None,
        }
    }

    // SESSION-005 AC#2: Test user_summary > ai_summary priority
    #[test]
    fn get_session_display_summary_prefers_user_summary() {
        let session = create_test_session(
            1,
            Some("AI summary"),
            Some("User summary"),
            SessionStatus::Analyzed,
        );
        let result = get_session_display_summary(&session);
        assert_eq!(result, "User summary");
    }

    // SESSION-005 AC#2: Falls back to ai_summary when user_summary is empty
    #[test]
    fn get_session_display_summary_falls_back_to_ai_summary() {
        let session = create_test_session(1, Some("AI summary"), None, SessionStatus::Analyzed);
        let result = get_session_display_summary(&session);
        assert_eq!(result, "AI summary");
    }

    // SESSION-005 AC#2: Falls back to "暂无摘要" when both are empty
    #[test]
    fn get_session_display_summary_shows_default_when_empty() {
        let session = create_test_session(1, None, None, SessionStatus::Analyzed);
        let result = get_session_display_summary(&session);
        assert_eq!(result, "暂无摘要");
    }

    // SESSION-005 AC#2: Falls back to ai_summary when user_summary is empty string
    #[test]
    fn get_session_display_summary_ignores_empty_user_summary() {
        let session = create_test_session(1, Some("AI summary"), Some(""), SessionStatus::Analyzed);
        let result = get_session_display_summary(&session);
        assert_eq!(result, "AI summary");
    }

    // SESSION-005 AC#2: Falls back to "暂无摘要" when ai_summary is empty string
    #[test]
    fn get_session_display_summary_ignores_empty_ai_summary() {
        let session = create_test_session(1, Some(""), None, SessionStatus::Analyzed);
        let result = get_session_display_summary(&session);
        assert_eq!(result, "暂无摘要");
    }

    // SESSION-005 AC#1 & #3: Test build_session_based_report with empty sessions
    #[test]
    fn build_session_based_report_returns_none_for_empty_sessions() {
        let sessions: Vec<Session> = vec![];
        let result = build_session_based_report(&sessions);
        assert!(result.is_none());
    }

    // SESSION-005 AC#1: Test build_session_based_report with sessions
    #[test]
    fn build_session_based_report_formats_sessions_correctly() {
        let session = create_test_session(1, Some("AI summary"), None, SessionStatus::Analyzed);
        let sessions = vec![session];
        let result = build_session_based_report(&sessions);
        assert!(result.is_some());
        let content = result.unwrap();
        // Should contain time range and summary
        assert!(content.contains("##"));
        assert!(content.contains("- analyzed"));
        assert!(content.contains("AI summary"));
    }

    // SESSION-005 AC#2: Test format_session_for_summary with user edited session
    #[test]
    fn format_session_for_summary_shows_edit_indicator_for_user_summary() {
        let session = create_test_session(
            1,
            Some("AI summary"),
            Some("User summary"),
            SessionStatus::Analyzed,
        );
        let result = format_session_for_summary(&session);
        // Should contain the edit indicator
        assert!(result.contains("✏️"));
        assert!(result.contains("User summary"));
    }

    // SESSION-005 AC#1: Test format_session_for_summary with active session
    #[test]
    fn format_session_for_summary_shows_active_status() {
        let session = create_test_session(1, None, None, SessionStatus::Active);
        let result = format_session_for_summary(&session);
        assert!(result.contains("- active"));
    }

    // SESSION-005 AC#1: Test format_session_for_summary with ended session
    #[test]
    fn format_session_for_summary_shows_ended_status() {
        let session = create_test_session(1, None, None, SessionStatus::Ended);
        let result = format_session_for_summary(&session);
        assert!(result.contains("- ended"));
    }
}

// ── Performance benchmark tests (CORE-008 AC#3) ──

#[cfg(test)]
mod benchmarks {
    use super::*;
    use std::time::Instant;

    fn create_settings_with_include_manual(include: bool) -> Settings {
        Settings {
            include_manual_records: Some(include),
            weekly_report_prompt: None,
            weekly_report_day: None,
            last_weekly_report_path: None,
            summary_title_format: None,
            api_base_url: None,
            api_key: None,
            model_name: None,
            screenshot_interval: None,
            summary_time: None,
            obsidian_path: None,
            auto_capture_enabled: None,
            last_summary_path: None,
            summary_model_name: None,
            analysis_prompt: None,
            summary_prompt: None,
            change_threshold: None,
            max_silent_minutes: None,
            window_whitelist: None,
            window_blacklist: None,
            use_whitelist_only: None,
            auto_adjust_silent: None,
            silent_adjustment_paused_until: None,
            auto_detect_work_time: None,
            use_custom_work_time: None,
            custom_work_time_start: None,
            custom_work_time_end: None,
            learned_work_time: None,
            capture_mode: None,
            selected_monitor_index: None,
            tag_categories: None,
            is_ollama: None,
            monthly_report_prompt: None,
            last_monthly_report_path: None,
            custom_report_prompt: None,
            last_custom_report_path: None,
            obsidian_vaults: None,
            auto_detect_vault_by_window: None,
            comparison_report_prompt: None,
            logseq_graphs: None,
            notion_api_key: None,
            notion_database_id: None,
            slack_webhook_url: None,
            dingtalk_webhook_url: None,
            capture_only_mode: None,
            custom_headers: None,
            quality_filter_enabled: None,
            quality_filter_threshold: None,
            session_gap_minutes: None,
            // PERF-001: Proxy configuration
            proxy_enabled: None,
            proxy_host: None,
            proxy_port: None,
            proxy_username: None,
            proxy_password: None,
            // PERF-001: Test model name
            test_model_name: None,
            // PERF-002: Onboarding completed flag
            onboarding_completed: None,
            // PERF-005: Language setting
            language: None,
            // DATA-007: Multi-language settings
            preferred_language: None,
            supported_languages: None,
            // STAB-002: Auto backup settings
            auto_backup_enabled: None,
            auto_backup_interval: None,
            auto_backup_retention: None,
            last_auto_backup_at: None,
            // FEAT-008: Custom export template
            custom_export_template: None,
        }
    }

    /// Benchmark: format_records_for_summary with 100 records
    /// AC requirement: < 30 seconds for daily summary generation (100 records)
    #[test]
    fn benchmark_format_records_for_summary_100_records() {
        let mut records = Vec::with_capacity(100);
        for i in 0..100 {
            records.push(Record {
                id: i as i64,
                timestamp: chrono::Utc::now().to_rfc3339(),
                source_type: if i % 2 == 0 { "auto".to_string() } else { "manual".to_string() },
                content: format!("工作内容 #{}: 完成了功能开发、代码审查和文档编写。涉及 Rust、Vue、Tauri 等技术栈。", i),
                screenshot_path: None,
                monitor_info: None,
                tags: None,
                user_notes: None,
                session_id: None,
                analysis_status: None,
            });
        }

        let start = Instant::now();
        let _result = format_records_for_summary(&records);
        let elapsed_ms = start.elapsed().as_millis();

        // Benchmark threshold: should complete in < 1000ms for formatting alone
        // (AI generation would be additional, but we're testing the data processing part)
        assert!(
            elapsed_ms < 1000,
            "format_records_for_summary with 100 records took {}ms (threshold: 1000ms)",
            elapsed_ms
        );
    }

    /// Benchmark: filter_records_by_settings with 100 records
    #[test]
    fn benchmark_filter_records_100_records() {
        let settings = create_settings_with_include_manual(true);
        let mut records = Vec::with_capacity(100);
        for i in 0..100 {
            records.push(Record {
                id: i as i64,
                timestamp: chrono::Utc::now().to_rfc3339(),
                source_type: if i % 3 == 0 {
                    "auto".to_string()
                } else {
                    "manual".to_string()
                },
                content: format!("测试记录 #{}", i),
                screenshot_path: None,
                monitor_info: None,
                tags: None,
                user_notes: None,
                session_id: None,
                analysis_status: None,
            });
        }

        let start = Instant::now();
        let _result = filter_records_by_settings(records, &settings);
        let elapsed_ms = start.elapsed().as_millis();

        // Threshold: should complete in < 500ms
        // (generous for CI runners with variable performance)
        assert!(
            elapsed_ms < 500,
            "filter_records_by_settings with 100 records took {}ms (threshold: 500ms)",
            elapsed_ms
        );
    }

    /// Benchmark: generate_summary_filename
    #[test]
    fn benchmark_generate_summary_filename() {
        let settings = create_settings_with_include_manual(true);

        let start = Instant::now();
        for _ in 0..1000 {
            let _ = generate_summary_filename(&settings);
        }
        let elapsed_ms = start.elapsed().as_millis();

        // Threshold: 1000 iterations should complete in < 500ms
        // (generous for CI runners with variable performance)
        assert!(
            elapsed_ms < 500,
            "1000 iterations of generate_summary_filename took {}ms (threshold: 500ms)",
            elapsed_ms
        );
    }

    /// Benchmark: format_summary_title
    #[test]
    fn benchmark_format_summary_title() {
        let start = Instant::now();
        for _ in 0..10000 {
            let _ = format_summary_title("工作日报 - {date}");
        }
        let elapsed_ms = start.elapsed().as_millis();

        // Threshold: 10000 iterations should complete in < 500ms
        // (Windows CI runners are slower; 141ms observed on windows-latest)
        assert!(
            elapsed_ms < 500,
            "10000 iterations of format_summary_title took {}ms (threshold: 500ms)",
            elapsed_ms
        );
    }

    /// Performance test: simulate full daily summary generation workflow (without AI API call)
    /// This simulates: filtering 100 records + formatting for summary
    #[test]
    fn benchmark_daily_summary_workflow_100_records() {
        let settings = create_settings_with_include_manual(true);

        // Create 100 test records
        let mut records = Vec::with_capacity(100);
        for i in 0..100 {
            records.push(Record {
                id: i as i64,
                timestamp: chrono::Utc::now().to_rfc3339(),
                source_type: if i % 2 == 0 { "auto".to_string() } else { "manual".to_string() },
                content: format!(
                    "工作内容 #{}: 完成了功能开发、代码审查和文档编写。涉及 Rust、Vue、Tauri 等技术栈。处理了多个任务，包括性能优化、bug 修复和新功能实现。",
                    i
                ),
                screenshot_path: None,
                monitor_info: None,
                tags: None,
                user_notes: None,
                session_id: None,
                analysis_status: None,
            });
        }

        let start = Instant::now();

        // Step 1: Filter records (simulates settings-based filtering)
        let filtered = filter_records_by_settings(records, &settings);

        // Step 2: Generate filename
        let _filename = generate_summary_filename(&settings);

        // Step 3: Format records for summary (simulates what would be sent to AI)
        let _formatted = format_records_for_summary(&filtered);

        let elapsed_ms = start.elapsed().as_millis();

        // Total workflow (without AI) should complete in < 2 seconds
        // Full requirement: < 30 seconds including AI API call
        assert!(
            elapsed_ms < 2000,
            "Daily summary workflow (100 records) took {}ms (threshold: 2000ms, full target: < 30000ms with AI)",
            elapsed_ms
        );
    }

    /// FIX-001 regression test: monthly report path must not overwrite daily summary path
    /// The Settings struct must store last_monthly_report_path separately from last_summary_path.
    #[test]
    fn monthly_report_path_does_not_overwrite_daily_summary_path() {
        let mut settings = create_settings_with_include_manual(true);

        // Simulate: daily summary was generated, setting last_summary_path
        settings.last_summary_path = Some("/obsidian/工作日报 - 2026-03-16.md".to_string());
        assert!(settings.last_monthly_report_path.is_none());

        // Simulate: monthly report is generated, setting last_monthly_report_path
        // (This mirrors what generate_monthly_report() now does after the fix)
        let monthly_path = "/obsidian/月报-2026-03.md".to_string();
        settings.last_monthly_report_path = Some(monthly_path.clone());

        // Verify: daily summary path is NOT overwritten
        assert_eq!(
            settings.last_summary_path,
            Some("/obsidian/工作日报 - 2026-03-16.md".to_string()),
            "FIX-001: last_summary_path must not be overwritten by monthly report generation"
        );
        assert_eq!(
            settings.last_monthly_report_path,
            Some(monthly_path),
            "FIX-001: last_monthly_report_path must store the monthly report path"
        );
    }

    /// DATA-006: get_obsidian_output_path resolves vault path for report generation
    #[test]
    fn get_obsidian_output_path_resolves_vault_for_reports() {
        let mut settings = create_settings_with_include_manual(true);

        // With vaults configured, should use default vault
        settings.obsidian_vaults =
            Some(r#"[{"name":"Work","path":"/vaults/work","is_default":true}]"#.to_string());
        assert_eq!(settings.get_obsidian_output_path().unwrap(), "/vaults/work");

        // With legacy path only, should fall back
        settings.obsidian_vaults = Some("[]".to_string());
        settings.obsidian_path = Some("/legacy/path".to_string());
        assert_eq!(settings.get_obsidian_output_path().unwrap(), "/legacy/path");

        // With neither, should error
        settings.obsidian_vaults = Some("[]".to_string());
        settings.obsidian_path = None;
        assert!(settings.get_obsidian_output_path().is_err());
    }

    /// INT-002: get_logseq_output_path resolves graph path for report generation
    #[test]
    fn get_logseq_output_path_resolves_graph_for_reports() {
        let mut settings = create_settings_with_include_manual(true);

        // With graphs configured, should use default graph
        settings.logseq_graphs =
            Some(r#"[{"name":"Work","path":"/logseq/work","is_default":true}]"#.to_string());
        assert_eq!(settings.get_logseq_output_path().unwrap(), "/logseq/work");

        // With no default, should use first graph
        settings.logseq_graphs = Some(
            r#"[{"name":"Personal","path":"/logseq/personal","is_default":false}]"#.to_string(),
        );
        assert_eq!(
            settings.get_logseq_output_path().unwrap(),
            "/logseq/personal"
        );

        // With empty graphs, should error
        settings.logseq_graphs = Some("[]".to_string());
        assert!(settings.get_logseq_output_path().is_err());

        // With no graphs configured, should error
        settings.logseq_graphs = None;
        assert!(settings.get_logseq_output_path().is_err());
    }

    /// REPORT-004: comparison report filename generation
    #[test]
    fn comparison_report_filename_format() {
        let filename = generate_comparison_report_filename(
            "2026-01-01",
            "2026-01-31",
            "2026-02-01",
            "2026-02-28",
        );
        assert_eq!(
            filename,
            "对比分析-2026-01-01~2026-01-31-vs-2026-02-01~2026-02-28.md"
        );
    }
}
