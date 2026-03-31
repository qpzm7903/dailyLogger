pub mod migration;
mod records;
mod schema;
mod settings;
pub mod tags;

use once_cell::sync::Lazy;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::command;

use crate::errors::{AppError, AppResult};

pub use schema::init_database;
// Re-export all public items from settings module (including Tauri command generated types)
pub use settings::*;
// Re-export all public items from records module
pub use records::*;
// Re-export all public items from tags module (including Tauri command generated types)
pub use tags::*;

#[cfg(test)]
pub use schema::init_test_database;

/// DEBT-001: Unified test database setup helper.
/// Creates an in-memory database with the complete schema and sets it as the global DB_CONNECTION.
/// This ensures all tests use a consistent schema and avoids schema drift.
#[cfg(test)]
pub fn setup_test_db_with_schema() {
    let conn = Connection::open_in_memory().expect("Failed to create in-memory database");
    init_test_database(&conn).expect("Failed to initialize test database schema");
    let mut db = DB_CONNECTION.lock().expect("Failed to lock DB_CONNECTION");
    *db = Some(conn);
}

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
    // VAULT-001: Auto-detect vault by active window title
    pub auto_detect_vault_by_window: Option<bool>,
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
    // STAB-002: 自动备份配置
    pub auto_backup_enabled: Option<bool>,
    pub auto_backup_interval: Option<String>, // "daily" | "weekly" | "monthly"
    pub auto_backup_retention: Option<i32>,   // 保留数量，3-20
    pub last_auto_backup_at: Option<String>,  // RFC3339 时间戳
    // FEAT-008: 自定义导出模板 (v3.8.0)
    pub custom_export_template: Option<String>, // 用户自定义导出模板
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
    /// VAULT-001: Window title patterns for auto-detection (comma-separated, case-insensitive match)
    pub window_patterns: Option<Vec<String>>,
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

    /// VAULT-001: Get vault by name
    pub fn get_vault_by_name(&self, name: &str) -> Option<ObsidianVault> {
        if let Some(ref vaults_json) = self.obsidian_vaults {
            if let Ok(vaults) = serde_json::from_str::<Vec<ObsidianVault>>(vaults_json) {
                if let Some(vault) = vaults.into_iter().find(|v| v.name == name) {
                    return Some(vault);
                }
            }
        }
        None
    }

    /// VAULT-001: Get vault by active window title (for auto-detection)
    pub fn get_vault_by_window_title(&self, title: &str) -> Option<ObsidianVault> {
        if let Some(ref vaults_json) = self.obsidian_vaults {
            if let Ok(vaults) = serde_json::from_str::<Vec<ObsidianVault>>(vaults_json) {
                for vault in vaults {
                    if let Some(ref patterns) = vault.window_patterns {
                        if crate::window_info::matches_any(title, patterns) {
                            return Some(vault);
                        }
                    }
                }
            }
        }
        None
    }

    /// VAULT-001: Get effective output vault path
    pub fn get_effective_vault(
        &self,
        vault_name: Option<&str>,
        auto_detect: bool,
    ) -> AppResult<String> {
        // 1. If explicitly specified, use that vault
        if let Some(name) = vault_name {
            if let Some(vault) = self.get_vault_by_name(name) {
                return Ok(vault.path.clone());
            }
            return Err(AppError::validation(format!("Vault '{}' not found", name)));
        }

        // 2. If auto-detect is enabled, try to detect by window
        if auto_detect {
            let window = crate::window_info::get_active_window();
            if !window.title.is_empty() {
                if let Some(vault) = self.get_vault_by_window_title(&window.title) {
                    tracing::info!(
                        "Auto-detected vault '{}' for window '{}'",
                        vault.name,
                        window.title
                    );
                    return Ok(vault.path.clone());
                }
            }
        }

        // 3. Fall back to default vault (existing logic)
        self.get_obsidian_output_path().map_err(AppError::internal)
    }
}

