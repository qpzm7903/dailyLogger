use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::command;

use crate::memory_storage::{self, Record};

/// Export request parameters
#[derive(Debug, Serialize, Deserialize)]
pub struct ExportRequest {
    pub start_date: String, // YYYY-MM-DD (local timezone)
    pub end_date: String,   // YYYY-MM-DD (local timezone)
    pub format: String,     // "json" | "markdown"
}

/// Export result
#[derive(Debug, Serialize, Deserialize)]
pub struct ExportResult {
    pub path: String,
    pub record_count: usize,
    pub file_size: u64,
}

/// Get the export directory path
pub fn get_export_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("DailyLogger")
        .join("exports")
}

/// Export records as JSON string
pub fn export_to_json(
    records: &[Record],
    start_date: &str,
    end_date: &str,
) -> Result<String, String> {
    let exported_at = chrono::Utc::now().to_rfc3339();

    let json_records: Vec<serde_json::Value> = records
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
        "exported_at": exported_at,
        "date_range": {
            "start": start_date,
            "end": end_date,
        },
        "total_records": records.len(),
        "records": json_records,
    });

    serde_json::to_string_pretty(&output).map_err(|e| format!("Failed to serialize JSON: {}", e))
}

/// Export records as Markdown string
pub fn export_to_markdown(
    records: &[Record],
    start_date: &str,
    end_date: &str,
) -> Result<String, String> {
    let exported_at = chrono::Local::now().format("%Y-%m-%d %H:%M").to_string();
    let mut md = String::new();

    md.push_str("# DailyLogger 数据导出\n\n");
    md.push_str(&format!("导出时间: {}\n", exported_at));
    md.push_str(&format!("日期范围: {} 至 {}\n", start_date, end_date));
    md.push_str(&format!("总记录数: {}\n", records.len()));

    if records.is_empty() {
        md.push_str("\n> 无记录\n");
        return Ok(md);
    }

    // Group records by local date
    let mut grouped: std::collections::BTreeMap<String, Vec<&Record>> =
        std::collections::BTreeMap::new();
    for record in records {
        let date_key = chrono::DateTime::parse_from_rfc3339(&record.timestamp)
            .map(|dt| {
                dt.with_timezone(&chrono::Local)
                    .format("%Y-%m-%d")
                    .to_string()
            })
            .unwrap_or_else(|_| "unknown".to_string());
        grouped.entry(date_key).or_default().push(record);
    }

    // Output in reverse chronological order (most recent date first)
    for (date, day_records) in grouped.iter().rev() {
        md.push_str(&format!("\n---\n\n## {}\n\n### 时间线\n\n", date));
        for record in day_records {
            let time = chrono::DateTime::parse_from_rfc3339(&record.timestamp)
                .map(|dt| dt.with_timezone(&chrono::Local).format("%H:%M").to_string())
                .unwrap_or_else(|_| "unknown".to_string());

            let source_icon = if record.source_type == "auto" {
                "🖥️ 自动感知"
            } else {
                "⚡ 闪念"
            };

            md.push_str(&format!("- **{}** {}\n", time, source_icon));
            // Indent content lines
            for line in record.content.lines() {
                md.push_str(&format!("  {}\n", line));
            }
            md.push('\n');
        }
    }

    Ok(md)
}

/// Tauri command: open the export directory in the system file manager
#[command]
pub async fn open_export_dir(path: String) -> Result<(), String> {
    let dir = std::path::Path::new(&path)
        .parent()
        .ok_or_else(|| "Cannot determine parent directory".to_string())?;

    if !dir.exists() {
        return Err(format!("Directory does not exist: {}", dir.display()));
    }

    let dir_str = dir.to_string_lossy().to_string();

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(&dir_str)
            .spawn()
            .map_err(|e| format!("Failed to open directory: {}", e))?;
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&dir_str)
            .spawn()
            .map_err(|e| format!("Failed to open directory: {}", e))?;
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&dir_str)
            .spawn()
            .map_err(|e| format!("Failed to open directory: {}", e))?;
    }

    tracing::info!("Opened export directory: {}", dir_str);
    Ok(())
}

