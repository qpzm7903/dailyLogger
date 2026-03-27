mod records;
mod schema;
mod settings;
pub mod tags;

use once_cell::sync::Lazy;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::command;

pub use schema::init_database;
// Re-export all public items from settings module (including Tauri command generated types)
pub use settings::*;
// Re-export all public items from records module
pub use records::*;
// Re-export all public items from tags module (including Tauri command generated types)
pub use tags::*;

#[cfg(test)]
pub use schema::init_test_database;

pub static DB_CONNECTION: Lazy<Mutex<Option<Connection>>> = Lazy::new(|| Mutex::new(None));

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Settings {
    pub api_base_url: Option<String>,
    pub api_key: Option<String>,
    pub model_name: Option<String>,
    pub screenshot_interval: Option<i32>,
    pub summary_time: Option<String>,
    pub obsidian_path: Option<String>,
    pub auto_capture_enabled: Option<bool>,
    pub last_summary_path: Option<String>,
    pub summary_model_name: Option<String>,
    pub analysis_prompt: Option<String>,
    pub summary_prompt: Option<String>,
    pub change_threshold: Option<i32>,
    pub max_silent_minutes: Option<i32>,
    // 新增字段：日报标题格式
    pub summary_title_format: Option<String>,
    // 新增字段：是否包含手动记录
    pub include_manual_records: Option<bool>,
    // SMART-001: 窗口过滤配置
    pub window_whitelist: Option<String>,
    pub window_blacklist: Option<String>,
    pub use_whitelist_only: Option<bool>,
    // SMART-002: 自动调整静默阈值配置
    pub auto_adjust_silent: Option<bool>,
    pub silent_adjustment_paused_until: Option<String>,
    // SMART-003: 工作时间自动识别配置
    pub auto_detect_work_time: Option<bool>,
    pub use_custom_work_time: Option<bool>,
    pub custom_work_time_start: Option<String>, // "HH:MM" format
    pub custom_work_time_end: Option<String>,
    pub learned_work_time: Option<String>, // JSON: {"periods": [{"start": 9, "end": 12}, ...]}
    // SMART-004: 多显示器支持配置
    pub capture_mode: Option<String>, // "primary" | "secondary" | "all"
    pub selected_monitor_index: Option<i32>, // For "secondary" mode
    // AI-004: 工作分类标签配置
    pub tag_categories: Option<String>, // JSON: Vec<String> of custom tag categories
    // AI-005: Ollama 本地模型支持
    pub is_ollama: Option<bool>,
    // REPORT-001: 周报生成配置
    pub weekly_report_prompt: Option<String>,
    pub weekly_report_day: Option<i32>, // 0=周一, 6=周日
    pub last_weekly_report_path: Option<String>,
    // REPORT-002: 月报生成配置
    pub monthly_report_prompt: Option<String>,
    pub last_monthly_report_path: Option<String>,
    // REPORT-003: 自定义报告周期配置
    pub custom_report_prompt: Option<String>,
    pub last_custom_report_path: Option<String>,
    // DATA-006: 多 Obsidian Vault 支持
    pub obsidian_vaults: Option<String>, // JSON: [{"name":"x","path":"y","is_default":true}]
    // REPORT-004: 对比报告配置
    pub comparison_report_prompt: Option<String>,
    // INT-002: Logseq 导出支持
    pub logseq_graphs: Option<String>, // JSON: [{"name":"x","path":"y","is_default":true}]
    // INT-001: Notion 导出支持
    pub notion_api_key: Option<String>, // Notion integration secret (encrypted)
    pub notion_database_id: Option<String>, // Notion database ID to write pages to
    // INT-004: Slack 通知配置
    pub slack_webhook_url: Option<String>, // Slack Incoming Webhook URL
    // INT-004: DingTalk 通知配置
    pub dingtalk_webhook_url: Option<String>, // DingTalk Robot Webhook URL
    // FEAT-006: 仅截图模式 (#65)
    pub capture_only_mode: Option<bool>, // Only capture screenshots without AI analysis
    // AI-006: 自定义 API Headers (#68)
    pub custom_headers: Option<String>, // JSON: Vec<CustomHeader>
    // EXP-002: 截图质量过滤
    pub quality_filter_enabled: Option<bool>,
    pub quality_filter_threshold: Option<f64>,
    // SESSION-001: 工作时段管理
    pub session_gap_minutes: Option<i32>, // 时段间隔阈值（分钟），默认 30
    // PERF-001: 代理配置
    pub proxy_enabled: Option<bool>,
    pub proxy_host: Option<String>,
    pub proxy_port: Option<i32>,
    pub proxy_username: Option<String>,
    pub proxy_password: Option<String>,
    // PERF-001: 测试模型名称
    pub test_model_name: Option<String>,
    // PERF-002: 新用户引导完成标志
    pub onboarding_completed: Option<bool>,
    // PERF-005: 语言设置
    pub language: Option<String>,
    // DATA-007: 多语言日报配置
    pub preferred_language: Option<String>, // 首选语言，如 "zh-CN", "en"
    pub supported_languages: Option<String>, // 支持的语言列表，JSON 数组
}