/// Model information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub model_id: String,
    pub context_window: Option<u64>,
    pub error: Option<String>,
}

// Note: get_model_info command has been moved to commands/model_commands.rs
// The business logic is now in services/model_service.rs::get_model_info_service

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

// ANALYTICS-001: Productivity Trend Types
// ============================================

/// Period comparison result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeriodComparison {
    pub current_total: i64,
    pub previous_total: i64,
    pub change_percent: f64, // Percentage change, positive = increase
    pub trend: String,       // "up", "down", or "stable"
}

/// Hourly distribution for peak hours analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HourlyDistribution {
    pub hour: u32,       // 0-23
    pub count: i64,      // Number of records in this hour
    pub percentage: f64, // Percentage of total
}

/// Daily trend data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyTrendPoint {
    pub date: String, // YYYY-MM-DD
    pub screenshot_count: i64,
    pub record_count: i64,
}

/// Full productivity trend result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductivityTrend {
    pub comparison_type: String, // "week" or "month"
    pub current_period: DateRange,
    pub previous_period: DateRange,
    pub screenshot_comparison: PeriodComparison,
    pub record_comparison: PeriodComparison,
    pub daily_trend: Vec<DailyTrendPoint>, // Daily data for current period
    pub peak_hours: Vec<HourlyDistribution>, // Top 5 busiest hours
    pub average_daily_records: f64,        // Average records per day
}

/// Get the start and end of previous week (Monday to Sunday)
fn get_previous_week_range() -> (String, String) {
    let now = Local::now();
    let weekday = now.weekday().num_days_from_monday() as i64;
    // Current week's Monday
    let current_week_monday = now - chrono::Duration::days(weekday);
    // Previous week's Monday = current week's Monday - 7 days
    let previous_week_monday = current_week_monday - chrono::Duration::days(7);
    let previous_week_sunday = previous_week_monday + chrono::Duration::days(6);
    let start = format!("{}T00:00:00", previous_week_monday.format("%Y-%m-%d"));
    let end = format!("{}T23:59:59.999", previous_week_sunday.format("%Y-%m-%d"));
    (start, end)
}

/// Get the start and end of previous month
fn get_previous_month_range() -> (String, String) {
    let now = Local::now();
    let (year, month) = if now.month() == 1 {
        (now.year() - 1, 12)
    } else {
        (now.year(), now.month() - 1)
    };
    let start = format!("{}-{:02}-01T00:00:00", year, month);
    let last_day = get_last_day_of_month(year, month);
    let end = format!("{}-{:02}-{}T23:59:59.999", year, month, last_day);
    (start, end)
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
fn parse_date(date_str: &str) -> AppResult<NaiveDate> {
    NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
        .map_err(|e| AppError::validation(format!("Failed to parse date '{}': {}", date_str, e)))
}

/// Count records in a date range
fn count_records_in_range(conn: &rusqlite::Connection, start: &str, end: &str) -> AppResult<i64> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM records WHERE timestamp >= ? AND timestamp <= ?",
        [start, end],
        |row| row.get(0),
    )?;
    Ok(count)
}

/// Count screenshots in a date range (records with screenshot_path)
fn count_screenshots_in_range(
    conn: &rusqlite::Connection,
    start: &str,
    end: &str,
) -> AppResult<i64> {
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM records WHERE timestamp >= ? AND timestamp <= ? AND screenshot_path IS NOT NULL AND screenshot_path != ''",
            [start, end],
            |row| row.get(0),
        )?;
    Ok(count)
}

/// Count sessions in a date range
fn count_sessions_in_range(conn: &rusqlite::Connection, start: &str, end: &str) -> AppResult<i64> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM sessions WHERE date >= ? AND date <= ?",
        [start[..10].to_string(), end[..10].to_string()],
        |row| row.get(0),
    )?;
    Ok(count)
}