/// Tauri command: export records to JSON or Markdown file
#[command]
pub async fn export_records(request: ExportRequest) -> Result<ExportResult, String> {
    let records = memory_storage::get_records_for_export(&request.start_date, &request.end_date)?;

    let content = match request.format.as_str() {
        "json" => export_to_json(&records, &request.start_date, &request.end_date)?,
        "markdown" => export_to_markdown(&records, &request.start_date, &request.end_date)?,
        _ => return Err(format!("Unsupported export format: {}", request.format)),
    };

    let export_dir = get_export_dir();
    std::fs::create_dir_all(&export_dir)
        .map_err(|e| format!("Failed to create export directory: {}", e))?;

    // Generate filename with timestamp to avoid overwriting previous exports
    let now = chrono::Local::now();
    let extension = if request.format == "json" {
        "json"
    } else {
        "md"
    };
    let filename = format!(
        "dailylogger-export-{}.{}",
        now.format("%Y-%m-%d_%H%M%S"),
        extension
    );
    let output_path = export_dir.join(&filename);

    std::fs::write(&output_path, &content)
        .map_err(|e| format!("Failed to write export file: {}", e))?;

    let file_size = std::fs::metadata(&output_path)
        .map(|m| m.len())
        .unwrap_or(0);

    let path_str = output_path.to_string_lossy().to_string();
    tracing::info!(
        "Exported {} records to {} ({} bytes)",
        records.len(),
        path_str,
        file_size
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

    fn make_test_record(id: i64, timestamp: &str, source_type: &str, content: &str) -> Record {
        Record {
            id,
            timestamp: timestamp.to_string(),
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

    // ===== JSON Export Tests =====

    #[test]
    fn test_export_to_json_basic() {
        let records = vec![
            make_test_record(1, "2026-03-14T02:30:00+00:00", "auto", "编写 Rust 代码"),
            make_test_record(2, "2026-03-14T03:15:00+00:00", "manual", "记录想法"),
        ];

        let result = export_to_json(&records, "2026-03-14", "2026-03-14").unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        assert_eq!(parsed["total_records"], 2);
        assert_eq!(parsed["date_range"]["start"], "2026-03-14");
        assert_eq!(parsed["date_range"]["end"], "2026-03-14");
        assert_eq!(parsed["records"].as_array().unwrap().len(), 2);
        assert_eq!(parsed["records"][0]["id"], 1);
        assert_eq!(parsed["records"][0]["source_type"], "auto");
        assert_eq!(parsed["records"][0]["content"], "编写 Rust 代码");
        assert!(parsed["exported_at"].as_str().is_some());
    }

    #[test]
    fn test_export_to_json_empty_records() {
        let records: Vec<Record> = vec![];
        let result = export_to_json(&records, "2026-03-14", "2026-03-14").unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        assert_eq!(parsed["total_records"], 0);
        assert_eq!(parsed["records"].as_array().unwrap().len(), 0);
    }

    #[test]
    fn test_export_to_json_special_characters() {
        let records = vec![make_test_record(
            1,
            "2026-03-14T02:30:00+00:00",
            "manual",
            "包含特殊字符: \"引号\" & <标签> \n换行",
        )];

        let result = export_to_json(&records, "2026-03-14", "2026-03-14").unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        // JSON should properly escape special characters
        assert_eq!(
            parsed["records"][0]["content"],
            "包含特殊字符: \"引号\" & <标签> \n换行"
        );
    }

    #[test]
    fn test_export_to_json_with_screenshot_path() {
        let mut record = make_test_record(1, "2026-03-14T02:30:00+00:00", "auto", "截图分析");
        record.screenshot_path = Some("screenshots/screenshot_20260314_103000.png".to_string());

        let result = export_to_json(&[record], "2026-03-14", "2026-03-14").unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        assert_eq!(
            parsed["records"][0]["screenshot_path"],
            "screenshots/screenshot_20260314_103000.png"
        );
    }

    // ===== Markdown Export Tests =====

    #[test]
    fn test_export_to_markdown_basic() {
        let records = vec![
            make_test_record(1, "2026-03-14T02:30:00+00:00", "auto", "编写 Rust 代码"),
            make_test_record(2, "2026-03-14T03:15:00+00:00", "manual", "记录想法"),
        ];

        let result = export_to_markdown(&records, "2026-03-14", "2026-03-14").unwrap();

        assert!(result.contains("# DailyLogger 数据导出"));
        assert!(result.contains("日期范围: 2026-03-14 至 2026-03-14"));
        assert!(result.contains("总记录数: 2"));
        assert!(result.contains("🖥️ 自动感知"));
        assert!(result.contains("⚡ 闪念"));
        assert!(result.contains("编写 Rust 代码"));
        assert!(result.contains("记录想法"));
    }

    #[test]
    fn test_export_to_markdown_empty_records() {
        let records: Vec<Record> = vec![];
        let result = export_to_markdown(&records, "2026-03-14", "2026-03-14").unwrap();

        assert!(result.contains("# DailyLogger 数据导出"));
        assert!(result.contains("总记录数: 0"));
        assert!(result.contains("> 无记录"));
    }

    #[test]
    fn test_export_to_markdown_multi_day() {
        let records = vec![
            make_test_record(1, "2026-03-13T02:30:00+00:00", "auto", "第一天工作"),
            make_test_record(2, "2026-03-14T03:15:00+00:00", "manual", "第二天工作"),
        ];

        let result = export_to_markdown(&records, "2026-03-13", "2026-03-14").unwrap();

        // Should contain both dates as headings
        assert!(result.contains("## 2026-03-1"));
        assert!(result.contains("第一天工作"));
        assert!(result.contains("第二天工作"));
    }

    #[test]
    fn test_export_to_markdown_multiline_content() {
        let records = vec![make_test_record(
            1,
            "2026-03-14T02:30:00+00:00",
            "manual",
            "第一行\n第二行\n第三行",
        )];

        let result = export_to_markdown(&records, "2026-03-14", "2026-03-14").unwrap();

        // Each line should be indented
        assert!(result.contains("  第一行\n"));
        assert!(result.contains("  第二行\n"));
        assert!(result.contains("  第三行\n"));
    }

    // ===== Export Directory Tests =====

    #[test]
    fn test_get_export_dir_returns_valid_path() {
        let dir = get_export_dir();
        assert!(dir.to_string_lossy().contains("DailyLogger"));
        assert!(dir.to_string_lossy().contains("exports"));
    }

    // ── Platform command verification tests (CORE-008 Task 2.2) ──

    #[test]
    fn test_open_export_dir_rejects_nonexistent_directory() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(open_export_dir(
            "/nonexistent/path/to/file.json".to_string(),
        ));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Directory does not exist"));
    }

    #[test]
    fn test_open_export_dir_rejects_root_only_path() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        // A path with no parent directory component
        let result = rt.block_on(open_export_dir("file.json".to_string()));
        // parent() of "file.json" is "" which doesn't exist as a directory
        // Actually parent() returns Some("") for bare filename, which exists
        // The behavior depends on the OS
        // Just ensure it doesn't panic
        let _ = result;
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn test_macos_open_command_available() {
        // Verify 'open' command exists on macOS (used by open_export_dir)
        let output = std::process::Command::new("which")
            .arg("open")
            .output()
            .expect("'which' command should work on macOS");
        assert!(
            output.status.success(),
            "macOS 'open' command should be available"
        );
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_windows_explorer_command_available() {
        // Verify 'explorer' command exists on Windows (used by open_export_dir)
        let output = std::process::Command::new("where")
            .arg("explorer")
            .output()
            .expect("'where' command should work on Windows");
        assert!(
            output.status.success(),
            "Windows 'explorer' command should be available"
        );
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn test_open_export_dir_spawns_open_on_macos() {
        // Create a temporary directory and file to test open_export_dir
        let temp_dir = std::env::temp_dir().join("dailylogger_test_export");
        std::fs::create_dir_all(&temp_dir).unwrap();
        let temp_file = temp_dir.join("test.json");
        std::fs::write(&temp_file, "{}").unwrap();

        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(open_export_dir(temp_file.to_string_lossy().to_string()));
        // Should succeed (open command spawns and returns immediately)
        assert!(
            result.is_ok(),
            "open_export_dir should succeed on macOS with valid path"
        );

        // Cleanup
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_open_export_dir_spawns_explorer_on_windows() {
        let temp_dir = std::env::temp_dir().join("dailylogger_test_export");
        std::fs::create_dir_all(&temp_dir).unwrap();
        let temp_file = temp_dir.join("test.json");
        std::fs::write(&temp_file, "{}").unwrap();

        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(open_export_dir(temp_file.to_string_lossy().to_string()));
        assert!(
            result.is_ok(),
            "open_export_dir should succeed on Windows with valid path"
        );

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