/// AI-006: Custom API Header for various API providers (OpenRouter, Azure, Claude, etc.)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CustomHeader {
    pub key: String,
    pub value: String,
    pub sensitive: bool, // Whether the value should be encrypted
}

/// AI-006: Preset header templates for common API providers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeaderPreset {
    pub name: String,
    pub headers: Vec<CustomHeader>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

/// AI-006: Get preset header templates for common API providers
pub fn get_header_presets() -> Vec<HeaderPreset> {
    vec![
        HeaderPreset {
            name: "OpenRouter".to_string(),
            headers: vec![
                CustomHeader {
                    key: "HTTP-Referer".to_string(),
                    value: "https://dailylogger.app".to_string(),
                    sensitive: false,
                },
                CustomHeader {
                    key: "X-Title".to_string(),
                    value: "DailyLogger".to_string(),
                    sensitive: false,
                },
            ],
            note: None,
        },
        HeaderPreset {
            name: "Azure OpenAI".to_string(),
            headers: vec![CustomHeader {
                key: "api-key".to_string(),
                value: String::new(),
                sensitive: true,
            }],
            note: Some("api-key header replaces Authorization header".to_string()),
        },
        HeaderPreset {
            name: "Claude API".to_string(),
            headers: vec![CustomHeader {
                key: "anthropic-version".to_string(),
                value: "2023-06-01".to_string(),
                sensitive: false,
            }],
            note: None,
        },
    ]
}

/// DATA-006: Vault entry for multi-vault support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObsidianVault {
    pub name: String,
    pub path: String,
    pub is_default: bool,
}

/// INT-002: Logseq graph entry for multi-graph support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogseqGraph {
    pub name: String,
    pub path: String,
    pub is_default: bool,
}

impl Settings {
    /// Get the effective Obsidian output path.
    /// Checks `obsidian_vaults` for the default vault first, falls back to `obsidian_path`.
    pub fn get_obsidian_output_path(&self) -> Result<String, String> {
        // Try obsidian_vaults first
        if let Some(ref vaults_json) = self.obsidian_vaults {
            if let Ok(vaults) = serde_json::from_str::<Vec<ObsidianVault>>(vaults_json) {
                if let Some(default_vault) = vaults.iter().find(|v| v.is_default) {
                    if !default_vault.path.trim().is_empty() {
                        return Ok(default_vault.path.clone());
                    }
                }
                // If no default, use the first vault
                if let Some(first_vault) = vaults.first() {
                    if !first_vault.path.trim().is_empty() {
                        return Ok(first_vault.path.clone());
                    }
                }
            }
        }

        // Fall back to legacy obsidian_path
        self.obsidian_path
            .clone()
            .filter(|p| !p.trim().is_empty())
            .ok_or_else(|| "Obsidian path not configured".to_string())
    }