/// Get AI analysis success rate (percentage of analyzed records)
fn get_analysis_success_rate(
    conn: &rusqlite::Connection,
    start: &str,
    end: &str,
) -> AppResult<f64> {
    let total: i64 = conn.query_row(
        "SELECT COUNT(*) FROM records WHERE timestamp >= ? AND timestamp <= ?",
        [start, end],
        |row| row.get(0),
    )?;

    if total == 0 {
        return Ok(0.0);
    }

    let analyzed: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM records WHERE timestamp >= ? AND timestamp <= ? AND analysis_status = 'analyzed'",
            [start, end],
            |row| row.get(0),
        )?;

    Ok((analyzed as f64) / (total as f64) * 100.0)
}

/// Get daily breakdown statistics for a date range
fn get_daily_breakdown(
    conn: &rusqlite::Connection,
    start: &str,
    end: &str,
) -> AppResult<Vec<DailyStatistic>> {
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
) -> AppResult<Statistics> {
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
                .ok_or_else(|| AppError::validation("custom_start is required for custom range"))?;
            let end_str = custom_end
                .clone()
                .ok_or_else(|| AppError::validation("custom_end is required for custom range"))?;

            // Validate and parse dates
            let _ = parse_date(&start_str)?;
            let _ = parse_date(&end_str)?;

            let start = format!("{}T00:00:00", start_str);
            let end = format!("{}T23:59:59.999", end_str);
            (start, end, format!("{} 至 {}", start_str, end_str))
        }
        _ => {
            return Err(AppError::validation(format!(
                "Invalid range_type: {}. Use: today, week, month, custom",
                range_type
            )))
        }
    };

    let db = DB_CONNECTION.lock()?;
    let conn = db
        .as_ref()
        .ok_or_else(|| AppError::database("Database not initialized"))?;

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

/// ANALYTICS-001: Get productivity trend data for week-over-week or month-over-month comparison
///
/// # Arguments
/// * `comparison_type` - One of: "week" (this week vs last week), "month" (this month vs last month)
#[command]
pub async fn get_productivity_trend(comparison_type: String) -> Result<ProductivityTrend, String> {
    let comparison_type_lower = comparison_type.to_lowercase();

    // Determine current and previous period ranges
    let (current_start, current_end, current_label, previous_start, previous_end, previous_label) =
        match comparison_type_lower.as_str() {
            "week" => {
                let (current_s, current_e) = get_week_range();
                let (previous_s, previous_e) = get_previous_week_range();
                (
                    current_s,
                    current_e,
                    "本周".to_string(),
                    previous_s,
                    previous_e,
                    "上周".to_string(),
                )
            }
            "month" => {
                let (current_s, current_e) = get_month_range();
                let (previous_s, previous_e) = get_previous_month_range();
                (
                    current_s,
                    current_e,
                    "本月".to_string(),
                    previous_s,
                    previous_e,
                    "上月".to_string(),
                )
            }
            _ => {
                return Err(format!(
                    "Invalid comparison_type: {}. Use: 'week' or 'month'",
                    comparison_type
                ))
            }
        };

    let db = DB_CONNECTION
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;
    let conn = db.as_ref().ok_or("Database not initialized")?;

    // Get counts for current period
    let current_screenshot_count = count_screenshots_in_range(conn, &current_start, &current_end)?;
    let current_record_count = count_records_in_range(conn, &current_start, &current_end)?;

    // Get counts for previous period
    let previous_screenshot_count =
        count_screenshots_in_range(conn, &previous_start, &previous_end)?;
    let previous_record_count = count_records_in_range(conn, &previous_start, &previous_end)?;

    // Calculate comparison metrics
    let screenshot_comparison =
        calculate_comparison(current_screenshot_count, previous_screenshot_count);
    let record_comparison = calculate_comparison(current_record_count, previous_record_count);

    // Get daily trend for current period
    let daily_trend = get_daily_trend_for_period(conn, &current_start, &current_end)?;

    // Get peak hours
    let peak_hours = get_peak_hours(conn, &current_start, &current_end)?;

    // Calculate average daily records
    let days_count = daily_trend.len() as f64;
    let average_daily_records = if days_count > 0.0 {
        current_record_count as f64 / days_count
    } else {
        0.0
    };

    Ok(ProductivityTrend {
        comparison_type: comparison_type_lower,
        current_period: DateRange {
            start: current_start,
            end: current_end,
            label: current_label,
        },
        previous_period: DateRange {
            start: previous_start,
            end: previous_end,
            label: previous_label,
        },
        screenshot_comparison,
        record_comparison,
        daily_trend,
        peak_hours,
        average_daily_records,
    })
}

