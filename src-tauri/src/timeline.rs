//! Timeline visualization module.
//!
//! Provides data structures and API for generating timeline views of daily records.
//! Records are grouped by hour for easy visualization.

use chrono::{DateTime, Local, Timelike};
use rusqlite::params;
use serde::{Deserialize, Serialize};
use tauri::command;

use crate::memory_storage::{Record, DB_CONNECTION};

/// A single event on the timeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineEvent {
    /// The original record
    pub record: Record,
    /// Hour of the day (0-23)
    pub hour: u32,
    /// Formatted time string (e.g., "14:30")
    pub time_str: String,
    /// Event type for styling (auto, manual)
    pub event_type: String,
    /// Short preview of content (truncated)
    pub preview: String,
}

/// A group of events within the same hour.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineHourGroup {
    /// Hour of the day (0-23)
    pub hour: u32,
    /// Display label (e.g., "14:00 - 15:00")
    pub label: String,
    /// Events in this hour
    pub events: Vec<TimelineEvent>,
    /// Count of events
    pub count: usize,
}

/// Complete timeline data for a day.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineData {
    /// Date string (YYYY-MM-DD)
    pub date: String,
    /// Hour groups with events
    pub hour_groups: Vec<TimelineHourGroup>,
    /// Total event count
    pub total_events: usize,
    /// Hours with activity count
    pub active_hours: usize,
    /// Work time estimate (hours)
    pub work_time_estimate: f64,
}

/// Generate preview text from content.
fn generate_preview(content: &str, max_len: usize) -> String {
    // Try to extract meaningful preview from JSON content
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(content) {
        // For screenshot analysis, get the summary
        if let Some(summary) = json.get("summary").and_then(|s| s.as_str()) {
            let truncated = if summary.len() > max_len {
                format!("{}...", &summary[..max_len])
            } else {
                summary.to_string()
            };
            return truncated;
        }
        // For manual notes, get the note text
        if let Some(note) = json.get("note").and_then(|n| n.as_str()) {
            let truncated = if note.len() > max_len {
                format!("{}...", &note[..max_len])
            } else {
                note.to_string()
            };
            return truncated;
        }
    }

    // Fallback: truncate raw content
    if content.len() > max_len {
        format!("{}...", &content[..max_len])
    } else {
        content.to_string()
    }
}

/// Parse timestamp string to DateTime.
fn parse_timestamp(timestamp: &str) -> Result<DateTime<Local>, String> {
    DateTime::parse_from_rfc3339(timestamp)
        .map(|dt| dt.with_timezone(&Local))
        .map_err(|e| format!("Failed to parse timestamp: {}", e))
}

/// Convert records to timeline events.
fn records_to_events(records: Vec<Record>) -> Vec<TimelineEvent> {
    records
        .into_iter()
        .filter_map(|record| {
            let dt = parse_timestamp(&record.timestamp).ok()?;
            let hour = dt.hour();
            let time_str = dt.format("%H:%M").to_string();
            let preview = generate_preview(&record.content, 80);

            Some(TimelineEvent {
                record,
                hour,
                time_str,
                event_type: "auto".to_string(), // Will be set based on source_type
                preview,
            })
        })
        .collect()
}

/// Group events by hour.
fn group_by_hour(events: Vec<TimelineEvent>) -> Vec<TimelineHourGroup> {
    let mut hour_map: std::collections::BTreeMap<u32, Vec<TimelineEvent>> =
        std::collections::BTreeMap::new();

    for event in events {
        let event_type = event.record.source_type.clone();
        let mut event = event;
        event.event_type = event_type;
        hour_map.entry(event.hour).or_default().push(event);
    }

    hour_map
        .into_iter()
        .map(|(hour, events)| {
            let label = format!("{:02}:00 - {:02}:00", hour, (hour + 1) % 24);
            let count = events.len();
            TimelineHourGroup {
                hour,
                label,
                events,
                count,
            }
        })
        .collect()
}

/// Calculate work time estimate based on event distribution.
fn calculate_work_time_estimate(hour_groups: &[TimelineHourGroup]) -> f64 {
    if hour_groups.is_empty() {
        return 0.0;
    }

    // Simple heuristic: count active hours with a minimum events threshold
    let active_hours = hour_groups
        .iter()
        .filter(|g| g.count >= 2) // At least 2 events per hour to count as active
        .count();

    // Estimate 0.75 hours per active hour (accounting for gaps)
    active_hours as f64 * 0.75
}

/// Get timeline data for a specific date.
pub fn get_timeline_data_for_date(date: &str) -> Result<TimelineData, String> {
    let db = DB_CONNECTION
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;
    let conn = db.as_ref().ok_or("Database not initialized")?;

    // Parse date and create time range
    let target_date = chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d")
        .map_err(|e| format!("Invalid date format: {}", e))?;

    let start_time = target_date
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_local_timezone(Local)
        .unwrap()
        .with_timezone(&chrono::Utc)
        .to_rfc3339();

    let end_time = target_date
        .and_hms_opt(23, 59, 59)
        .unwrap()
        .and_local_timezone(Local)
        .unwrap()
        .with_timezone(&chrono::Utc)
        .to_rfc3339();

    // Query records for the date
    let mut stmt = conn
        .prepare(
            "SELECT id, timestamp, source_type, content, screenshot_path, monitor_info, tags, user_notes
             FROM records
             WHERE timestamp >= ?1 AND timestamp <= ?2
             ORDER BY timestamp ASC",
        )
        .map_err(|e| format!("Failed to prepare query: {}", e))?;

    let records = stmt
        .query_map(params![start_time, end_time], |row| {
            Ok(Record {
                id: row.get(0)?,
                timestamp: row.get(1)?,
                source_type: row.get(2)?,
                content: row.get(3)?,
                screenshot_path: row.get(4)?,
                monitor_info: row.get(5)?,
                tags: row.get(6)?,
                user_notes: row.get(7)?,
            })
        })
        .map_err(|e| format!("Failed to query records: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to collect records: {}", e))?;

    // Convert to timeline
    let events = records_to_events(records);
    let hour_groups = group_by_hour(events);
    let total_events: usize = hour_groups.iter().map(|g| g.count).sum();
    let active_hours = hour_groups.len();
    let work_time_estimate = calculate_work_time_estimate(&hour_groups);

    Ok(TimelineData {
        date: date.to_string(),
        hour_groups,
        total_events,
        active_hours,
        work_time_estimate,
    })
}