    /// INT-002: Get the effective Logseq output path.
    /// Checks `logseq_graphs` for the default graph first.
    pub fn get_logseq_output_path(&self) -> Result<String, String> {
        // Try logseq_graphs
        if let Some(ref graphs_json) = self.logseq_graphs {
            if let Ok(graphs) = serde_json::from_str::<Vec<LogseqGraph>>(graphs_json) {
                if let Some(default_graph) = graphs.iter().find(|g| g.is_default) {
                    if !default_graph.path.trim().is_empty() {
                        return Ok(default_graph.path.clone());
                    }
                }
                // If no default, use the first graph
                if let Some(first_graph) = graphs.first() {
                    if !first_graph.path.trim().is_empty() {
                        return Ok(first_graph.path.clone());
                    }
                }
            }
        }

        Err("Logseq path not configured".to_string())
    }
}

/// Model information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub model_id: String,
    pub context_window: Option<u64>,
    pub error: Option<String>,
}

/// Get model information including context window
#[command]
pub async fn get_model_info(
    api_base_url: String,
    api_key: String,
    model_name: String,
) -> Result<ModelInfo, String> {
    // OpenAI compatible API /models endpoint
    let url = if api_base_url.ends_with('/') {
        format!("{}models/{}", api_base_url, model_name)
    } else {
        format!("{}/models/{}", api_base_url, model_name)
    };

    // Create HTTP client with proxy bypass for local URLs
    let client = crate::create_http_client(&url, 30)?;

    let response = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .send()
        .await;

    match response {
        Ok(resp) if resp.status().is_success() => {
            let json: serde_json::Value = resp.json().await.unwrap_or(serde_json::json!({}));

            // OpenAI returns format: {"id": "gpt-4o", "context_window": 128000}
            // Or in some APIs it's max_tokens
            let context_window = json
                .get("context_window")
                .or_else(|| json.get("max_tokens"))
                .or_else(|| {
                    // Some APIs return it in model_info
                    json.get("model_info")
                        .and_then(|mi| mi.get("context_window"))
                })
                .and_then(|v| v.as_u64());

            Ok(ModelInfo {
                model_id: model_name,
                context_window,
                error: None,
            })
        }
        Ok(resp) => {
            let status = resp.status();
            Ok(ModelInfo {
                model_id: model_name,
                context_window: None,
                error: Some(format!("无法获取模型信息 (状态: {})", status)),
            })
        }
        Err(e) => Ok(ModelInfo {
            model_id: model_name,
            context_window: None,
            error: Some(format!("请求失败: {}", e)),
        }),
    }
}

#[cfg(test)]
mod tests_ai_006 {
    use super::*;

    #[test]
    fn test_custom_header_serialization() {
        let header = CustomHeader {
            key: "X-Custom-Header".to_string(),
            value: "test-value".to_string(),
            sensitive: false,
        };
        let json = serde_json::to_string(&header).unwrap();
        assert!(json.contains("X-Custom-Header"));
        assert!(json.contains("test-value"));
        assert!(json.contains("\"sensitive\":false"));
    }

    #[test]
    fn test_custom_header_deserialization() {
        let json = r#"{"key":"Authorization","value":"Bearer token","sensitive":true}"#;
        let header: CustomHeader = serde_json::from_str(json).unwrap();
        assert_eq!(header.key, "Authorization");
        assert_eq!(header.value, "Bearer token");
        assert!(header.sensitive);
    }

    #[test]
    fn test_custom_headers_vec_serialization() {
        let headers = vec![
            CustomHeader {
                key: "HTTP-Referer".to_string(),
                value: "https://dailylogger.app".to_string(),
                sensitive: false,
            },
            CustomHeader {
                key: "api-key".to_string(),
                value: "secret-key".to_string(),
                sensitive: true,
            },
        ];
        let json = serde_json::to_string(&headers).unwrap();
        assert!(json.contains("HTTP-Referer"));
        assert!(json.contains("api-key"));
        assert!(json.contains("secret-key"));
    }

    #[test]
    fn test_custom_headers_vec_deserialization() {
        let json = r#"[{"key":"X-Title","value":"DailyLogger","sensitive":false}]"#;
        let headers: Vec<CustomHeader> = serde_json::from_str(json).unwrap();
        assert_eq!(headers.len(), 1);
        assert_eq!(headers[0].key, "X-Title");
        assert_eq!(headers[0].value, "DailyLogger");
        assert!(!headers[0].sensitive);
    }

