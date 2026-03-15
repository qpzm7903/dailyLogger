use chrono::Datelike;
use std::path::PathBuf;
use tauri::command;

use crate::memory_storage::{self, Record, Settings};

const DEFAULT_SUMMARY_PROMPT: &str = r#"你是一个工作日志助手。请根据以下今日工作记录，生成一份结构化的 Markdown 格式日报。

要求：
1. 按时间顺序组织
2. 提取关键工作内容和技术关键词
3. 总结今日工作成果和遇到的问题
4. 输出纯 Markdown 格式，不要有其他说明文字

今日记录：
{records}

请生成日报："#;

/// Default title format for daily summaries
pub const DEFAULT_TITLE_FORMAT: &str = "工作日报 - {date}";

/// Default prompt template for weekly reports
const DEFAULT_WEEKLY_REPORT_PROMPT: &str = r#"你是一个工作日志助手。请根据以下本周工作记录，生成一份结构化的 Markdown 格式周报。

要求：
1. 按日期分组展示工作内容
2. 提取本周关键成果和技术亮点
3. 总结遇到的问题和解决方案
4. 列出下周待跟进事项
5. 输出纯 Markdown 格式，不要有其他说明文字

本周记录：
{records}

请生成周报："#;

/// Get the default weekly report prompt
pub fn get_default_weekly_report_prompt() -> String {
    DEFAULT_WEEKLY_REPORT_PROMPT.to_string()
}

/// Default prompt template for monthly reports
const DEFAULT_MONTHLY_REPORT_PROMPT: &str = r#"你是一个工作日志助手。请根据以下本月工作记录，生成一份结构化的 Markdown 格式月报。

要求：
1. 按周分组展示工作内容
2. 提取本月关键成果和技术亮点
3. 总结遇到的问题和解决方案
4. 分析月度工作趋势（哪些方面投入更多时间）
5. 列出下月待跟进事项
6. 输出纯 Markdown 格式，不要有其他说明文字

本月记录：
{records}

请生成月报："#;

/// Default prompt template for custom period reports - REPORT-003
const DEFAULT_CUSTOM_REPORT_PROMPT: &str = r#"你是一个工作日志助手。请根据以下指定时间段的工作记录，生成一份结构化的 Markdown 格式报告。

要求：
1. 按日期分组展示工作内容
2. 提取该时间段的关键成果和技术亮点
3. 总结遇到的问题和解决方案
4. 列出后续待跟进事项
5. 输出纯 Markdown 格式，不要有其他说明文字

时间段：{start_date} 至 {end_date}
记录：
{records}

请生成报告："#;