/// Get timeline data for today.
pub fn get_today_timeline_data() -> Result<TimelineData, String> {
    let today = Local::now().format("%Y-%m-%d").to_string();
    get_timeline_data_for_date(&today)
}

/// Get timeline data for a date range.
pub fn get_timeline_data_for_range(
    start_date: &str,
    end_date: &str,
) -> Result<Vec<TimelineData>, String> {
    let start = chrono::NaiveDate::parse_from_str(start_date, "%Y-%m-%d")
        .map_err(|e| format!("Invalid start date format: {}", e))?;
    let end = chrono::NaiveDate::parse_from_str(end_date, "%Y-%m-%d")
        .map_err(|e| format!("Invalid end date format: {}", e))?;

    if start > end {
        return Err("Start date must be before or equal to end date".to_string());
    }

    let mut result = Vec::new();
    let mut current = start;

    while current <= end {
        let date_str = current.format("%Y-%m-%d").to_string();
        match get_timeline_data_for_date(&date_str) {
            Ok(data) => result.push(data),
            Err(e) => tracing::warn!("Failed to get timeline for {}: {}", date_str, e),
        }
        current = current
            .checked_add_days(chrono::Days::new(1))
            .ok_or("Date overflow")?;
    }

    Ok(result)
}

// ============================================================================
// Tauri Commands
// ============================================================================

/// Tauri command to get timeline data for today.
#[command]
pub fn get_timeline_today() -> Result<TimelineData, String> {
    get_today_timeline_data()
}

/// Tauri command to get timeline data for a specific date.
#[command]
pub fn get_timeline_for_date(date: String) -> Result<TimelineData, String> {
    get_timeline_data_for_date(&date)
}

/// Tauri command to get timeline data for a date range.
#[command]
pub fn get_timeline_for_range(
    start_date: String,
    end_date: String,
) -> Result<Vec<TimelineData>, String> {
    get_timeline_data_for_range(&start_date, &end_date)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_preview_from_json_summary() {
        let content =
            r#"{"summary": "Working on code review for PR #123", "details": "more text"}"#;
        let preview = generate_preview(content, 30);
        // "Working on code review for PR " is exactly 30 chars
        assert_eq!(preview, "Working on code review for PR ...");
    }

    #[test]
    fn test_generate_preview_from_json_note() {
        let content = r#"{"note": "Meeting with team about project timeline", "time": "10:00"}"#;
        let preview = generate_preview(content, 35);
        // "Meeting with team about project tim" is exactly 35 chars
        assert_eq!(preview, "Meeting with team about project tim...");
    }

    #[test]
    fn test_generate_preview_truncates_long_text() {
        let content = "This is a very long text that should be truncated because it exceeds the maximum length";
        let preview = generate_preview(content, 20);
        assert_eq!(preview, "This is a very long ...");
    }

    #[test]
    fn test_generate_preview_short_text() {
        let content = "Short text";
        let preview = generate_preview(content, 80);
        assert_eq!(preview, "Short text");
    }

    #[test]
    fn test_calculate_work_time_estimate_empty() {
        let groups: Vec<TimelineHourGroup> = vec![];
        let estimate = calculate_work_time_estimate(&groups);
        assert_eq!(estimate, 0.0);
    }

    #[test]
    fn test_calculate_work_time_estimate_single_event() {
        let groups = vec![TimelineHourGroup {
            hour: 10,
            label: "10:00 - 11:00".to_string(),
            events: vec![],
            count: 1, // Less than threshold
        }];
        let estimate = calculate_work_time_estimate(&groups);
        assert_eq!(estimate, 0.0);
    }

    #[test]
    fn test_calculate_work_time_estimate_active_hours() {
        let groups = vec![
            TimelineHourGroup {
                hour: 9,
                label: "09:00 - 10:00".to_string(),
                events: vec![],
                count: 3, // Active
            },
            TimelineHourGroup {
                hour: 10,
                label: "10:00 - 11:00".to_string(),
                events: vec![],
                count: 2, // Active
            },
            TimelineHourGroup {
                hour: 14,
                label: "14:00 - 15:00".to_string(),
                events: vec![],
                count: 5, // Active
            },
        ];
        let estimate = calculate_work_time_estimate(&groups);
        // 3 active hours * 0.75 = 2.25
        assert!((estimate - 2.25).abs() < 0.01);
    }

    #[test]
    fn test_parse_timestamp_valid() {
        let timestamp = "2024-03-15T10:30:00Z";
        let result = parse_timestamp(timestamp);
        assert!(result.is_ok());
        let dt = result.unwrap();
        assert_eq!(dt.hour(), 10);
        assert_eq!(dt.minute(), 30);
    }

    #[test]
    fn test_parse_timestamp_invalid() {
        let timestamp = "invalid-timestamp";
        let result = parse_timestamp(timestamp);
        assert!(result.is_err());
    }
}