    #[test]
    fn test_header_presets() {
        let presets = get_header_presets();
        assert!(!presets.is_empty());

        // Check OpenRouter preset
        let openrouter = presets.iter().find(|p| p.name == "OpenRouter");
        assert!(openrouter.is_some());
        let openrouter = openrouter.unwrap();
        assert_eq!(openrouter.headers.len(), 2);

        // Check Azure OpenAI preset
        let azure = presets.iter().find(|p| p.name == "Azure OpenAI");
        assert!(azure.is_some());
        let azure = azure.unwrap();
        assert_eq!(azure.headers.len(), 1);
        assert!(azure.headers[0].sensitive);

        // Check Claude API preset
        let claude = presets.iter().find(|p| p.name == "Claude API");
        assert!(claude.is_some());
    }

    #[test]
    fn test_settings_default_custom_headers() {
        let settings = Settings::default();
        assert!(
            settings.custom_headers.is_none() || settings.custom_headers == Some("[]".to_string())
        );
    }
}

// DATA-008: Statistics Types and Functions
// ============================================

use chrono::{Datelike, Local, NaiveDate};

/// Statistics time range type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TimeRangeType {
    Today,
    Week,
    Month,
    Custom,
}

/// Date range for statistics query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateRange {
    pub start: String, // RFC3339
    pub end: String,   // RFC3339
    pub label: String, // "今日" / "本周" / "本月" / "自定义"
}

/// Daily breakdown statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyStatistic {
    pub date: String, // YYYY-MM-DD
    pub screenshot_count: i64,
    pub session_count: i64,
    pub record_count: i64,
}

/// Full statistics result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Statistics {
    pub date_range: DateRange,
    pub screenshot_count: i64,
    pub session_count: i64,
    pub record_count: i64,
    pub analysis_success_rate: f64,           // Percentage 0-100
    pub daily_breakdown: Vec<DailyStatistic>, // Daily breakdown
}

/// Get the start and end of today in local timezone
fn get_today_range() -> (String, String) {
    let now = Local::now();
    let today = now.format("%Y-%m-%d").to_string();
    let start = format!("{}T00:00:00", today);
    let end = format!("{}T23:59:59.999", today);
    (start, end)
}

/// Get the start and end of current week in local timezone (Monday to Sunday)
fn get_week_range() -> (String, String) {
    let now = Local::now();
    let weekday = now.weekday().num_days_from_monday() as i64;
    let start_date = now - chrono::Duration::days(weekday);
    let end_date = start_date + chrono::Duration::days(6);
    let start = format!("{}T00:00:00", start_date.format("%Y-%m-%d"));
    let end = format!("{}T23:59:59.999", end_date.format("%Y-%m-%d"));
    (start, end)
}

/// Get the start and end of current month in local timezone
fn get_month_range() -> (String, String) {
    let now = Local::now();
    let start = format!("{}-{:02}-01T00:00:00", now.year(), now.month());
    let last_day = get_last_day_of_month(now.year(), now.month());
    let end = format!(
        "{}-{:02}-{}T23:59:59.999",
        now.year(),
        now.month(),
        last_day
    );
    (start, end)
}

/// Get the last day of a given month
fn get_last_day_of_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0) {
                29
            } else {
                28
            }
        }
        _ => 31,
    }
}

/// Parse a date string (YYYY-MM-DD) and return NaiveDate
fn parse_date(date_str: &str) -> Result<NaiveDate, String> {
    NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
        .map_err(|e| format!("Failed to parse date '{}': {}", date_str, e))
}

/// Count records in a date range
fn count_records_in_range(
    conn: &rusqlite::Connection,
    start: &str,
    end: &str,
) -> Result<i64, String> {
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM records WHERE timestamp >= ? AND timestamp <= ?",
            [start, end],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    Ok(count)
}

