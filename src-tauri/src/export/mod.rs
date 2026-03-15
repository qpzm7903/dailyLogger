use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::command;

use crate::memory_storage::{self, Record};

/// Export request parameters
#[derive(Debug, Serialize, Deserialize)]
pub struct ExportRequest {
    pub start_date: String, // YYYY-MM-DD
    pub end_date: String,   // YYYY-MM-DD
    pub format: String,     // "json" | "markdown"
}

/// Export result
#[derive(Debug, Serialize, Deserialize)]
pub struct ExportResult {
    pub path: String,
    pub record_count: usize,
    pub file_size: u64,
}

/// Get the default export directory
fn get_export_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("DailyLogger")
        .join("exports")
}

/// Export records to JSON format string
pub fn export_records_to_json(
    records: &[Record],
    start_date: &str,
    end_date: &str,
) -> Result<String, String> {
    let export_time = chrono::Utc::now().to_rfc3339();

    let records_json: Vec<serde_json::Value> = records
        .iter()
        .map(|r| {
            serde_json::json!({
                "id": r.id,
                "timestamp": r.timestamp,
                "source_type": r.source_type,
                "content": r.content,
                "screenshot_path": r.screenshot_path,
            })
        })
        .collect();

    let output = serde_json::json!({
        "exported_at": export_time,
        "date_range": {
            "start": start_date,
            "end": end_date,
        },
        "total_records": records.len(),
        "records": records_json,
    });

    serde_json::to_string_pretty(&output).map_err(|e| format!("Failed to serialize JSON: {}", e))
}

/// Export records to Markdown format string
pub fn export_records_to_markdown(
    records: &[Record],
    start_date: &str,
    end_date: &str,
) -> Result<String, String> {
    let export_time = chrono::Local::now().format("%Y-%m-%d %H:%M").to_string();

    let mut md = String::new();
    md.push_str("# DailyLogger 数据导出\n\n");
    md.push_str(&format!("导出时间: {}\n", export_time));
    md.push_str(&format!("日期范围: {} 至 {}\n", start_date, end_date));
    md.push_str(&format!("总记录数: {}\n", records.len()));
    md.push_str("\n---\n");

    if records.is_empty() {
        md.push_str("\n*该日期范围内无记录*\n");
        return Ok(md);
    }

    // Group records by local date (ascending)
    let mut grouped: std::collections::BTreeMap<String, Vec<&Record>> =
        std::collections::BTreeMap::new();

    for record in records {
        let date_str = chrono::DateTime::parse_from_rfc3339(&record.timestamp)
            .map(|dt| {
                dt.with_timezone(&chrono::Local)
                    .format("%Y-%m-%d")
                    .to_string()
            })
            .unwrap_or_else(|_| "unknown".to_string());
        grouped.entry(date_str).or_default().push(record);
    }

    for (date, day_records) in &grouped {
        md.push_str(&format!("\n## {}\n\n### 时间线\n\n", date));

        // Sort records within each day by timestamp ascending
        let mut sorted_records: Vec<&&Record> = day_records.iter().collect();
        sorted_records.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

        for record in sorted_records {
            let time = chrono::DateTime::parse_from_rfc3339(&record.timestamp)
                .map(|dt| dt.with_timezone(&chrono::Local).format("%H:%M").to_string())
                .unwrap_or_else(|_| "unknown".to_string());

            let source_icon = if record.source_type == "auto" {
                "🖥️ 自动感知"
            } else {
                "⚡ 闪念"
            };

            md.push_str(&format!("- **{}** {}\n", time, source_icon));
            md.push_str(&format!("  {}\n\n", record.content));
        }

        md.push_str("---\n");
    }

    Ok(md)
}

