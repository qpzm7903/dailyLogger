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
    let api_key = settings.api_key.clone().ok_or("API Key not configured")?;
    // 日报生成优先使用 summary_model_name，未配置时回退到 model_name
    let model_name = settings
        .summary_model_name
        .clone()
        .filter(|s| !s.is_empty())
        .or_else(|| settings.model_name.clone())
        .unwrap_or_else(|| "gpt-4o".to_string());

    if api_key.is_empty() {
        return Err("API Key is empty".to_string());
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
    let response = client
        .post(&endpoint)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await
        .map_err(|e| {
            let elapsed_ms = start.elapsed().as_millis();
            tracing::error!(
                "{}",
                serde_json::json!({
                    "event": "llm_error",
                    "caller": "generate_daily_summary",
                    "error": format!("API request failed: {}", e),
                    "elapsed_ms": elapsed_ms,
                })
            );
            format!("API request failed: {}", e)
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
        }
    }

    fn create_settings_with_include_manual(include: bool) -> Settings {
        Settings {
            include_manual_records: Some(include),
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
}

/// Returns the default summary prompt template.
/// This is used when the user has not configured a custom prompt.
#[command]
pub fn get_default_summary_prompt() -> String {
    DEFAULT_SUMMARY_PROMPT.to_string()
}