/// Get the default monthly report prompt
pub fn get_default_monthly_report_prompt() -> String {
    DEFAULT_MONTHLY_REPORT_PROMPT.to_string()
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

/// Format records into a string for the summary prompt.
/// Each record is formatted as: "- [HH:MM] 🖥️/⚡ source: content"
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

            format!("- [{}] {}: {}", time, source, r.content)
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

#[command]
pub async fn generate_daily_summary() -> Result<String, String> {
    // CORE-007: Check network status before attempting API call
    if !crate::network_status::is_online() {
        // Queue the task for later processing
        let _ = crate::offline_queue::enqueue_task(
            &crate::offline_queue::OfflineTaskType::DailySummary,
            "{}",
            None,
        );
        return Err("当前处于离线状态，日报生成已加入队列，网络恢复后将自动处理".to_string());
    }

    let settings = memory_storage::get_settings_sync()
        .map_err(|e| format!("Failed to get settings: {}", e))?;

    let obsidian_path = settings
        .obsidian_path
        .clone()
        .ok_or("Obsidian path not configured")?;

    if obsidian_path.is_empty() {
        return Err("Obsidian path is empty".to_string());
    }

    let api_base_url = settings
        .api_base_url
        .clone()
        .ok_or("API Base URL not configured")?;
    let api_key = settings.api_key.clone().unwrap_or_default();
    // 日报生成优先使用 summary_model_name，未配置时回退到 model_name
    let model_name = settings
        .summary_model_name
        .clone()
        .filter(|s| !s.is_empty())
        .or_else(|| settings.model_name.clone())
        .unwrap_or_else(|| "gpt-4o".to_string());

    // Check if this is an Ollama endpoint (doesn't require API Key)
    let is_ollama = crate::ollama::is_ollama_endpoint(&api_base_url);

    // For non-Ollama endpoints, API Key is required
    if !is_ollama && api_key.is_empty() {
        return Err("API Key is required for non-Ollama endpoints".to_string());
    }

    // Get all today's records
    let all_records = memory_storage::get_all_today_records_for_summary()
        .map_err(|e| format!("Failed to get records: {}", e))?;

    // Filter records based on include_manual_records setting
    let records = filter_records_by_settings(all_records, &settings);

    if records.is_empty() {
        return Err("No records for today after filtering".to_string());
    }

    // Format records for summary
    let records_text = format_records_for_summary(&records);

    let prompt_template = settings
        .summary_prompt
        .as_deref()
        .filter(|s| !s.is_empty())
        .unwrap_or(DEFAULT_SUMMARY_PROMPT);

    let prompt = prompt_template.replace("{records}", &records_text);

    let client = reqwest::Client::new();

    let request_body = serde_json::json!({
        "model": model_name,
        "messages": [
            {
                "role": "user",
                "content": prompt
            }
        ],
        "max_tokens": 2000
    });

    let masked_key = crate::mask_api_key(&api_key);
    let endpoint = format!("{}/chat/completions", api_base_url);
    tracing::info!(
        "{}",
        serde_json::json!({
            "event": "llm_request",
            "caller": "generate_daily_summary",
            "endpoint": endpoint,
            "model": model_name,
            "max_tokens": 2000,
            "api_key_masked": masked_key,
            "has_image": false,
            "prompt": prompt,
            "records_count": records.len(),
        })
    );

    let start = std::time::Instant::now();
    let mut request = client
        .post(&endpoint)
        .header("Content-Type", "application/json")
        .json(&request_body);

    // Only add Authorization header if API key is provided (not required for Ollama)
    if !api_key.is_empty() {
        request = request.header("Authorization", format!("Bearer {}", api_key));
    }

    let response = request.send().await.map_err(|e| {
        let elapsed_ms = start.elapsed().as_millis();
        let error_msg = crate::ollama::format_connection_error(&e.to_string(), is_ollama);
        tracing::error!(
            "{}",
            serde_json::json!({
                "event": "llm_error",
                "caller": "generate_daily_summary",
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
                "caller": "generate_daily_summary",
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

    let summary = response_json["choices"][0]["message"]["content"]
        .as_str()
        .ok_or("No content in response")?;

    tracing::info!(
        "{}",
        serde_json::json!({
            "event": "llm_response",
            "caller": "generate_daily_summary",
            "status": 200,
            "elapsed_ms": elapsed_ms,
            "usage": response_json.get("usage"),
            "model": response_json.get("model"),
            "response_id": response_json.get("id"),
            "content": summary,
        })
    );

    // Generate filename based on title format setting
    let filename = generate_summary_filename(&settings);

    let output_dir = PathBuf::from(&obsidian_path);
    std::fs::create_dir_all(&output_dir)
        .map_err(|e| format!("Failed to create output directory: {}", e))?;

    let output_path = output_dir.join(&filename);
    std::fs::write(&output_path, summary).map_err(|e| format!("Failed to write summary: {}", e))?;

    let path_str = output_path.to_string_lossy().to_string();

    let mut updated_settings = settings.clone();
    updated_settings.last_summary_path = Some(path_str.clone());
    memory_storage::save_settings_sync(&updated_settings)
        .map_err(|e| format!("Failed to update settings: {}", e))?;

    tracing::info!("Daily summary generated: {}", path_str);

    Ok(path_str)
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
            custom_report_prompt: None,
            last_custom_report_path: None,
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
            diff >= 89 && diff <= 92,
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

    // NOTE: Performance benchmark tests moved to dedicated `mod benchmarks` below (CORE-008)
}

/// Returns the default summary prompt template.
/// This is used when the user has not configured a custom prompt.
#[command]
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
#[command]
pub async fn generate_weekly_report() -> Result<String, String> {
    // CORE-007: Check network status before attempting API call
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

    let obsidian_path = settings
        .obsidian_path
        .clone()
        .ok_or("Obsidian path not configured")?;

    if obsidian_path.is_empty() {
        return Err("Obsidian path is empty".to_string());
    }

    let api_base_url = settings
        .api_base_url
        .clone()
        .ok_or("API Base URL not configured")?;
    let api_key = settings.api_key.clone().unwrap_or_default();

    // Use summary_model_name for weekly report, fallback to model_name
    let model_name = settings
        .summary_model_name
        .clone()
        .filter(|s| !s.is_empty())
        .or_else(|| settings.model_name.clone())
        .unwrap_or_else(|| "gpt-4o".to_string());

    // Check if this is an Ollama endpoint
    let is_ollama = crate::ollama::is_ollama_endpoint(&api_base_url);

    if !is_ollama && api_key.is_empty() {
        return Err("API Key is required for non-Ollama endpoints".to_string());
    }

    // Get week start day from settings (0=Monday by default)
    let week_start_day = settings.weekly_report_day.unwrap_or(0);

    // Get all records for this week
    let all_records = memory_storage::get_week_records_sync(week_start_day)
        .map_err(|e| format!("Failed to get week records: {}", e))?;

    // Filter records based on include_manual_records setting
    let records = filter_records_by_settings(all_records, &settings);

    if records.is_empty() {
        return Err("本周无记录".to_string());
    }

    // Format records for summary
    let records_text = format_records_for_summary(&records);

    let prompt_template = settings
        .weekly_report_prompt
        .as_deref()
        .filter(|s| !s.is_empty())
        .unwrap_or(DEFAULT_WEEKLY_REPORT_PROMPT);

    let prompt = prompt_template.replace("{records}", &records_text);

    let client = reqwest::Client::new();

    let request_body = serde_json::json!({
        "model": model_name,
        "messages": [
            {
                "role": "user",
                "content": prompt
            }
        ],
        "max_tokens": 3000
    });

    let masked_key = crate::mask_api_key(&api_key);
    let endpoint = format!("{}/chat/completions", api_base_url);
    tracing::info!(
        "{}",
        serde_json::json!({
            "event": "llm_request",
            "caller": "generate_weekly_report",
            "endpoint": endpoint,
            "model": model_name,
            "max_tokens": 3000,
            "api_key_masked": masked_key,
            "has_image": false,
            "prompt": prompt,
            "records_count": records.len(),
        })
    );

    let start = std::time::Instant::now();
    let mut request = client
        .post(&endpoint)
        .header("Content-Type", "application/json")
        .json(&request_body);

    if !api_key.is_empty() {
        request = request.header("Authorization", format!("Bearer {}", api_key));
    }

    let response = request.send().await.map_err(|e| {
        let elapsed_ms = start.elapsed().as_millis();
        let error_msg = crate::ollama::format_connection_error(&e.to_string(), is_ollama);
        tracing::error!(
            "{}",
            serde_json::json!({
                "event": "llm_error",
                "caller": "generate_weekly_report",
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
                "caller": "generate_weekly_report",
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

    let summary = response_json["choices"][0]["message"]["content"]
        .as_str()
        .ok_or("No content in response")?;

    tracing::info!(
        "{}",
        serde_json::json!({
            "event": "llm_response",
            "caller": "generate_weekly_report",
            "status": 200,
            "elapsed_ms": elapsed_ms,
            "usage": response_json.get("usage"),
            "model": response_json.get("model"),
            "response_id": response_json.get("id"),
            "content": summary,
        })
    );

    // Generate filename based on week dates (use same week_start_day as data query)
    let filename = generate_weekly_report_filename(week_start_day);

    let output_dir = PathBuf::from(&obsidian_path);
    std::fs::create_dir_all(&output_dir)
        .map_err(|e| format!("Failed to create output directory: {}", e))?;

    let output_path = output_dir.join(&filename);
    std::fs::write(&output_path, summary)
        .map_err(|e| format!("Failed to write weekly report: {}", e))?;

    let path_str = output_path.to_string_lossy().to_string();

    // Save last weekly report path to settings (separate from daily summary path)
    let mut updated_settings = settings.clone();
    updated_settings.last_weekly_report_path = Some(path_str.clone());
    memory_storage::save_settings_sync(&updated_settings)
        .map_err(|e| format!("Failed to update settings: {}", e))?;

    tracing::info!("Weekly report generated: {}", path_str);

    Ok(path_str)
}

/// Generate monthly report - REPORT-002
#[command]
pub async fn generate_monthly_report() -> Result<String, String> {
    // CORE-007: Check network status before attempting API call
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

    let obsidian_path = settings
        .obsidian_path
        .clone()
        .ok_or("Obsidian path not configured")?;

    if obsidian_path.is_empty() {
        return Err("Obsidian path is empty".to_string());
    }

    let api_base_url = settings
        .api_base_url
        .clone()
        .ok_or("API Base URL not configured")?;
    let api_key = settings.api_key.clone().unwrap_or_default();

    // Use summary_model_name for monthly report, fallback to model_name
    let model_name = settings
        .summary_model_name
        .clone()
        .filter(|s| !s.is_empty())
        .or_else(|| settings.model_name.clone())
        .unwrap_or_else(|| "gpt-4o".to_string());

    // Check if this is an Ollama endpoint
    let is_ollama = crate::ollama::is_ollama_endpoint(&api_base_url);

    if !is_ollama && api_key.is_empty() {
        return Err("API Key is required for non-Ollama endpoints".to_string());
    }

    // Get all records for this month
    let all_records = memory_storage::get_month_records_sync()
        .map_err(|e| format!("Failed to get month records: {}", e))?;

    // Filter records based on include_manual_records setting
    let records = filter_records_by_settings(all_records, &settings);

    if records.is_empty() {
        return Err("本月无记录".to_string());
    }

    // Format records for summary (use the week-grouped format for trend analysis)
    let records_text = format_records_by_week(&records);

    let prompt_template = settings
        .monthly_report_prompt
        .as_deref()
        .filter(|s| !s.is_empty())
        .unwrap_or(DEFAULT_MONTHLY_REPORT_PROMPT);

    let prompt = prompt_template.replace("{records}", &records_text);

    let client = reqwest::Client::new();

    let request_body = serde_json::json!({
        "model": model_name,
        "messages": [
            {
                "role": "user",
                "content": prompt
            }
        ],
        "max_tokens": 4000
    });

    let masked_key = crate::mask_api_key(&api_key);
    let endpoint = format!("{}/chat/completions", api_base_url);
    tracing::info!(
        "{}",
        serde_json::json!({
            "event": "llm_request",
            "caller": "generate_monthly_report",
            "endpoint": endpoint,
            "model": model_name,
            "max_tokens": 4000,
            "api_key_masked": masked_key,
            "has_image": false,
            "prompt": prompt,
            "records_count": records.len(),
        })
    );

    let start = std::time::Instant::now();
    let mut request = client
        .post(&endpoint)
        .header("Content-Type", "application/json")
        .json(&request_body);

    if !api_key.is_empty() {
        request = request.header("Authorization", format!("Bearer {}", api_key));
    }

    let response = request.send().await.map_err(|e| {
        let elapsed_ms = start.elapsed().as_millis();
        let error_msg = crate::ollama::format_connection_error(&e.to_string(), is_ollama);
        tracing::error!(
            "{}",
            serde_json::json!({
                "event": "llm_error",
                "caller": "generate_monthly_report",
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
                "caller": "generate_monthly_report",
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

    let summary = response_json["choices"][0]["message"]["content"]
        .as_str()
        .ok_or("No content in response")?;

    tracing::info!(
        "{}",
        serde_json::json!({
            "event": "llm_response",
            "caller": "generate_monthly_report",
            "status": 200,
            "elapsed_ms": elapsed_ms,
            "usage": response_json.get("usage"),
            "model": response_json.get("model"),
            "response_id": response_json.get("id"),
            "content": summary,
        })
    );

    // Generate filename based on month
    let filename = generate_monthly_report_filename();

    let output_dir = PathBuf::from(&obsidian_path);
    std::fs::create_dir_all(&output_dir)
        .map_err(|e| format!("Failed to create output directory: {}", e))?;

    let output_path = output_dir.join(&filename);
    std::fs::write(&output_path, summary)
        .map_err(|e| format!("Failed to write monthly report: {}", e))?;

    let path_str = output_path.to_string_lossy().to_string();

    // Save last monthly report path to settings
    let mut updated_settings = settings.clone();
    updated_settings.last_summary_path = Some(path_str.clone());
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
#[command]
pub async fn generate_custom_report(
    start_date: String,
    end_date: String,
    report_name: Option<String>,
) -> Result<String, String> {
    // CORE-007: Check network status before attempting API call
    if !crate::network_status::is_online() {
        return Err("当前处于离线状态，报告生成需要网络连接。请检查网络连接后重试。".to_string());
    }

    // Validate date format
    let parsed_start = chrono::NaiveDate::parse_from_str(&start_date, "%Y-%m-%d")
        .map_err(|e| format!("无效的起始日期格式 (需要 YYYY-MM-DD): {}", e))?;
    let parsed_end = chrono::NaiveDate::parse_from_str(&end_date, "%Y-%m-%d")
        .map_err(|e| format!("无效的结束日期格式 (需要 YYYY-MM-DD): {}", e))?;

    if parsed_end < parsed_start {
        return Err("结束日期不能早于起始日期".to_string());
    }

    let settings = memory_storage::get_settings_sync()
        .map_err(|e| format!("Failed to get settings: {}", e))?;

    let obsidian_path = settings
        .obsidian_path
        .clone()
        .ok_or("Obsidian path not configured")?;

    if obsidian_path.is_empty() {
        return Err("Obsidian path is empty".to_string());
    }

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

    // Get records for the specified date range
    let all_records =
        memory_storage::get_records_by_date_range_sync(start_date.clone(), end_date.clone())
            .map_err(|e| format!("Failed to get records: {}", e))?;

    let records = filter_records_by_settings(all_records, &settings);

    if records.is_empty() {
        return Err("所选时间范围内无记录".to_string());
    }

    // Format records (use week-grouped format for longer periods, simple format for shorter)
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

    let client = reqwest::Client::new();

    let request_body = serde_json::json!({
        "model": model_name,
        "messages": [
            {
                "role": "user",
                "content": prompt
            }
        ],
        "max_tokens": 4000
    });

    let masked_key = crate::mask_api_key(&api_key);
    let endpoint = format!("{}/chat/completions", api_base_url);
    tracing::info!(
        "{}",
        serde_json::json!({
            "event": "llm_request",
            "caller": "generate_custom_report",
            "endpoint": endpoint,
            "model": model_name,
            "max_tokens": 4000,
            "api_key_masked": masked_key,
            "has_image": false,
            "prompt": prompt,
            "records_count": records.len(),
        })
    );

    let start = std::time::Instant::now();
    let mut request = client
        .post(&endpoint)
        .header("Content-Type", "application/json")
        .json(&request_body);

    if !api_key.is_empty() {
        request = request.header("Authorization", format!("Bearer {}", api_key));
    }

    let response = request.send().await.map_err(|e| {
        let elapsed_ms = start.elapsed().as_millis();
        let error_msg = crate::ollama::format_connection_error(&e.to_string(), is_ollama);
        tracing::error!(
            "{}",
            serde_json::json!({
                "event": "llm_error",
                "caller": "generate_custom_report",
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
                "caller": "generate_custom_report",
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

    let summary = response_json["choices"][0]["message"]["content"]
        .as_str()
        .ok_or("No content in response")?;

    tracing::info!(
        "{}",
        serde_json::json!({
            "event": "llm_response",
            "caller": "generate_custom_report",
            "status": 200,
            "elapsed_ms": elapsed_ms,
            "usage": response_json.get("usage"),
            "model": response_json.get("model"),
            "response_id": response_json.get("id"),
            "content": summary,
        })
    );

    let name = report_name.as_deref().unwrap_or("自定义报告");
    let filename = generate_custom_report_filename(name, &start_date, &end_date);

    let output_dir = PathBuf::from(&obsidian_path);
    std::fs::create_dir_all(&output_dir)
        .map_err(|e| format!("Failed to create output directory: {}", e))?;

    let output_path = output_dir.join(&filename);
    std::fs::write(&output_path, summary)
        .map_err(|e| format!("Failed to write custom report: {}", e))?;

    let path_str = output_path.to_string_lossy().to_string();

    // Save last custom report path to settings
    let mut updated_settings = settings.clone();
    updated_settings.last_custom_report_path = Some(path_str.clone());
    memory_storage::save_settings_sync(&updated_settings)
        .map_err(|e| format!("Failed to update settings: {}", e))?;

    tracing::info!("Custom report generated: {}", path_str);

    Ok(path_str)
}

// ── Performance benchmark tests (CORE-008 AC#3) ──

#[cfg(test)]
mod benchmarks {
    use super::*;
    use crate::memory_storage::Record;
    use std::time::Instant;

    fn create_test_record(source_type: &str, content: &str) -> Record {
        Record {
            id: 1,
            timestamp: chrono::Utc::now().to_rfc3339(),
            source_type: source_type.to_string(),
            content: content.to_string(),
            screenshot_path: None,
            monitor_info: None,
            tags: None,
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
            custom_report_prompt: None,
            last_custom_report_path: None,
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
            });
        }

        let start = Instant::now();
        let _result = filter_records_by_settings(records, &settings);
        let elapsed_ms = start.elapsed().as_millis();

        // Threshold: should complete in < 100ms
        assert!(
            elapsed_ms < 100,
            "filter_records_by_settings with 100 records took {}ms (threshold: 100ms)",
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

        // Threshold: 1000 iterations should complete in < 100ms
        assert!(
            elapsed_ms < 100,
            "1000 iterations of generate_summary_filename took {}ms (threshold: 100ms)",
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

        // Threshold: 10000 iterations should complete in < 100ms
        assert!(
            elapsed_ms < 100,
            "10000 iterations of format_summary_title took {}ms (threshold: 100ms)",
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
}