/// Calculate comparison between current and previous period
fn calculate_comparison(current: i64, previous: i64) -> PeriodComparison {
    let change_percent = if previous == 0 {
        if current == 0 {
            0.0
        } else {
            100.0 // If previous was 0 and current is not 0, it's a 100% increase
        }
    } else {
        ((current - previous) as f64 / previous as f64) * 100.0
    };

    let trend = if change_percent > 5.0 {
        "up".to_string()
    } else if change_percent < -5.0 {
        "down".to_string()
    } else {
        "stable".to_string()
    };

    PeriodComparison {
        current_total: current,
        previous_total: previous,
        change_percent,
        trend,
    }
}

/// Get daily trend data for a period
fn get_daily_trend_for_period(
    conn: &rusqlite::Connection,
    start: &str,
    end: &str,
) -> Result<Vec<DailyTrendPoint>, String> {
    let start_date = parse_date(&start[..10])?;
    let end_date = parse_date(&end[..10])?;

    let mut result = Vec::new();
    let mut current_date = start_date;

    while current_date <= end_date {
        let date_str = current_date.format("%Y-%m-%d").to_string();
        let day_start = format!("{}T00:00:00", date_str);
        let day_end = format!("{}T23:59:59.999", date_str);

        let screenshot_count = count_screenshots_in_range(conn, &day_start, &day_end).unwrap_or(0);
        let record_count = count_records_in_range(conn, &day_start, &day_end).unwrap_or(0);

        result.push(DailyTrendPoint {
            date: date_str,
            screenshot_count,
            record_count,
        });

        current_date += chrono::Duration::days(1);
    }

    Ok(result)
}

/// Get peak hours (top 5 busiest hours)
fn get_peak_hours(
    conn: &rusqlite::Connection,
    start: &str,
    end: &str,
) -> Result<Vec<HourlyDistribution>, String> {
    // Query to get hour distribution
    let mut stmt = conn
        .prepare(
            "SELECT CAST(strftime('%H', timestamp) AS INTEGER) as hour, COUNT(*) as count
         FROM records
         WHERE timestamp >= ? AND timestamp <= ?
         GROUP BY hour
         ORDER BY count DESC
         LIMIT 5",
        )
        .map_err(|e| format!("Failed to prepare query: {}", e))?;

    let rows = stmt
        .query_map([start, end], |row| {
            let hour: i64 = row.get(0)?;
            let count: i64 = row.get(1)?;
            Ok((hour, count))
        })
        .map_err(|e| format!("Failed to query: {}", e))?;

    let mut distributions: Vec<HourlyDistribution> = Vec::new();
    let mut total_count: i64 = 0;

    for (hour, count) in rows.flatten() {
        total_count += count;
        distributions.push(HourlyDistribution {
            hour: hour as u32,
            count,
            percentage: 0.0, // Will calculate after we have total
        });
    }

    // Calculate percentages
    for dist in &mut distributions {
        dist.percentage = if total_count > 0 {
            (dist.count as f64 / total_count as f64) * 100.0
        } else {
            0.0
        };
    }

    Ok(distributions)
}