/// Tauri command: export records in specified format
#[command]
pub async fn export_records(request: ExportRequest) -> Result<ExportResult, String> {
    // Validate format
    if request.format != "json" && request.format != "markdown" {
        return Err(format!(
            "Unsupported export format: '{}'. Use 'json' or 'markdown'.",
            request.format
        ));
    }

    // Validate date range
    let start = chrono::NaiveDate::parse_from_str(&request.start_date, "%Y-%m-%d")
        .map_err(|e| format!("Invalid start_date format (expected YYYY-MM-DD): {}", e))?;
    let end = chrono::NaiveDate::parse_from_str(&request.end_date, "%Y-%m-%d")
        .map_err(|e| format!("Invalid end_date format (expected YYYY-MM-DD): {}", e))?;

    if start > end {
        return Err("Start date cannot be after end date".to_string());
    }

    // Get records for the date range
    let records = memory_storage::get_records_by_date_range_sync(
        request.start_date.clone(),
        request.end_date.clone(),
    )?;

    // Generate output content
    let (content, extension) = match request.format.as_str() {
        "json" => (
            export_records_to_json(&records, &request.start_date, &request.end_date)?,
            "json",
        ),
        "markdown" => (
            export_records_to_markdown(&records, &request.start_date, &request.end_date)?,
            "md",
        ),
        _ => unreachable!(),
    };

    // Create export directory
    let export_dir = get_export_dir();
    std::fs::create_dir_all(&export_dir)
        .map_err(|e| format!("Failed to create export directory: {}", e))?;

    // Generate filename
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let filename = format!("dailylogger-export-{}.{}", today, extension);
    let output_path = export_dir.join(&filename);

    // Write file
    std::fs::write(&output_path, &content)
        .map_err(|e| format!("Failed to write export file: {}", e))?;

    let file_size = std::fs::metadata(&output_path)
        .map(|m| m.len())
        .unwrap_or(0);

    let path_str = output_path.to_string_lossy().to_string();
    tracing::info!(
        "Exported {} records to {} ({})",
        records.len(),
        path_str,
        request.format
    );

    Ok(ExportResult {
        path: path_str,
        record_count: records.len(),
        file_size,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_record(id: i64, timestamp: &str, source_type: &str, content: &str) -> Record {
        Record {
            id,
            timestamp: timestamp.to_string(),
            source_type: source_type.to_string(),
            content: content.to_string(),
            screenshot_path: None,
            monitor_info: None,
            tags: None,
        }
    }

    // ── Tests for export_records_to_json ──

    #[test]
    fn json_export_contains_required_fields() {
        let records = vec![create_test_record(
            1,
            "2026-03-14T02:30:00+00:00",
            "auto",
            "Working on Rust code",
        )];

        let result = export_records_to_json(&records, "2026-03-07", "2026-03-14").unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        assert!(parsed["exported_at"].is_string());
        assert_eq!(parsed["date_range"]["start"], "2026-03-07");
        assert_eq!(parsed["date_range"]["end"], "2026-03-14");
        assert_eq!(parsed["total_records"], 1);
        assert!(parsed["records"].is_array());
    }

    #[test]
    fn json_export_record_structure() {
        let records = vec![create_test_record(
            42,
            "2026-03-14T02:30:00+00:00",
            "auto",
            "Writing tests",
        )];

        let result = export_records_to_json(&records, "2026-03-14", "2026-03-14").unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        let record = &parsed["records"][0];
        assert_eq!(record["id"], 42);
        assert_eq!(record["timestamp"], "2026-03-14T02:30:00+00:00");
        assert_eq!(record["source_type"], "auto");
        assert_eq!(record["content"], "Writing tests");
        assert!(record["screenshot_path"].is_null());
    }

    #[test]
    fn json_export_empty_records() {
        let records: Vec<Record> = vec![];

        let result = export_records_to_json(&records, "2026-03-07", "2026-03-14").unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        assert_eq!(parsed["total_records"], 0);
        assert_eq!(parsed["records"].as_array().unwrap().len(), 0);
    }

    #[test]
    fn json_export_special_characters() {
        let records = vec![create_test_record(
            1,
            "2026-03-14T02:30:00+00:00",
            "manual",
            "Contains \"quotes\" and <html> & special chars",
        )];

        let result = export_records_to_json(&records, "2026-03-14", "2026-03-14").unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        assert_eq!(
            parsed["records"][0]["content"],
            "Contains \"quotes\" and <html> & special chars"
        );
    }

    #[test]
    fn json_export_multiple_records() {
        let records = vec![
            create_test_record(1, "2026-03-14T02:30:00+00:00", "auto", "Record 1"),
            create_test_record(2, "2026-03-14T05:00:00+00:00", "manual", "Record 2"),
            create_test_record(3, "2026-03-15T01:00:00+00:00", "auto", "Record 3"),
        ];

        let result = export_records_to_json(&records, "2026-03-14", "2026-03-15").unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        assert_eq!(parsed["total_records"], 3);
        assert_eq!(parsed["records"].as_array().unwrap().len(), 3);
    }

    // ── Tests for export_records_to_markdown ──

    #[test]
    fn markdown_export_contains_header() {
        let records: Vec<Record> = vec![];

        let result = export_records_to_markdown(&records, "2026-03-07", "2026-03-14").unwrap();

        assert!(result.contains("# DailyLogger 数据导出"));
        assert!(result.contains("导出时间:"));
        assert!(result.contains("日期范围: 2026-03-07 至 2026-03-14"));
        assert!(result.contains("总记录数: 0"));
    }

    #[test]
    fn markdown_export_empty_records_message() {
        let records: Vec<Record> = vec![];

        let result = export_records_to_markdown(&records, "2026-03-07", "2026-03-14").unwrap();

        assert!(result.contains("*该日期范围内无记录*"));
    }

    #[test]
    fn markdown_export_groups_by_date() {
        let records = vec![
            create_test_record(1, "2026-03-14T02:30:00+00:00", "auto", "Day 1 record"),
            create_test_record(2, "2026-03-15T05:00:00+00:00", "manual", "Day 2 record"),
        ];

        let result = export_records_to_markdown(&records, "2026-03-14", "2026-03-15").unwrap();

        // Should have date headers (the exact local date depends on timezone,
        // but the structure should have ## date headers)
        assert!(result.contains("## 20"));
        assert!(result.contains("### 时间线"));
    }

    #[test]
    fn markdown_export_auto_source_icon() {
        let records = vec![create_test_record(
            1,
            "2026-03-14T02:30:00+00:00",
            "auto",
            "Auto content",
        )];

        let result = export_records_to_markdown(&records, "2026-03-14", "2026-03-14").unwrap();

        assert!(result.contains("🖥️ 自动感知"));
        assert!(result.contains("Auto content"));
    }

    #[test]
    fn markdown_export_manual_source_icon() {
        let records = vec![create_test_record(
            1,
            "2026-03-14T02:30:00+00:00",
            "manual",
            "Manual note",
        )];

        let result = export_records_to_markdown(&records, "2026-03-14", "2026-03-14").unwrap();

        assert!(result.contains("⚡ 闪念"));
        assert!(result.contains("Manual note"));
    }

    #[test]
    fn markdown_export_total_records_count() {
        let records = vec![
            create_test_record(1, "2026-03-14T02:30:00+00:00", "auto", "R1"),
            create_test_record(2, "2026-03-14T05:00:00+00:00", "manual", "R2"),
        ];

        let result = export_records_to_markdown(&records, "2026-03-14", "2026-03-14").unwrap();

        assert!(result.contains("总记录数: 2"));
    }

    // ── Tests for ExportRequest validation ──

    #[test]
    fn export_request_deserializes_correctly() {
        let json = r#"{"start_date":"2026-03-07","end_date":"2026-03-14","format":"json"}"#;
        let req: ExportRequest = serde_json::from_str(json).unwrap();

        assert_eq!(req.start_date, "2026-03-07");
        assert_eq!(req.end_date, "2026-03-14");
        assert_eq!(req.format, "json");
    }

    #[test]
    fn export_result_serializes_correctly() {
        let result = ExportResult {
            path: "/tmp/export.json".to_string(),
            record_count: 42,
            file_size: 1024,
        };

        let json = serde_json::to_string(&result).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed["path"], "/tmp/export.json");
        assert_eq!(parsed["record_count"], 42);
        assert_eq!(parsed["file_size"], 1024);
    }
}