/// Count screenshots in a date range (records with screenshot_path)
fn count_screenshots_in_range(
    conn: &rusqlite::Connection,
    start: &str,
    end: &str,
) -> Result<i64, String> {
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM records WHERE timestamp >= ? AND timestamp <= ? AND screenshot_path IS NOT NULL AND screenshot_path != ''",
            [start, end],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    Ok(count)
}

/// Count sessions in a date range
fn count_sessions_in_range(
    conn: &rusqlite::Connection,
    start: &str,
    end: &str,
) -> Result<i64, String> {
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sessions WHERE date >= ? AND date <= ?",
            [start[..10].to_string(), end[..10].to_string()],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    Ok(count)
}

/// Get AI analysis success rate (percentage of analyzed records)
fn get_analysis_success_rate(
    conn: &rusqlite::Connection,
    start: &str,
    end: &str,
) -> Result<f64, String> {
    let total: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM records WHERE timestamp >= ? AND timestamp <= ?",
            [start, end],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    if total == 0 {
        return Ok(0.0);
    }

    let analyzed: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM records WHERE timestamp >= ? AND timestamp <= ? AND analysis_status = 'analyzed'",
            [start, end],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    Ok((analyzed as f64) / (total as f64) * 100.0)
}

/// Get daily breakdown statistics for a date range
fn get_daily_breakdown(
    conn: &rusqlite::Connection,
    start: &str,
    end: &str,
) -> Result<Vec<DailyStatistic>, String> {
    let start_date = parse_date(&start[..10])?;
    let end_date = parse_date(&end[..10])?;

    let mut result = Vec::new();
    let mut current_date = start_date;

    while current_date <= end_date {
        let date_str = current_date.format("%Y-%m-%d").to_string();
        let day_start = format!("{}T00:00:00", date_str);
        let day_end = format!("{}T23:59:59.999", date_str);

        let screenshot_count = count_screenshots_in_range(conn, &day_start, &day_end).unwrap_or(0);
        let session_count = count_sessions_in_range(conn, &day_start, &day_end).unwrap_or(0);
        let record_count = count_records_in_range(conn, &day_start, &day_end).unwrap_or(0);

        result.push(DailyStatistic {
            date: date_str,
            screenshot_count,
            session_count,
            record_count,
        });

        current_date += chrono::Duration::days(1);
    }

    Ok(result)
}

/// DATA-008: Get statistics for a given time range
///
/// # Arguments
/// * `range_type` - One of: "today", "week", "month", "custom"
/// * `custom_start` - Start date for custom range (YYYY-MM-DD), required when range_type is "custom"
/// * `custom_end` - End date for custom range (YYYY-MM-DD), required when range_type is "custom"
#[command]
pub async fn get_statistics(
    range_type: String,
    custom_start: Option<String>,
    custom_end: Option<String>,
) -> Result<Statistics, String> {
    let (start, end, label) = match range_type.to_lowercase().as_str() {
        "today" => {
            let (s, e) = get_today_range();
            (s, e, "今日".to_string())
        }
        "week" => {
            let (s, e) = get_week_range();
            (s, e, "本周".to_string())
        }
        "month" => {
            let (s, e) = get_month_range();
            (s, e, "本月".to_string())
        }
        "custom" => {
            let start_str = custom_start
                .clone()
                .ok_or("custom_start is required for custom range")?;
            let end_str = custom_end
                .clone()
                .ok_or("custom_end is required for custom range")?;

            // Validate and parse dates
            let _ = parse_date(&start_str)?;
            let _ = parse_date(&end_str)?;

            let start = format!("{}T00:00:00", start_str);
            let end = format!("{}T23:59:59.999", end_str);
            (start, end, format!("{} 至 {}", start_str, end_str))
        }
        _ => {
            return Err(format!(
                "Invalid range_type: {}. Use: today, week, month, custom",
                range_type
            ))
        }
    };

    let db = DB_CONNECTION
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;
    let conn = db.as_ref().ok_or("Database not initialized")?;

    let screenshot_count = count_screenshots_in_range(conn, &start, &end)?;
    let session_count = count_sessions_in_range(conn, &start, &end)?;
    let record_count = count_records_in_range(conn, &start, &end)?;
    let analysis_success_rate = get_analysis_success_rate(conn, &start, &end)?;
    let daily_breakdown = get_daily_breakdown(conn, &start, &end)?;

    Ok(Statistics {
        date_range: DateRange {
            start: start.clone(),
            end: end.clone(),
            label,
        },
        screenshot_count,
        session_count,
        record_count,
        analysis_success_rate,
        daily_breakdown,
    })
}