#[cfg(test)]
mod tests_statistics {
    use super::*;
    use serial_test::serial;

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
    #[serial]
    fn test_get_statistics_empty_database() {
        // Use unified test database setup
        crate::memory_storage::setup_test_db_with_schema();

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
    #[serial]
    fn test_get_statistics_with_data() {
        use rusqlite::params;

        // Use unified test database setup
        crate::memory_storage::setup_test_db_with_schema();

        // Use fixed date strings to avoid timezone-dependent failures on Windows CI
        // The test date 2026-03-27 is arbitrary but stable across all timezones
        let start = "2026-03-27T00:00:00".to_string();
        let end = "2026-03-27T23:59:59.999".to_string();
        let timestamp1 = "2026-03-27T10:00:00".to_string();
        let timestamp2 = "2026-03-27T11:00:00".to_string();
        let timestamp3 = "2026-03-27T14:00:00".to_string();

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
    #[serial]
    fn test_get_statistics_with_sessions() {
        use rusqlite::params;

        // Use unified test database setup
        crate::memory_storage::setup_test_db_with_schema();

        // Get today's range first to ensure consistent date
        let (start, end) = get_today_range();
        let date_part = &start[..10]; // "YYYY-MM-DD"

        // Insert test sessions
        {
            let db = crate::memory_storage::DB_CONNECTION.lock().unwrap();
            let conn = db.as_ref().unwrap();

            // Insert two sessions for today
            conn.execute(
                "INSERT INTO sessions (date, start_time, end_time, status) VALUES (?1, '09:00', '12:00', 'completed')",
                params![date_part],
            )
            .unwrap();

            conn.execute(
                "INSERT INTO sessions (date, start_time, end_time, status) VALUES (?1, '14:00', '18:00', 'completed')",
                params![date_part],
            )
            .unwrap();
        }

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
    #[serial]
    fn test_get_statistics_analysis_success_rate() {
        use rusqlite::params;

        // Use unified test database setup
        crate::memory_storage::setup_test_db_with_schema();

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
        assert!(result.unwrap_err().message.contains("Invalid range_type"));
    }

    #[test]
    fn test_get_statistics_custom_range_requires_dates() {
        let rt = tokio::runtime::Runtime::new().unwrap();

        // Custom range without dates should fail
        let result = rt.block_on(get_statistics("custom".to_string(), None, None));
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .message
            .contains("custom_start is required"));

        // Custom range with only start date should fail
        let result = rt.block_on(get_statistics(
            "custom".to_string(),
            Some("2026-03-01".to_string()),
            None,
        ));
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .message
            .contains("custom_end is required"));
    }

    // STAB-001 Task 4.2: Database connection reconnection tests
    #[test]
    fn test_check_connection_with_valid_connection() {
        use crate::memory_storage::schema::connection_is_valid;
        use crate::memory_storage::schema::init_test_database;
        use rusqlite::Connection;

        // Create in-memory database for testing
        let conn = Connection::open_in_memory().unwrap();
        init_test_database(&conn).unwrap();

        // Connection should be valid
        let is_valid = connection_is_valid(Some(&conn));
        assert!(is_valid);
    }

    #[test]
    fn test_check_connection_with_no_connection() {
        use crate::memory_storage::schema::connection_is_valid;

        // Should return false (needs reconnect)
        let is_valid = connection_is_valid(None);
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

// VAULT-001: Tests for multi-vault auto-selection
#[cfg(test)]
mod tests_vault_001 {
    use super::*;

    fn create_test_vaults() -> Settings {
        Settings {
            obsidian_vaults: Some(
                serde_json::to_string(&vec![
                    ObsidianVault {
                        name: "Work Vault".to_string(),
                        path: "/path/to/work".to_string(),
                        is_default: true,
                        window_patterns: Some(vec!["VS Code".to_string(), "Firefox".to_string()]),
                    },
                    ObsidianVault {
                        name: "Personal Vault".to_string(),
                        path: "/path/to/personal".to_string(),
                        is_default: false,
                        window_patterns: Some(vec!["Chrome".to_string()]),
                    },
                    ObsidianVault {
                        name: "No Pattern Vault".to_string(),
                        path: "/path/to/none".to_string(),
                        is_default: false,
                        window_patterns: None,
                    },
                ])
                .unwrap(),
            ),
            auto_detect_vault_by_window: None,
            ..Default::default()
        }
    }

    // Tests for get_vault_by_name
    #[test]
    fn test_get_vault_by_name_exists() {
        let settings = create_test_vaults();
        let vault = settings.get_vault_by_name("Work Vault");
        assert!(vault.is_some());
        let vault = vault.unwrap();
        assert_eq!(vault.name, "Work Vault");
        assert_eq!(vault.path, "/path/to/work");
        assert!(vault.is_default);
    }

    #[test]
    fn test_get_vault_by_name_not_found() {
        let settings = create_test_vaults();
        let vault = settings.get_vault_by_name("Non Existent Vault");
        assert!(vault.is_none());
    }

    #[test]
    fn test_get_vault_by_name_empty() {
        let settings = create_test_vaults();
        let vault = settings.get_vault_by_name("");
        assert!(vault.is_none());
    }

    // Tests for get_vault_by_window_title
    #[test]
    fn test_get_vault_by_window_title_exact_match() {
        let settings = create_test_vaults();
        let vault = settings.get_vault_by_window_title("VS Code - main.rs");
        assert!(vault.is_some());
        assert_eq!(vault.unwrap().name, "Work Vault");
    }

    #[test]
    fn test_get_vault_by_window_title_partial_match() {
        let settings = create_test_vaults();
        // "firefox development" contains "Firefox" pattern
        let vault = settings.get_vault_by_window_title("Firefox Development - Browser");
        assert!(vault.is_some());
        assert_eq!(vault.unwrap().name, "Work Vault");
    }

    #[test]
    fn test_get_vault_by_window_title_no_match() {
        let settings = create_test_vaults();
        let vault = settings.get_vault_by_window_title("Some Random App");
        assert!(vault.is_none());
    }

    #[test]
    fn test_get_vault_by_window_title_empty_patterns_vault() {
        let settings = create_test_vaults();
        // "No Pattern Vault" has no patterns, should not match
        let vault = settings.get_vault_by_window_title("VS Code");
        // Should match "Work Vault" since "VS Code" has patterns
        assert!(vault.is_some());
        assert_eq!(vault.unwrap().name, "Work Vault");
    }

    #[test]
    fn test_get_vault_by_window_title_case_insensitive() {
        let settings = create_test_vaults();
        let vault = settings.get_vault_by_window_title("vs code editor");
        assert!(vault.is_some());
        assert_eq!(vault.unwrap().name, "Work Vault");
    }

    #[test]
    fn test_get_vault_by_window_title_multiple_matches() {
        let settings = create_test_vaults();
        // Both "VS Code" and "Firefox" are in Work Vault patterns
        let vault = settings.get_vault_by_window_title("VS Code - Firefox");
        // Should return the first match (Work Vault)
        assert!(vault.is_some());
        assert_eq!(vault.unwrap().name, "Work Vault");
    }

    // Tests for get_effective_vault
    #[test]
    fn test_get_effective_vault_explicit_name() {
        let settings = create_test_vaults();
        let result = settings.get_effective_vault(Some("Personal Vault"), false);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "/path/to/personal");
    }

    #[test]
    fn test_get_effective_vault_explicit_name_not_found() {
        let settings = create_test_vaults();
        let result = settings.get_effective_vault(Some("Non Existent"), false);
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("not found"));
    }

    #[test]
    fn test_get_effective_vault_empty_vaults_falls_back_to_default() {
        let settings = Settings {
            obsidian_vaults: None,
            obsidian_path: Some("/fallback/path".to_string()),
            auto_detect_vault_by_window: None,
            ..Default::default()
        };
        let result = settings.get_effective_vault(None, false);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "/fallback/path");
    }
}