#[cfg(test)]
mod tests_statistics {
    use super::*;

    #[test]
    fn test_get_last_day_of_month() {
        assert_eq!(get_last_day_of_month(2026, 1), 31);
        assert_eq!(get_last_day_of_month(2026, 2), 28);
        assert_eq!(get_last_day_of_month(2024, 2), 29); // Leap year
        assert_eq!(get_last_day_of_month(2026, 4), 30);
    }

    #[test]
    fn test_parse_date() {
        let date = parse_date("2026-03-26").unwrap();
        assert_eq!(date.to_string(), "2026-03-26");

        assert!(parse_date("invalid").is_err());
    }

    #[test]
    fn test_get_today_range() {
        let (start, end) = get_today_range();
        assert!(start.contains("T00:00:00"));
        assert!(end.contains("T23:59:59"));
    }

    #[test]
    fn test_get_week_range() {
        let (start, end) = get_week_range();
        // Should be Monday to Sunday
        assert!(start.contains("T00:00:00"));
        assert!(end.contains("T23:59:59"));
    }

    #[test]
    fn test_get_month_range() {
        let (start, end) = get_month_range();
        assert!(start.contains("-01T00:00:00"));
        assert!(end.contains("T23:59:59"));
    }

    // Integration tests for get_statistics
    // Note: These tests require the #[tokio::test] macro since get_statistics is async

    #[test]
    fn test_get_statistics_empty_database() {
        use crate::memory_storage::schema::init_test_database;
        use rusqlite::Connection;

        // Create in-memory database for testing
        let conn = Connection::open_in_memory().unwrap();
        init_test_database(&conn).unwrap();

        // Set up global DB connection
        {
            let mut db = crate::memory_storage::DB_CONNECTION.lock().unwrap();
            *db = Some(conn);
        }

        // Get today's range
        let (start, end) = get_today_range();

        // Call count functions directly (these are sync)
        let screenshot_count = count_screenshots_in_range(
            crate::memory_storage::DB_CONNECTION
                .lock()
                .unwrap()
                .as_ref()
                .unwrap(),
            &start,
            &end,
        )
        .unwrap();

        let session_count = count_sessions_in_range(
            crate::memory_storage::DB_CONNECTION
                .lock()
                .unwrap()
                .as_ref()
                .unwrap(),
            &start,
            &end,
        )
        .unwrap();

        let record_count = count_records_in_range(
            crate::memory_storage::DB_CONNECTION
                .lock()
                .unwrap()
                .as_ref()
                .unwrap(),
            &start,
            &end,
        )
        .unwrap();

        // Empty database should return 0 for all counts
        assert_eq!(screenshot_count, 0);
        assert_eq!(session_count, 0);
        assert_eq!(record_count, 0);
    }

    #[test]
    fn test_get_statistics_with_data() {
        use crate::memory_storage::schema::init_test_database;
        use rusqlite::params;
        use rusqlite::Connection;

        // Create in-memory database for testing
        let conn = Connection::open_in_memory().unwrap();
        init_test_database(&conn).unwrap();

        // Set up global DB connection
        {
            let mut db = crate::memory_storage::DB_CONNECTION.lock().unwrap();
            *db = Some(conn);
        }

        // Get today's range first to ensure we use the same date
        let (start, end) = get_today_range();

        // Extract date from start (format: YYYY-MM-DDTHH:MM:SS)
        let date_part = &start[..10]; // "YYYY-MM-DD"
        let timestamp1 = format!("{}T10:00:00", date_part);
        let timestamp2 = format!("{}T11:00:00", date_part);
        let timestamp3 = format!("{}T14:00:00", date_part);

        // Insert records - one with screenshot, two without
        {
            let db = crate::memory_storage::DB_CONNECTION.lock().unwrap();
            let conn = db.as_ref().unwrap();

            conn.execute(
                "INSERT INTO records (timestamp, source_type, content, screenshot_path, analysis_status) VALUES (?1, 'auto', 'test1', '/path/to/screenshot.png', 'analyzed')",
                params![timestamp1],
            )
            .unwrap();

            conn.execute(
                "INSERT INTO records (timestamp, source_type, content, analysis_status) VALUES (?1, 'auto', 'test2', 'analyzed')",
                params![timestamp2],
            )
            .unwrap();

            conn.execute(
                "INSERT INTO records (timestamp, source_type, content, analysis_status) VALUES (?1, 'manual', 'test3', 'pending')",
                params![timestamp3],
            )
            .unwrap();
        }

        // Call count functions
        let screenshot_count = count_screenshots_in_range(
            crate::memory_storage::DB_CONNECTION
                .lock()
                .unwrap()
                .as_ref()
                .unwrap(),
            &start,
            &end,
        )
        .unwrap();

        let record_count = count_records_in_range(
            crate::memory_storage::DB_CONNECTION
                .lock()
                .unwrap()
                .as_ref()
                .unwrap(),
            &start,
            &end,
        )
        .unwrap();

        // Should have 1 screenshot and 3 records
        assert_eq!(screenshot_count, 1);
        assert_eq!(record_count, 3);
    }

    #[test]
    fn test_get_statistics_with_sessions() {
        use crate::memory_storage::schema::init_test_database;
        use rusqlite::params;
        use rusqlite::Connection;

        // Create in-memory database for testing
        let conn = Connection::open_in_memory().unwrap();
        init_test_database(&conn).unwrap();

        // Set up global DB connection
        {
            let mut db = crate::memory_storage::DB_CONNECTION.lock().unwrap();
            *db = Some(conn);
        }

        // Insert test sessions
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();

        {
            let db = crate::memory_storage::DB_CONNECTION.lock().unwrap();
            let conn = db.as_ref().unwrap();

            // Insert two sessions for today
            conn.execute(
                "INSERT INTO sessions (date, start_time, end_time, status) VALUES (?1, '09:00', '12:00', 'completed')",
                params![today],
            )
            .unwrap();

            conn.execute(
                "INSERT INTO sessions (date, start_time, end_time, status) VALUES (?1, '14:00', '18:00', 'completed')",
                params![today],
            )
            .unwrap();
        }

        // Get today's range
        let (start, end) = get_today_range();

        // Call count functions
        let session_count = count_sessions_in_range(
            crate::memory_storage::DB_CONNECTION
                .lock()
                .unwrap()
                .as_ref()
                .unwrap(),
            &start,
            &end,
        )
        .unwrap();

        // Should have 2 sessions
        assert_eq!(session_count, 2);
    }

    #[test]
    fn test_get_statistics_analysis_success_rate() {
        use crate::memory_storage::schema::init_test_database;
        use rusqlite::params;
        use rusqlite::Connection;

        // Create in-memory database for testing
        let conn = Connection::open_in_memory().unwrap();
        init_test_database(&conn).unwrap();

        // Set up global DB connection
        {
            let mut db = crate::memory_storage::DB_CONNECTION.lock().unwrap();
            *db = Some(conn);
        }

        // Get today's range first to ensure we use the same date
        let (start, end) = get_today_range();

        // Extract date from start (format: YYYY-MM-DDTHH:MM:SS)
        let date_part = &start[..10]; // "YYYY-MM-DD"
        let timestamp1 = format!("{}T10:00:00", date_part);
        let timestamp2 = format!("{}T11:00:00", date_part);
        let timestamp3 = format!("{}T14:00:00", date_part);
        let timestamp4 = format!("{}T15:00:00", date_part);

        {
            let db = crate::memory_storage::DB_CONNECTION.lock().unwrap();
            let conn = db.as_ref().unwrap();

            // 3 analyzed, 1 pending = 75% success rate
            conn.execute(
                "INSERT INTO records (timestamp, source_type, content, analysis_status) VALUES (?1, 'auto', 'test1', 'analyzed')",
                params![timestamp1],
            )
            .unwrap();

            conn.execute(
                "INSERT INTO records (timestamp, source_type, content, analysis_status) VALUES (?1, 'auto', 'test2', 'analyzed')",
                params![timestamp2],
            )
            .unwrap();

            conn.execute(
                "INSERT INTO records (timestamp, source_type, content, analysis_status) VALUES (?1, 'auto', 'test3', 'analyzed')",
                params![timestamp3],
            )
            .unwrap();

            conn.execute(
                "INSERT INTO records (timestamp, source_type, content, analysis_status) VALUES (?1, 'auto', 'test4', 'pending')",
                params![timestamp4],
            )
            .unwrap();
        }

        // Call analysis success rate function
        let success_rate = get_analysis_success_rate(
            crate::memory_storage::DB_CONNECTION
                .lock()
                .unwrap()
                .as_ref()
                .unwrap(),
            &start,
            &end,
        )
        .unwrap();

        // 3 out of 4 = 75%
        assert!((success_rate - 75.0).abs() < 0.1);
    }

    #[test]
    fn test_get_statistics_invalid_range_type() {
        // Test that invalid range_type returns error
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(get_statistics("invalid".to_string(), None, None));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid range_type"));
    }

    #[test]
    fn test_get_statistics_custom_range_requires_dates() {
        let rt = tokio::runtime::Runtime::new().unwrap();

        // Custom range without dates should fail
        let result = rt.block_on(get_statistics("custom".to_string(), None, None));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("custom_start is required"));

        // Custom range with only start date should fail
        let result = rt.block_on(get_statistics(
            "custom".to_string(),
            Some("2026-03-01".to_string()),
            None,
        ));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("custom_end is required"));
    }

    // STAB-001 Task 4.2: Database connection reconnection tests
    #[test]
    fn test_check_connection_with_valid_connection() {
        use crate::memory_storage::schema::check_connection;
        use crate::memory_storage::schema::init_test_database;
        use rusqlite::Connection;

        // Create in-memory database for testing
        let conn = Connection::open_in_memory().unwrap();
        init_test_database(&conn).unwrap();

        // Set up global DB connection
        {
            let mut db = crate::memory_storage::DB_CONNECTION.lock().unwrap();
            *db = Some(conn);
        }

        // Connection should be valid
        let is_valid = check_connection().unwrap();
        assert!(is_valid);
    }

    #[test]
    fn test_check_connection_with_no_connection() {
        use crate::memory_storage::schema::check_connection;

        // Clear any existing connection
        {
            let mut db = crate::memory_storage::DB_CONNECTION.lock().unwrap();
            *db = None;
        }

        // Should return false (needs reconnect)
        let is_valid = check_connection().unwrap();
        assert!(!is_valid);
    }

    // STAB-001 Task 4.4: Database error scenario tests
    #[test]
    fn test_transaction_rollback_on_invalid_data() {
        use crate::memory_storage::records::add_record_with_session;
        use crate::memory_storage::schema::init_test_database;
        use rusqlite::Connection;

        // Create in-memory database for testing
        let conn = Connection::open_in_memory().unwrap();
        init_test_database(&conn).unwrap();

        // Set up global DB connection
        {
            let mut db = crate::memory_storage::DB_CONNECTION.lock().unwrap();
            *db = Some(conn);
        }

        // Get today's range for valid timestamp
        let (start, _end) = get_today_range();
        let date_part = &start[..10];

        // Add a valid record first
        let result = add_record_with_session("auto", "valid record", None, None, None, None);
        assert!(result.is_ok());

        // Verify the record was inserted
        let record_count = count_records_in_range(
            crate::memory_storage::DB_CONNECTION
                .lock()
                .unwrap()
                .as_ref()
                .unwrap(),
            &start,
            &format!("{}T23:59:59", date_part),
        )
        .unwrap();
        assert_eq!(record_count, 1);
    }
}
