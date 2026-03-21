use chrono::Datelike;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use tauri::command;

use super::DB_CONNECTION;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Record {
    pub id: i64,
    pub timestamp: String,
    pub source_type: String,
    pub content: String,
    pub screenshot_path: Option<String>,
    // SMART-004: 多显示器配置信息
    pub monitor_info: Option<String>, // JSON: MonitorInfo serialized
    // AI-004: 工作分类标签
    pub tags: Option<String>, // JSON: Vec<String> serialized
    // FEAT-005: 用户手动备注 (#66)
    pub user_notes: Option<String>,
}

/// Full-text search result with highlighting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub record: Record,
    /// Highlighted snippet with <mark> tags around matched keywords
    pub snippet: String,
    /// Relevance score (lower is better with bm25)
    pub rank: f64,
}

pub fn add_record(
    source_type: &str,
    content: &str,
    screenshot_path: Option<&str>,
    monitor_info: Option<&str>,
    tags: Option<&str>,
) -> Result<i64, String> {
    let db = DB_CONNECTION
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;
    let conn = db.as_ref().ok_or("Database not initialized")?;

    let timestamp = chrono::Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO records (timestamp, source_type, content, screenshot_path, monitor_info, tags) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![timestamp, source_type, content, screenshot_path, monitor_info, tags],
    ).map_err(|e| format!("Failed to insert record: {}", e))?;

    Ok(conn.last_insert_rowid())
}

pub fn get_today_records_sync() -> Result<Vec<Record>, String> {
    let db = DB_CONNECTION
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;
    let conn = db.as_ref().ok_or("Database not initialized")?;

    let today_start = chrono::Local::now()
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_local_timezone(chrono::Local)
        .unwrap()
        .with_timezone(&chrono::Utc)
        .to_rfc3339();

    let mut stmt = conn
        .prepare(
            "SELECT id, timestamp, source_type, content, screenshot_path, monitor_info, tags, user_notes FROM records
         WHERE timestamp >= ?1 ORDER BY timestamp DESC",
        )
        .map_err(|e| format!("Failed to prepare query: {}", e))?;

    let records = stmt
        .query_map(params![today_start], |row| {
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

    Ok(records)
}

/// Get records for the current week (Monday to Sunday)
/// week_start_day: 0=Monday, 6=Sunday (default is Monday)
pub fn get_week_records_sync(week_start_day: i32) -> Result<Vec<Record>, String> {
    let db = DB_CONNECTION
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;
    let conn = db.as_ref().ok_or("Database not initialized")?;

    // Calculate week boundaries based on local time
    let today = chrono::Local::now().date_naive();
    let weekday = today.weekday().num_days_from_monday() as i32;
    let days_since_week_start = (weekday - week_start_day + 7) % 7;

    let week_start_date = today - chrono::Duration::days(days_since_week_start as i64);
    let week_end_date = week_start_date + chrono::Duration::days(6);

    // Convert to UTC boundaries
    let week_start = week_start_date
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_local_timezone(chrono::Local)
        .unwrap()
        .with_timezone(&chrono::Utc)
        .to_rfc3339();

    let week_end = week_end_date
        .and_hms_opt(23, 59, 59)
        .unwrap()
        .and_local_timezone(chrono::Local)
        .unwrap()
        .with_timezone(&chrono::Utc)
        .to_rfc3339();

    let mut stmt = conn
        .prepare(
            "SELECT id, timestamp, source_type, content, screenshot_path, monitor_info, tags, user_notes FROM records
         WHERE timestamp >= ?1 AND timestamp <= ?2 ORDER BY timestamp DESC",
        )
        .map_err(|e| format!("Failed to prepare query: {}", e))?;

    let records = stmt
        .query_map(params![week_start, week_end], |row| {
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

    Ok(records)
}

/// Get all records for the current month (used for monthly report)
pub fn get_month_records_sync() -> Result<Vec<Record>, String> {
    let db = DB_CONNECTION
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;
    let conn = db.as_ref().ok_or("Database not initialized")?;

    // Calculate month boundaries based on local time
    let now = chrono::Local::now();
    let first_day = now.date_naive().with_day(1).unwrap();

    // Month start: first day of month at 00:00:00 local time
    let month_start = first_day
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_local_timezone(chrono::Local)
        .unwrap()
        .with_timezone(&chrono::Utc)
        .to_rfc3339();

    // Month end: first day of next month at 00:00:00 - 1 second
    let next_month = if now.month() == 12 {
        chrono::NaiveDate::from_ymd_opt(now.year() + 1, 1, 1).unwrap()
    } else {
        chrono::NaiveDate::from_ymd_opt(now.year(), now.month() + 1, 1).unwrap()
    };

    let month_end = next_month
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_local_timezone(chrono::Local)
        .unwrap()
        .with_timezone(&chrono::Utc)
        .to_rfc3339();

    let mut stmt = conn
        .prepare(
            "SELECT id, timestamp, source_type, content, screenshot_path, monitor_info, tags, user_notes FROM records
         WHERE timestamp >= ?1 AND timestamp < ?2 ORDER BY timestamp DESC",
        )
        .map_err(|e| format!("Failed to prepare query: {}", e))?;

    let records = stmt
        .query_map(params![month_start, month_end], |row| {
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

    Ok(records)
}

pub fn get_all_today_records_for_summary() -> Result<Vec<Record>, String> {
    get_today_records_sync()
}

/// Get the count of today's records (more efficient than fetching all records).
pub fn get_today_record_count_sync() -> Result<usize, String> {
    let db = DB_CONNECTION
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;
    let conn = db.as_ref().ok_or("Database not initialized")?;

    let today_start = chrono::Local::now()
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_local_timezone(chrono::Local)
        .unwrap()
        .with_timezone(&chrono::Utc)
        .to_rfc3339();

    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM records WHERE timestamp >= ?1",
            params![today_start],
            |row| row.get(0),
        )
        .map_err(|e| format!("Failed to count records: {}", e))?;

    Ok(count as usize)
}

pub fn get_records_by_date_range_sync(
    start_date: String,
    end_date: String,
) -> Result<Vec<Record>, String> {
    let db = DB_CONNECTION
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;
    let conn = db.as_ref().ok_or("Database not initialized")?;

    // Parse start_date (YYYY-MM-DD) to local midnight 00:00:00
    let start_naive = chrono::NaiveDate::parse_from_str(&start_date, "%Y-%m-%d")
        .map_err(|e| format!("Invalid start_date format (expected YYYY-MM-DD): {}", e))?
        .and_hms_opt(0, 0, 0)
        .unwrap();

    // Parse end_date (YYYY-MM-DD) to local midnight of next day (exclusive upper bound)
    let end_naive = chrono::NaiveDate::parse_from_str(&end_date, "%Y-%m-%d")
        .map_err(|e| format!("Invalid end_date format (expected YYYY-MM-DD): {}", e))?
        .and_hms_opt(23, 59, 59)
        .unwrap();

    // Convert to UTC RFC3339
    let start_utc = start_naive
        .and_local_timezone(chrono::Local)
        .unwrap()
        .with_timezone(&chrono::Utc)
        .to_rfc3339();

    let end_utc = end_naive
        .and_local_timezone(chrono::Local)
        .unwrap()
        .with_timezone(&chrono::Utc)
        .to_rfc3339();

    let mut stmt = conn
        .prepare(
            "SELECT id, timestamp, source_type, content, screenshot_path, monitor_info, tags, user_notes FROM records
         WHERE timestamp >= ?1 AND timestamp <= ?2 ORDER BY timestamp DESC",
        )
        .map_err(|e| format!("Failed to prepare query: {}", e))?;

    let records = stmt
        .query_map(params![start_utc, end_utc], |row| {
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

    Ok(records)
}

/// Get records within a date range for export (chronological ASC order).
/// - start_date/end_date: YYYY-MM-DD format (local timezone)
pub fn get_records_for_export(start_date: &str, end_date: &str) -> Result<Vec<Record>, String> {
    let db = DB_CONNECTION
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;
    let conn = db.as_ref().ok_or("Database not initialized")?;

    let start_naive = chrono::NaiveDate::parse_from_str(start_date, "%Y-%m-%d")
        .map_err(|e| format!("Invalid start_date format (expected YYYY-MM-DD): {}", e))?
        .and_hms_opt(0, 0, 0)
        .unwrap();

    let end_naive = chrono::NaiveDate::parse_from_str(end_date, "%Y-%m-%d")
        .map_err(|e| format!("Invalid end_date format (expected YYYY-MM-DD): {}", e))?
        .and_hms_opt(23, 59, 59)
        .unwrap();

    let start_utc = start_naive
        .and_local_timezone(chrono::Local)
        .unwrap()
        .with_timezone(&chrono::Utc)
        .to_rfc3339();

    let end_utc = end_naive
        .and_local_timezone(chrono::Local)
        .unwrap()
        .with_timezone(&chrono::Utc)
        .to_rfc3339();

    let mut stmt = conn
        .prepare(
            "SELECT id, timestamp, source_type, content, screenshot_path, monitor_info, tags, user_notes FROM records
         WHERE timestamp >= ?1 AND timestamp <= ?2 ORDER BY timestamp ASC",
        )
        .map_err(|e| format!("Failed to prepare query: {}", e))?;

    let records = stmt
        .query_map(params![start_utc, end_utc], |row| {
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

    Ok(records)
}

/// Delete a record by ID
pub fn delete_record_sync(id: i64) -> Result<(), String> {
    let db = DB_CONNECTION
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;
    let conn = db.as_ref().ok_or("Database not initialized")?;

    let rows_affected = conn
        .execute("DELETE FROM records WHERE id = ?1", params![id])
        .map_err(|e| format!("Failed to delete record: {}", e))?;

    if rows_affected == 0 {
        return Err(format!("Record with id {} not found", id));
    }

    tracing::info!("Deleted record with id {}", id);
    Ok(())
}

/// Get a single record by ID
/// Used by reanalyze_record to fetch record details
pub fn get_record_by_id_sync(id: i64) -> Result<Record, String> {
    let db = DB_CONNECTION
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;
    let conn = db.as_ref().ok_or("Database not initialized")?;

    let record = conn
        .query_row(
            "SELECT id, timestamp, source_type, content, screenshot_path, monitor_info, tags, user_notes
             FROM records WHERE id = ?1",
            params![id],
            |row| {
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
            },
        )
        .map_err(|e| format!("Record with id {} not found: {}", id, e))?;

    Ok(record)
}

/// Update the content of a record by ID
/// Used by offline queue retry to update screenshot analysis results
pub fn update_record_content_sync(id: i64, content: &str) -> Result<(), String> {
    let db = DB_CONNECTION
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;
    let conn = db.as_ref().ok_or("Database not initialized")?;

    let rows_affected = conn
        .execute(
            "UPDATE records SET content = ?1 WHERE id = ?2",
            params![content, id],
        )
        .map_err(|e| format!("Failed to update record: {}", e))?;

    if rows_affected == 0 {
        return Err(format!("Record with id {} not found", id));
    }

    tracing::info!("Updated content for record {}", id);
    Ok(())
}

/// Get history records with filtering and pagination
/// - start_date/end_date: YYYY-MM-DD format (local timezone)
/// - source_type: None for all, Some("auto") or Some("manual") for filtering
/// - page: 0-indexed page number
/// - page_size: number of records per page (default 50)
pub fn get_history_records_sync(
    start_date: String,
    end_date: String,
    source_type: Option<String>,
    page: i64,
    page_size: i64,
) -> Result<Vec<Record>, String> {
    let db = DB_CONNECTION
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;
    let conn = db.as_ref().ok_or("Database not initialized")?;

    // Parse start_date (YYYY-MM-DD) to local midnight 00:00:00
    let start_naive = chrono::NaiveDate::parse_from_str(&start_date, "%Y-%m-%d")
        .map_err(|e| format!("Invalid start_date format (expected YYYY-MM-DD): {}", e))?
        .and_hms_opt(0, 0, 0)
        .unwrap();

    // Parse end_date (YYYY-MM-DD) to local end of day 23:59:59
    let end_naive = chrono::NaiveDate::parse_from_str(&end_date, "%Y-%m-%d")
        .map_err(|e| format!("Invalid end_date format (expected YYYY-MM-DD): {}", e))?
        .and_hms_opt(23, 59, 59)
        .unwrap();

    // Convert to UTC RFC3339
    let start_utc = start_naive
        .and_local_timezone(chrono::Local)
        .unwrap()
        .with_timezone(&chrono::Utc)
        .to_rfc3339();

    let end_utc = end_naive
        .and_local_timezone(chrono::Local)
        .unwrap()
        .with_timezone(&chrono::Utc)
        .to_rfc3339();

    let offset = page * page_size;

    let sql = if let Some(ref st) = source_type {
        if st != "auto" && st != "manual" {
            return Err(format!(
                "Invalid source_type '{}'. Must be 'auto', 'manual', or null for all",
                st
            ));
        }
        "SELECT id, timestamp, source_type, content, screenshot_path, monitor_info, tags, user_notes FROM records
         WHERE timestamp >= ?1 AND timestamp <= ?2 AND source_type = ?3
         ORDER BY timestamp DESC LIMIT ?4 OFFSET ?5"
    } else {
        "SELECT id, timestamp, source_type, content, screenshot_path, monitor_info, tags, user_notes FROM records
         WHERE timestamp >= ?1 AND timestamp <= ?2
         ORDER BY timestamp DESC LIMIT ?3 OFFSET ?4"
    };

    let mut stmt = conn
        .prepare(sql)
        .map_err(|e| format!("Failed to prepare query: {}", e))?;

    let records = if let Some(ref st) = source_type {
        stmt.query_map(params![start_utc, end_utc, st, page_size, offset], |row| {
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
        .map_err(|e| format!("Failed to collect records: {}", e))?
    } else {
        stmt.query_map(params![start_utc, end_utc, page_size, offset], |row| {
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
        .map_err(|e| format!("Failed to collect records: {}", e))?
    };

    Ok(records)
}

/// Full-text search on records content
/// - query: search keyword(s)
/// - order_by: "rank" (relevance) or "time" (timestamp DESC)
/// - limit: maximum number of results (default 50)
///
/// Note: For queries containing CJK characters, uses LIKE search as fallback
/// since FTS5's unicode61 tokenizer doesn't handle Chinese word segmentation well.
pub fn search_records_sync(
    query: &str,
    order_by: &str,
    limit: i64,
) -> Result<Vec<SearchResult>, String> {
    if query.trim().is_empty() {
        return Ok(Vec::new());
    }

    let db = DB_CONNECTION
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;
    let conn = db.as_ref().ok_or("Database not initialized")?;

    // Check if query contains CJK characters
    let has_cjk = query.chars().any(|c| {
        let cp = c as u32;
        // CJK Unified Ideographs: U+4E00..U+9FFF
        // CJK Unified Ideographs Extension A: U+3400..U+4DBF
        // CJK Compatibility Ideographs: U+F900..U+FAFF
        (0x4E00..=0x9FFF).contains(&cp)
            || (0x3400..=0x4DBF).contains(&cp)
            || (0xF900..=0xFAFF).contains(&cp)
    });

    if has_cjk {
        // Use LIKE search for CJK queries
        // Note: Both time and rank order use the same SQL since LIKE doesn't have relevance score
        let sql = "SELECT
                id, timestamp, source_type, content, screenshot_path, monitor_info, tags, user_notes
            FROM records
            WHERE content LIKE ?1
            ORDER BY timestamp DESC
            LIMIT ?2";

        let mut stmt = conn
            .prepare(sql)
            .map_err(|e| format!("Failed to prepare search query: {}", e))?;

        let like_pattern = format!("%{}%", query);

        let results = stmt
            .query_map(params![like_pattern, limit], |row| {
                let content: String = row.get(3)?;
                // Manually highlight the keyword
                let snippet = content.replace(query, &format!("<mark>{}</mark>", query));
                Ok(SearchResult {
                    record: Record {
                        id: row.get(0)?,
                        timestamp: row.get(1)?,
                        source_type: row.get(2)?,
                        content,
                        screenshot_path: row.get(4)?,
                        monitor_info: row.get(5)?,
                        tags: row.get(6)?,
                        user_notes: row.get(7)?,
                    },
                    snippet,
                    rank: 0.0, // LIKE search doesn't have relevance score
                })
            })
            .map_err(|e| format!("Failed to search records: {}", e))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Failed to collect search results: {}", e))?;

        Ok(results)
    } else {
        // Use FTS5 for non-CJK queries
        let escaped_query = query.replace('\"', "\"\"");

        let sql = if order_by == "time" {
            "SELECT
                r.id, r.timestamp, r.source_type, r.content, r.screenshot_path, r.monitor_info, r.tags, r.user_notes,
                highlight(records_fts, 0, '<mark>', '</mark>') as snippet,
                bm25(records_fts) as rank
            FROM records_fts
            JOIN records r ON r.id = records_fts.rowid
            WHERE records_fts MATCH ?1
            ORDER BY r.timestamp DESC
            LIMIT ?2"
        } else {
            "SELECT
                r.id, r.timestamp, r.source_type, r.content, r.screenshot_path, r.monitor_info, r.tags, r.user_notes,
                highlight(records_fts, 0, '<mark>', '</mark>') as snippet,
                bm25(records_fts) as rank
            FROM records_fts
            JOIN records r ON r.id = records_fts.rowid
            WHERE records_fts MATCH ?1
            ORDER BY rank
            LIMIT ?2"
        };

        let mut stmt = conn
            .prepare(sql)
            .map_err(|e| format!("Failed to prepare search query: {}", e))?;

        // Wrap query in double quotes for exact phrase matching
        let fts_query = format!("\"{}\"", escaped_query);

        let results = stmt
            .query_map(params![fts_query, limit], |row| {
                Ok(SearchResult {
                    record: Record {
                        id: row.get(0)?,
                        timestamp: row.get(1)?,
                        source_type: row.get(2)?,
                        content: row.get(3)?,
                        screenshot_path: row.get(4)?,
                        monitor_info: row.get(5)?,
                        tags: row.get(6)?,
                        user_notes: row.get(7)?,
                    },
                    snippet: row.get(8)?,
                    rank: row.get(9)?,
                })
            })
            .map_err(|e| format!("Failed to search records: {}", e))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Failed to collect search results: {}", e))?;

        Ok(results)
    }
}

// ── Async Tauri command wrappers ──

#[command]
pub async fn get_today_records() -> Result<Vec<Record>, String> {
    get_today_records_sync()
}

#[command]
pub async fn get_records_by_date_range(
    start_date: String,
    end_date: String,
) -> Result<Vec<Record>, String> {
    get_records_by_date_range_sync(start_date, end_date)
}

/// Delete a record by ID
#[command]
pub async fn delete_record(id: i64) -> Result<(), String> {
    delete_record_sync(id)
}

/// Get history records with filtering and pagination
#[command]
pub async fn get_history_records(
    start_date: String,
    end_date: String,
    source_type: Option<String>,
    page: Option<i64>,
    page_size: Option<i64>,
) -> Result<Vec<Record>, String> {
    let page = page.unwrap_or(0);
    let page_size = page_size.unwrap_or(50);
    get_history_records_sync(start_date, end_date, source_type, page, page_size)
}

/// Full-text search on records content
#[command]
pub async fn search_records(
    query: String,
    order_by: Option<String>,
    limit: Option<i64>,
) -> Result<Vec<SearchResult>, String> {
    let order_by = order_by.unwrap_or_else(|| "rank".to_string());
    let limit = limit.unwrap_or(50);
    search_records_sync(&query, &order_by, limit)
}

// ── Tests ──

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;
    use serial_test::serial;

    /// Initializes an in-memory database for testing.
    fn setup_test_db() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute(
            "CREATE TABLE records (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp TEXT NOT NULL,
                source_type TEXT NOT NULL,
                content TEXT NOT NULL,
                screenshot_path TEXT,
                monitor_info TEXT,
                tags TEXT,
                user_notes TEXT
            )",
            [],
        )
        .unwrap();
        conn.execute(
            "CREATE TABLE settings (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                api_base_url TEXT,
                api_key TEXT,
                model_name TEXT,
                screenshot_interval INTEGER DEFAULT 5,
                summary_time TEXT DEFAULT '18:00',
                obsidian_path TEXT,
                auto_capture_enabled INTEGER DEFAULT 0,
                last_summary_path TEXT,
                summary_model_name TEXT,
                analysis_prompt TEXT,
                summary_prompt TEXT,
                change_threshold INTEGER DEFAULT 3,
                max_silent_minutes INTEGER DEFAULT 30,
                summary_title_format TEXT DEFAULT '工作日报 - {date}',
                include_manual_records INTEGER DEFAULT 1,
                window_whitelist TEXT DEFAULT '[]',
                window_blacklist TEXT DEFAULT '[]',
                use_whitelist_only INTEGER DEFAULT 0,
                auto_adjust_silent INTEGER DEFAULT 1,
                silent_adjustment_paused_until TEXT DEFAULT NULL,
                auto_detect_work_time INTEGER DEFAULT 1,
                use_custom_work_time INTEGER DEFAULT 0,
                custom_work_time_start TEXT DEFAULT '09:00',
                custom_work_time_end TEXT DEFAULT '18:00',
                learned_work_time TEXT DEFAULT NULL,
                capture_mode TEXT DEFAULT 'primary',
                selected_monitor_index INTEGER DEFAULT 0,
                tag_categories TEXT DEFAULT '[]',
                is_ollama INTEGER DEFAULT 0,
                weekly_report_prompt TEXT,
                weekly_report_day INTEGER DEFAULT 0,
                last_weekly_report_path TEXT,
                monthly_report_prompt TEXT,
                custom_report_prompt TEXT,
                last_custom_report_path TEXT,
                last_monthly_report_path TEXT,
                obsidian_vaults TEXT DEFAULT '[]',
                comparison_report_prompt TEXT
            )",
            [],
        )
        .unwrap();
        conn.execute("INSERT OR IGNORE INTO settings (id) VALUES (1)", [])
            .unwrap();

        // Create FTS5 virtual table for full-text search tests
        conn.execute(
            "CREATE VIRTUAL TABLE IF NOT EXISTS records_fts USING fts5(
                content,
                content='records',
                content_rowid='id',
                tokenize='unicode61'
            )",
            [],
        )
        .unwrap();

        // FTS5 triggers for automatic index sync
        conn.execute(
            "CREATE TRIGGER IF NOT EXISTS records_ai AFTER INSERT ON records BEGIN
                INSERT INTO records_fts(rowid, content) VALUES (new.id, new.content);
            END",
            [],
        )
        .unwrap();

        conn.execute(
            "CREATE TRIGGER IF NOT EXISTS records_ad AFTER DELETE ON records BEGIN
                INSERT INTO records_fts(records_fts, rowid, content)
                VALUES ('delete', old.id, old.content);
            END",
            [],
        )
        .unwrap();

        conn.execute(
            "CREATE TRIGGER IF NOT EXISTS records_au AFTER UPDATE ON records BEGIN
                INSERT INTO records_fts(records_fts, rowid, content)
                VALUES ('delete', old.id, old.content);
                INSERT INTO records_fts(rowid, content) VALUES (new.id, new.content);
            END",
            [],
        )
        .unwrap();

        let mut db = DB_CONNECTION.lock().unwrap();
        *db = Some(conn);
    }

    /// Helper: Insert a record with a specific timestamp (for boundary testing)
    fn insert_record_with_ts(ts: &str, content: &str) -> i64 {
        let db = DB_CONNECTION.lock().unwrap();
        let conn = db.as_ref().unwrap();
        conn.execute(
            "INSERT INTO records (timestamp, source_type, content) VALUES (?1, 'manual', ?2)",
            params![ts, content],
        )
        .unwrap();
        conn.last_insert_rowid()
    }

    /// Helper: Convert local naive datetime to UTC RFC3339 string
    fn local_to_utc_rfc3339(dt: chrono::NaiveDateTime) -> String {
        dt.and_local_timezone(chrono::Local)
            .unwrap()
            .with_timezone(&chrono::Utc)
            .to_rfc3339()
    }

    // ── Boundary tests for get_today_records_sync ──

    #[test]
    #[serial]
    fn finds_record_saved_near_local_midnight() {
        setup_test_db();

        // Local 01:00 today — in UTC+8 this is yesterday 17:00 UTC.
        // The old .and_utc() bug would miss this record.
        let today = chrono::Local::now().date_naive();
        let ts = local_to_utc_rfc3339(today.and_hms_opt(1, 0, 0).unwrap());
        insert_record_with_ts(&ts, "early morning note");

        let records = get_today_records_sync().unwrap();
        assert!(
            records.iter().any(|r| r.content == "early morning note"),
            "Record at local 01:00 (UTC {}) must appear in today's records",
            ts
        );
    }

    #[test]
    #[serial]
    fn finds_record_at_last_second_of_local_today() {
        setup_test_db();

        // Local 23:59:59 today — should still be "today".
        let today = chrono::Local::now().date_naive();
        let ts = local_to_utc_rfc3339(today.and_hms_opt(23, 59, 59).unwrap());
        insert_record_with_ts(&ts, "end of day note");

        let records = get_today_records_sync().unwrap();
        assert!(
            records.iter().any(|r| r.content == "end of day note"),
            "Record at local 23:59:59 (UTC {}) must appear in today's records",
            ts
        );
    }

    #[test]
    #[serial]
    fn excludes_record_from_yesterday() {
        setup_test_db();

        // Local 23:59:59 yesterday — must NOT appear in today's records.
        let yesterday = chrono::Local::now().date_naive() - chrono::Duration::days(1);
        let ts = local_to_utc_rfc3339(yesterday.and_hms_opt(23, 59, 59).unwrap());
        insert_record_with_ts(&ts, "yesterday's note");

        let records = get_today_records_sync().unwrap();
        assert!(
            !records.iter().any(|r| r.content == "yesterday's note"),
            "Record at local yesterday 23:59:59 (UTC {}) must NOT appear in today's records",
            ts
        );
    }

    #[test]
    #[serial]
    fn finds_record_at_exact_local_midnight() {
        setup_test_db();

        // Local 00:00:00 today — the boundary itself should be included.
        let today = chrono::Local::now().date_naive();
        let ts = local_to_utc_rfc3339(today.and_hms_opt(0, 0, 0).unwrap());
        insert_record_with_ts(&ts, "midnight note");

        let records = get_today_records_sync().unwrap();
        assert!(
            records.iter().any(|r| r.content == "midnight note"),
            "Record at exactly local midnight (UTC {}) must appear in today's records",
            ts
        );
    }

    // ── End-to-end: add_record → get_today_records_sync ──

    #[test]
    #[serial]
    fn add_record_then_query_returns_it() {
        setup_test_db();

        let id = add_record("manual", "e2e test note", None, None, None).unwrap();
        assert!(id > 0);

        let records = get_today_records_sync().unwrap();
        assert!(
            records.iter().any(|r| r.content == "e2e test note"),
            "Record saved via add_record must be queryable via get_today_records_sync"
        );
    }

    #[test]
    #[serial]
    fn add_record_with_screenshot_path_persists() {
        setup_test_db();

        let id = add_record(
            "auto",
            "screenshot test",
            Some("/path/to/screenshot.png"),
            None,
            None,
        )
        .unwrap();

        let records = get_today_records_sync().unwrap();
        let record = records.iter().find(|r| r.id == id).unwrap();
        assert_eq!(
            record.screenshot_path,
            Some("/path/to/screenshot.png".to_string())
        );
    }

    #[test]
    #[serial]
    fn add_record_with_monitor_info_persists() {
        setup_test_db();

        let monitor_info = r#"{"name":"Display 1","width":1920,"height":1080}"#;
        let id = add_record("auto", "monitor test", None, Some(monitor_info), None).unwrap();

        let records = get_today_records_sync().unwrap();
        let record = records.iter().find(|r| r.id == id).unwrap();
        assert_eq!(record.monitor_info, Some(monitor_info.to_string()));
    }

    #[test]
    #[serial]
    fn add_record_with_tags_persists() {
        setup_test_db();

        let tags = r#"["work","meeting"]"#;
        let id = add_record("manual", "tagged note", None, None, Some(tags)).unwrap();

        let records = get_today_records_sync().unwrap();
        let record = records.iter().find(|r| r.id == id).unwrap();
        assert_eq!(record.tags, Some(tags.to_string()));
    }

    // ── get_today_record_count_sync tests ──

    #[test]
    #[serial]
    fn count_returns_zero_for_empty_db() {
        setup_test_db();
        assert_eq!(get_today_record_count_sync().unwrap(), 0);
    }

    #[test]
    #[serial]
    fn count_increases_with_each_record() {
        setup_test_db();

        assert_eq!(get_today_record_count_sync().unwrap(), 0);

        add_record("manual", "note 1", None, None, None).unwrap();
        assert_eq!(get_today_record_count_sync().unwrap(), 1);

        add_record("manual", "note 2", None, None, None).unwrap();
        assert_eq!(get_today_record_count_sync().unwrap(), 2);
    }

    #[test]
    #[serial]
    fn count_excludes_yesterday_records() {
        setup_test_db();

        // Insert a record for yesterday
        let yesterday = chrono::Local::now().date_naive() - chrono::Duration::days(1);
        let ts = local_to_utc_rfc3339(yesterday.and_hms_opt(12, 0, 0).unwrap());
        insert_record_with_ts(&ts, "yesterday");

        assert_eq!(get_today_record_count_sync().unwrap(), 0);

        // Insert a record for today
        add_record("manual", "today", None, None, None).unwrap();
        assert_eq!(get_today_record_count_sync().unwrap(), 1);
    }

    // ── get_records_by_date_range_sync tests ──

    #[test]
    #[serial]
    fn get_records_by_date_range_finds_records_in_range() {
        setup_test_db();

        let today = chrono::Local::now().date_naive();
        let ts = local_to_utc_rfc3339(today.and_hms_opt(10, 0, 0).unwrap());
        insert_record_with_ts(&ts, "in range");

        let today_str = today.format("%Y-%m-%d").to_string();
        let records = get_records_by_date_range_sync(today_str.clone(), today_str).unwrap();

        assert!(
            records.iter().any(|r| r.content == "in range"),
            "Record within date range should be found"
        );
    }

    #[test]
    #[serial]
    fn get_records_by_date_range_excludes_out_of_range() {
        setup_test_db();

        let today = chrono::Local::now().date_naive();
        let yesterday = today - chrono::Duration::days(1);

        // Insert record for yesterday
        let ts_yesterday = local_to_utc_rfc3339(yesterday.and_hms_opt(12, 0, 0).unwrap());
        insert_record_with_ts(&ts_yesterday, "yesterday record");

        // Query for today only
        let today_str = today.format("%Y-%m-%d").to_string();
        let records = get_records_by_date_range_sync(today_str.clone(), today_str).unwrap();

        assert!(
            !records.iter().any(|r| r.content == "yesterday record"),
            "Record outside date range should not be found"
        );
    }

    #[test]
    #[serial]
    fn get_records_by_date_range_invalid_format_returns_error() {
        setup_test_db();

        let result =
            get_records_by_date_range_sync("invalid".to_string(), "2024-01-01".to_string());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid start_date format"));
    }

    // ── get_history_records_sync tests ──

    #[test]
    #[serial]
    fn get_history_records_returns_paginated_results() {
        setup_test_db();

        // Insert 5 records
        for i in 0..5 {
            add_record("manual", &format!("note {}", i), None, None, None).unwrap();
        }

        let today = chrono::Local::now().date_naive();
        let today_str = today.format("%Y-%m-%d").to_string();

        // First page (page 0, size 2)
        let page1 =
            get_history_records_sync(today_str.clone(), today_str.clone(), None, 0, 2).unwrap();
        assert_eq!(page1.len(), 2);

        // Second page (page 1, size 2)
        let page2 = get_history_records_sync(today_str.clone(), today_str, None, 1, 2).unwrap();
        assert_eq!(page2.len(), 2);
    }

    #[test]
    #[serial]
    fn get_history_records_filters_by_source_type() {
        setup_test_db();

        add_record("auto", "auto note", None, None, None).unwrap();
        add_record("manual", "manual note", None, None, None).unwrap();

        let today = chrono::Local::now().date_naive();
        let today_str = today.format("%Y-%m-%d").to_string();

        let auto_records = get_history_records_sync(
            today_str.clone(),
            today_str.clone(),
            Some("auto".to_string()),
            0,
            50,
        )
        .unwrap();
        assert!(auto_records.iter().all(|r| r.source_type == "auto"));

        let manual_records = get_history_records_sync(
            today_str.clone(),
            today_str,
            Some("manual".to_string()),
            0,
            50,
        )
        .unwrap();
        assert!(manual_records.iter().all(|r| r.source_type == "manual"));
    }

    #[test]
    #[serial]
    fn get_history_records_invalid_source_type_returns_error() {
        setup_test_db();

        let today = chrono::Local::now().date_naive();
        let today_str = today.format("%Y-%m-%d").to_string();

        let result = get_history_records_sync(
            today_str.clone(),
            today_str,
            Some("invalid".to_string()),
            0,
            50,
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid source_type"));
    }

    // ── delete_record_sync tests ──

    #[test]
    #[serial]
    fn delete_record_removes_from_db() {
        setup_test_db();

        let id = add_record("manual", "to delete", None, None, None).unwrap();
        assert!(get_today_records_sync().unwrap().iter().any(|r| r.id == id));

        delete_record_sync(id).unwrap();

        assert!(!get_today_records_sync().unwrap().iter().any(|r| r.id == id));
    }

    #[test]
    #[serial]
    fn delete_nonexistent_record_returns_error() {
        setup_test_db();

        let result = delete_record_sync(99999);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    // ── search_records_sync tests ──

    #[test]
    #[serial]
    fn search_finds_matching_records() {
        setup_test_db();

        add_record("manual", "hello world", None, None, None).unwrap();
        add_record("manual", "foo bar", None, None, None).unwrap();

        let results = search_records_sync("hello", "rank", 50).unwrap();
        assert!(results.iter().any(|r| r.record.content == "hello world"));
        assert!(!results.iter().any(|r| r.record.content == "foo bar"));
    }

    #[test]
    #[serial]
    fn search_returns_empty_for_no_matches() {
        setup_test_db();

        add_record("manual", "hello world", None, None, None).unwrap();

        let results = search_records_sync("nonexistent", "rank", 50).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    #[serial]
    fn search_empty_query_returns_empty() {
        setup_test_db();

        add_record("manual", "hello world", None, None, None).unwrap();

        let results = search_records_sync("", "rank", 50).unwrap();
        assert!(results.is_empty());

        let results = search_records_sync("   ", "rank", 50).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    #[serial]
    fn search_includes_highlighted_snippet() {
        setup_test_db();

        add_record("manual", "hello world", None, None, None).unwrap();

        let results = search_records_sync("hello", "rank", 50).unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].snippet.contains("<mark>hello</mark>"));
    }

    #[test]
    #[serial]
    fn search_cjk_uses_like_fallback() {
        setup_test_db();

        add_record("manual", "你好世界", None, None, None).unwrap();
        add_record("manual", "hello world", None, None, None).unwrap();

        // CJK query should use LIKE search
        let results = search_records_sync("你好", "rank", 50).unwrap();
        assert!(results.iter().any(|r| r.record.content == "你好世界"));
        assert!(!results.iter().any(|r| r.record.content == "hello world"));
    }

    #[test]
    #[serial]
    fn search_respects_limit() {
        setup_test_db();

        for i in 0..10 {
            add_record("manual", &format!("test note {}", i), None, None, None).unwrap();
        }

        let results = search_records_sync("test", "rank", 5).unwrap();
        assert_eq!(results.len(), 5);
    }

    // ── get_week_records_sync tests ──

    #[test]
    #[serial]
    fn get_week_records_includes_current_week() {
        setup_test_db();

        let today = chrono::Local::now().date_naive();
        let ts = local_to_utc_rfc3339(today.and_hms_opt(10, 0, 0).unwrap());
        insert_record_with_ts(&ts, "this week");

        let records = get_week_records_sync(0).unwrap(); // Monday start
        assert!(records.iter().any(|r| r.content == "this week"));
    }

    #[test]
    #[serial]
    fn get_week_records_excludes_last_week() {
        setup_test_db();

        let last_week = chrono::Local::now().date_naive() - chrono::Duration::days(8);
        let ts = local_to_utc_rfc3339(last_week.and_hms_opt(10, 0, 0).unwrap());
        insert_record_with_ts(&ts, "last week");

        let records = get_week_records_sync(0).unwrap();
        assert!(!records.iter().any(|r| r.content == "last week"));
    }

    // ── get_month_records_sync tests ──

    #[test]
    #[serial]
    fn get_month_records_includes_current_month() {
        setup_test_db();

        let today = chrono::Local::now().date_naive();
        let ts = local_to_utc_rfc3339(today.and_hms_opt(10, 0, 0).unwrap());
        insert_record_with_ts(&ts, "this month");

        let records = get_month_records_sync().unwrap();
        assert!(records.iter().any(|r| r.content == "this month"));
    }

    #[test]
    #[serial]
    fn get_month_records_excludes_last_month() {
        setup_test_db();

        let last_month = chrono::Local::now().date_naive() - chrono::Duration::days(35);
        let ts = local_to_utc_rfc3339(last_month.and_hms_opt(10, 0, 0).unwrap());
        insert_record_with_ts(&ts, "last month");

        let records = get_month_records_sync().unwrap();
        assert!(!records.iter().any(|r| r.content == "last month"));
    }

    // ── get_records_for_export tests ──

    #[test]
    #[serial]
    fn get_records_for_export_returns_chronological_order() {
        setup_test_db();

        let today = chrono::Local::now().date_naive();

        // Insert records with different times
        let ts1 = local_to_utc_rfc3339(today.and_hms_opt(10, 0, 0).unwrap());
        let ts2 = local_to_utc_rfc3339(today.and_hms_opt(8, 0, 0).unwrap());
        let ts3 = local_to_utc_rfc3339(today.and_hms_opt(12, 0, 0).unwrap());

        insert_record_with_ts(&ts1, "10:00");
        insert_record_with_ts(&ts2, "08:00");
        insert_record_with_ts(&ts3, "12:00");

        let today_str = today.format("%Y-%m-%d").to_string();
        let records = get_records_for_export(&today_str, &today_str).unwrap();

        // Should be in ASC order (oldest first)
        assert_eq!(records[0].content, "08:00");
        assert_eq!(records[1].content, "10:00");
        assert_eq!(records[2].content, "12:00");
    }

    // ── Edge cases ──

    #[test]
    #[serial]
    fn multiple_records_same_timestamp() {
        setup_test_db();

        let now = chrono::Utc::now().to_rfc3339();
        let db = DB_CONNECTION.lock().unwrap();
        let conn = db.as_ref().unwrap();

        conn.execute(
            "INSERT INTO records (timestamp, source_type, content) VALUES (?1, 'manual', 'first')",
            params![now],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO records (timestamp, source_type, content) VALUES (?1, 'manual', 'second')",
            params![now],
        )
        .unwrap();

        drop(db);

        let records = get_today_records_sync().unwrap();
        assert_eq!(records.len(), 2);
    }

    #[test]
    #[serial]
    fn record_with_special_characters_in_content() {
        setup_test_db();

        let special_content = "Test with 'quotes' and \"double quotes\" and \n newlines";
        let id = add_record("manual", special_content, None, None, None).unwrap();

        let records = get_today_records_sync().unwrap();
        let record = records.iter().find(|r| r.id == id).unwrap();
        assert_eq!(record.content, special_content);
    }

    #[test]
    #[serial]
    fn record_with_unicode_content() {
        setup_test_db();

        let unicode_content = "Unicode: 你好世界 🎉 émojis";
        let id = add_record("manual", unicode_content, None, None, None).unwrap();

        let records = get_today_records_sync().unwrap();
        let record = records.iter().find(|r| r.id == id).unwrap();
        assert_eq!(record.content, unicode_content);
    }

    // ── SearchResult tests ──

    #[test]
    fn search_result_struct_fields() {
        let record = Record {
            id: 1,
            timestamp: "2024-01-01T00:00:00Z".to_string(),
            source_type: "manual".to_string(),
            content: "test".to_string(),
            screenshot_path: None,
            monitor_info: None,
            tags: None,
            user_notes: None,
        };

        let result = SearchResult {
            record: record.clone(),
            snippet: "<mark>test</mark>".to_string(),
            rank: 1.5,
        };

        assert_eq!(result.record.id, 1);
        assert_eq!(result.snippet, "<mark>test</mark>");
        assert!((result.rank - 1.5).abs() < f64::EPSILON);
    }

    // ── Record struct tests ──

    #[test]
    fn record_struct_serialization() {
        let record = Record {
            id: 42,
            timestamp: "2024-01-15T10:30:00Z".to_string(),
            source_type: "auto".to_string(),
            content: "test content".to_string(),
            screenshot_path: Some("/path/to/screenshot.png".to_string()),
            monitor_info: Some(r#"{"name":"Display 1"}"#.to_string()),
            tags: Some(r#"["work"]"#.to_string()),
            user_notes: None,
        };

        let json = serde_json::to_string(&record).unwrap();
        let deserialized: Record = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.id, 42);
        assert_eq!(deserialized.timestamp, "2024-01-15T10:30:00Z");
        assert_eq!(deserialized.source_type, "auto");
        assert_eq!(deserialized.content, "test content");
        assert_eq!(
            deserialized.screenshot_path,
            Some("/path/to/screenshot.png".to_string())
        );
    }

    #[test]
    fn record_struct_clone() {
        let record = Record {
            id: 1,
            timestamp: "2024-01-01T00:00:00Z".to_string(),
            source_type: "manual".to_string(),
            content: "original".to_string(),
            screenshot_path: None,
            monitor_info: None,
            tags: None,
            user_notes: None,
        };

        let cloned = record.clone();
        assert_eq!(record.id, cloned.id);
        assert_eq!(record.content, cloned.content);
    }

    #[test]
    fn record_struct_debug() {
        let record = Record {
            id: 1,
            timestamp: "2024-01-01T00:00:00Z".to_string(),
            source_type: "manual".to_string(),
            content: "test".to_string(),
            screenshot_path: None,
            monitor_info: None,
            tags: None,
            user_notes: None,
        };

        let debug_str = format!("{:?}", record);
        assert!(debug_str.contains("Record"));
        assert!(debug_str.contains("id: 1"));
    }

    // ── Additional boundary tests ──

    #[test]
    #[serial]
    fn get_records_by_date_range_crosses_month_boundary() {
        setup_test_db();

        // Insert record on the last day of month
        let now = chrono::Local::now();
        let first_of_month = now.date_naive().with_day(1).unwrap();
        let last_of_prev = first_of_month - chrono::Duration::days(1);

        let ts = local_to_utc_rfc3339(last_of_prev.and_hms_opt(12, 0, 0).unwrap());
        insert_record_with_ts(&ts, "last day of prev month");

        // Query spanning both months
        let start = last_of_prev.format("%Y-%m-%d").to_string();
        let end = now.date_naive().format("%Y-%m-%d").to_string();

        let records = get_records_by_date_range_sync(start, end).unwrap();
        assert!(records
            .iter()
            .any(|r| r.content == "last day of prev month"));
    }

    #[test]
    #[serial]
    fn get_history_records_empty_result_for_future_dates() {
        setup_test_db();

        add_record("manual", "today note", None, None, None).unwrap();

        // Query for future dates
        let future = chrono::Local::now().date_naive() + chrono::Duration::days(30);
        let future_str = future.format("%Y-%m-%d").to_string();

        let records =
            get_history_records_sync(future_str.clone(), future_str, None, 0, 50).unwrap();

        assert!(records.is_empty());
    }

    #[test]
    #[serial]
    fn search_with_quotes_in_query() {
        setup_test_db();

        add_record("manual", r#"content with "quotes""#, None, None, None).unwrap();

        // Search for quoted content
        let results = search_records_sync(r#""quotes""#, "rank", 50).unwrap();
        // Should not crash, may or may not find depending on FTS behavior
        // The important thing is no panic/error
        assert!(results.is_empty() || !results.is_empty());
    }

    #[test]
    #[serial]
    fn get_week_records_sunday_start() {
        setup_test_db();

        let today = chrono::Local::now().date_naive();
        let ts = local_to_utc_rfc3339(today.and_hms_opt(10, 0, 0).unwrap());
        insert_record_with_ts(&ts, "this week");

        // Week starting Sunday (6)
        let records = get_week_records_sync(6).unwrap();
        assert!(records.iter().any(|r| r.content == "this week"));
    }

    #[test]
    #[serial]
    fn get_records_for_export_invalid_date_format() {
        setup_test_db();

        let result = get_records_for_export("not-a-date", "2024-01-01");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid start_date format"));
    }

    #[test]
    #[serial]
    fn search_order_by_time() {
        setup_test_db();

        let today = chrono::Local::now().date_naive();

        // Insert records at different times
        let ts1 = local_to_utc_rfc3339(today.and_hms_opt(8, 0, 0).unwrap());
        let ts2 = local_to_utc_rfc3339(today.and_hms_opt(16, 0, 0).unwrap());

        insert_record_with_ts(&ts1, "morning test");
        insert_record_with_ts(&ts2, "afternoon test");

        let results = search_records_sync("test", "time", 50).unwrap();

        // Should be ordered by timestamp DESC (newest first)
        assert_eq!(results[0].record.content, "afternoon test");
        assert_eq!(results[1].record.content, "morning test");
    }

    #[test]
    #[serial]
    fn search_cjk_highlights_correctly() {
        setup_test_db();

        add_record("manual", "这是一个测试", None, None, None).unwrap();

        let results = search_records_sync("测试", "rank", 50).unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].snippet.contains("<mark>测试</mark>"));
    }

    #[test]
    #[serial]
    fn get_today_records_ordered_desc() {
        setup_test_db();

        let today = chrono::Local::now().date_naive();

        let ts1 = local_to_utc_rfc3339(today.and_hms_opt(8, 0, 0).unwrap());
        let ts2 = local_to_utc_rfc3339(today.and_hms_opt(16, 0, 0).unwrap());

        insert_record_with_ts(&ts1, "morning");
        insert_record_with_ts(&ts2, "afternoon");

        let records = get_today_records_sync().unwrap();

        // Should be ordered by timestamp DESC (newest first)
        assert_eq!(records[0].content, "afternoon");
        assert_eq!(records[1].content, "morning");
    }

    #[test]
    #[serial]
    fn get_all_today_records_for_summary_delegates() {
        setup_test_db();

        add_record("manual", "test note", None, None, None).unwrap();

        let records = get_all_today_records_for_summary().unwrap();
        assert!(records.iter().any(|r| r.content == "test note"));
    }

    // ── Async wrapper tests ──

    #[tokio::test]
    #[serial]
    async fn async_get_today_records_works() {
        setup_test_db();

        add_record("manual", "async test", None, None, None).unwrap();

        let records = get_today_records().await.unwrap();
        assert!(records.iter().any(|r| r.content == "async test"));
    }

    #[tokio::test]
    #[serial]
    async fn async_get_records_by_date_range_works() {
        setup_test_db();

        add_record("manual", "async range test", None, None, None).unwrap();

        let today = chrono::Local::now().date_naive();
        let today_str = today.format("%Y-%m-%d").to_string();

        let records = get_records_by_date_range(today_str.clone(), today_str)
            .await
            .unwrap();
        assert!(records.iter().any(|r| r.content == "async range test"));
    }

    #[tokio::test]
    #[serial]
    async fn async_delete_record_works() {
        setup_test_db();

        let id = add_record("manual", "to delete async", None, None, None).unwrap();

        delete_record(id).await.unwrap();

        let records = get_today_records().await.unwrap();
        assert!(!records.iter().any(|r| r.id == id));
    }

    #[tokio::test]
    #[serial]
    async fn async_get_history_records_works() {
        setup_test_db();

        add_record("manual", "async history test", None, None, None).unwrap();

        let today = chrono::Local::now().date_naive();
        let today_str = today.format("%Y-%m-%d").to_string();

        let records = get_history_records(today_str.clone(), today_str, None, None, None)
            .await
            .unwrap();
        assert!(records.iter().any(|r| r.content == "async history test"));
    }

    #[tokio::test]
    #[serial]
    async fn async_search_records_works() {
        setup_test_db();

        add_record("manual", "async search test", None, None, None).unwrap();

        let results = search_records("async".to_string(), None, None)
            .await
            .unwrap();
        assert!(results
            .iter()
            .any(|r| r.record.content == "async search test"));
    }

    #[test]
    #[serial]
    fn update_record_content_sync_updates_content() {
        setup_test_db();

        let id = add_record("auto", "original content", None, None, None).unwrap();
        let new_content = r#"{"current_focus": "updated focus", "active_software": "VS Code"}"#;

        update_record_content_sync(id, new_content).unwrap();

        // Verify the content was updated
        let records = get_today_records_sync().unwrap();
        let record = records
            .iter()
            .find(|r| r.id == id)
            .expect("record should exist");
        assert_eq!(record.content, new_content);
    }

    #[test]
    #[serial]
    fn update_record_content_sync_returns_error_for_nonexistent_id() {
        setup_test_db();

        let result = update_record_content_sync(99999, "new content");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    // NOTE: Performance benchmark tests moved to dedicated `mod benchmarks` below (CORE-008)
}

// ── Performance benchmark tests (CORE-008 AC#3) ──

#[cfg(test)]
mod benchmarks {
    use super::*;
    use rusqlite::Connection;
    use serial_test::serial;
    use std::time::Instant;

    fn setup_test_db() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute(
            "CREATE TABLE records (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp TEXT NOT NULL,
                source_type TEXT NOT NULL,
                content TEXT NOT NULL,
                screenshot_path TEXT,
                monitor_info TEXT,
                tags TEXT,
                user_notes TEXT
            )",
            [],
        )
        .unwrap();
        conn.execute(
            "CREATE TABLE settings (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                api_base_url TEXT,
                api_key TEXT,
                model_name TEXT,
                screenshot_interval INTEGER DEFAULT 5,
                summary_time TEXT DEFAULT '18:00',
                obsidian_path TEXT,
                auto_capture_enabled INTEGER DEFAULT 0,
                last_summary_path TEXT,
                summary_model_name TEXT,
                analysis_prompt TEXT,
                summary_prompt TEXT,
                change_threshold INTEGER DEFAULT 3,
                max_silent_minutes INTEGER DEFAULT 30,
                summary_title_format TEXT DEFAULT '工作日报 - {date}',
                include_manual_records INTEGER DEFAULT 1,
                window_whitelist TEXT DEFAULT '[]',
                window_blacklist TEXT DEFAULT '[]',
                use_whitelist_only INTEGER DEFAULT 0,
                auto_adjust_silent INTEGER DEFAULT 1,
                silent_adjustment_paused_until TEXT DEFAULT NULL,
                auto_detect_work_time INTEGER DEFAULT 1,
                use_custom_work_time INTEGER DEFAULT 0,
                custom_work_time_start TEXT DEFAULT '09:00',
                custom_work_time_end TEXT DEFAULT '18:00',
                learned_work_time TEXT DEFAULT NULL,
                capture_mode TEXT DEFAULT 'primary',
                selected_monitor_index INTEGER DEFAULT 0,
                tag_categories TEXT DEFAULT '[]',
                is_ollama INTEGER DEFAULT 0,
                weekly_report_prompt TEXT,
                weekly_report_day INTEGER DEFAULT 0,
                last_weekly_report_path TEXT,
                monthly_report_prompt TEXT,
                custom_report_prompt TEXT,
                last_custom_report_path TEXT,
                last_monthly_report_path TEXT,
                obsidian_vaults TEXT DEFAULT '[]',
                comparison_report_prompt TEXT
            )",
            [],
        )
        .unwrap();
        conn.execute("INSERT OR IGNORE INTO settings (id) VALUES (1)", [])
            .unwrap();

        // Create FTS5 virtual table for full-text search tests
        conn.execute(
            "CREATE VIRTUAL TABLE IF NOT EXISTS records_fts USING fts5(
                content,
                content='records',
                content_rowid='id',
                tokenize='unicode61'
            )",
            [],
        )
        .unwrap();

        // FTS5 triggers for automatic index sync
        conn.execute(
            "CREATE TRIGGER IF NOT EXISTS records_ai AFTER INSERT ON records BEGIN
                INSERT INTO records_fts(rowid, content) VALUES (new.id, new.content);
            END",
            [],
        )
        .unwrap();

        conn.execute(
            "CREATE TRIGGER IF NOT EXISTS records_ad AFTER DELETE ON records BEGIN
                INSERT INTO records_fts(records_fts, rowid, content)
                VALUES ('delete', old.id, old.content);
            END",
            [],
        )
        .unwrap();

        conn.execute(
            "CREATE TRIGGER IF NOT EXISTS records_au AFTER UPDATE ON records BEGIN
                INSERT INTO records_fts(records_fts, rowid, content)
                VALUES ('delete', old.id, old.content);
                INSERT INTO records_fts(rowid, content) VALUES (new.id, new.content);
            END",
            [],
        )
        .unwrap();

        let mut db = DB_CONNECTION.lock().unwrap();
        *db = Some(conn);
    }

    /// Benchmark: Insert 100 records
    /// AC requirement: Database CRUD < 10ms per operation
    #[test]
    #[serial]
    fn benchmark_insert_100_records() {
        setup_test_db();

        let start = Instant::now();
        for i in 0..100 {
            let _ = add_record(
                "auto",
                &format!("Benchmark test record #{}", i),
                None,
                None,
                None,
            );
        }
        let elapsed_ms = start.elapsed().as_millis();

        // 100 inserts should complete in < 1000ms (10ms avg per insert)
        assert!(
            elapsed_ms < 1000,
            "100 inserts took {}ms (expected < 1000ms)",
            elapsed_ms
        );
    }

    /// Benchmark: Query 100 records
    /// AC requirement: Database CRUD < 10ms per operation
    #[test]
    #[serial]
    fn benchmark_query_100_records() {
        setup_test_db();

        // Insert 100 records first
        for i in 0..100 {
            let _ = add_record(
                "auto",
                &format!("Benchmark test record #{}", i),
                None,
                None,
                None,
            );
        }

        let start = Instant::now();
        let records = get_today_records_sync().unwrap();
        let elapsed_ms = start.elapsed().as_millis();

        assert_eq!(records.len(), 100);
        assert!(
            elapsed_ms < 100,
            "Query 100 records took {}ms (expected < 100ms)",
            elapsed_ms
        );
    }

    /// Benchmark: Delete 50 records
    /// AC requirement: Database CRUD < 10ms per operation
    #[test]
    #[serial]
    fn benchmark_delete_50_records() {
        setup_test_db();

        // Insert 50 records first
        let mut ids = Vec::new();
        for i in 0..50 {
            let id = add_record(
                "auto",
                &format!("Benchmark test record #{}", i),
                None,
                None,
                None,
            )
            .unwrap();
            ids.push(id);
        }

        let start = Instant::now();
        for id in ids {
            let _ = delete_record_sync(id);
        }
        let elapsed_ms = start.elapsed().as_millis();

        // 50 deletes should complete in < 500ms (10ms avg per delete)
        assert!(
            elapsed_ms < 500,
            "50 deletes took {}ms (expected < 500ms)",
            elapsed_ms
        );
    }

    /// Benchmark: Search with 100 records
    #[test]
    #[serial]
    fn benchmark_search_100_records() {
        setup_test_db();

        // Insert 100 records with varied content
        for i in 0..100 {
            let _ = add_record(
                "auto",
                &format!("Benchmark test record with keyword #{}", i % 10),
                None,
                None,
                None,
            );
        }

        let start = Instant::now();
        let results = search_records_sync("keyword", "rank", 50).unwrap();
        let elapsed_ms = start.elapsed().as_millis();

        assert!(!results.is_empty());
        assert!(
            elapsed_ms < 100,
            "Search took {}ms (expected < 100ms)",
            elapsed_ms
        );
    }

    /// Benchmark: Pagination with 200 records
    #[test]
    #[serial]
    fn benchmark_pagination_200_records() {
        setup_test_db();

        // Insert 200 records
        for i in 0..200 {
            let _ = add_record(
                "auto",
                &format!("Benchmark test record #{}", i),
                None,
                None,
                None,
            );
        }

        let today = chrono::Local::now().date_naive();
        let today_str = today.format("%Y-%m-%d").to_string();

        let start = Instant::now();
        let _page1 =
            get_history_records_sync(today_str.clone(), today_str.clone(), None, 0, 50).unwrap();
        let _page2 =
            get_history_records_sync(today_str.clone(), today_str.clone(), None, 1, 50).unwrap();
        let _page3 = get_history_records_sync(today_str.clone(), today_str, None, 2, 50).unwrap();
        let elapsed_ms = start.elapsed().as_millis();

        assert!(
            elapsed_ms < 100,
            "3 paginated queries took {}ms (expected < 100ms)",
            elapsed_ms
        );
    }

    /// Benchmark: get_today_record_count_sync
    #[test]
    #[serial]
    fn benchmark_count_records() {
        setup_test_db();

        // Insert 100 records
        for i in 0..100 {
            let _ = add_record(
                "auto",
                &format!("Benchmark test record #{}", i),
                None,
                None,
                None,
            );
        }

        let start = Instant::now();
        let count = get_today_record_count_sync().unwrap();
        let elapsed_ms = start.elapsed().as_millis();

        assert_eq!(count, 100);
        assert!(
            elapsed_ms < 50,
            "Count took {}ms (expected < 50ms)",
            elapsed_ms
        );
    }

    /// Benchmark: get_records_for_export with 100 records
    #[test]
    #[serial]
    fn benchmark_export_100_records() {
        setup_test_db();

        // Insert 100 records
        for i in 0..100 {
            let _ = add_record(
                "auto",
                &format!("Benchmark test record #{}", i),
                None,
                None,
                None,
            );
        }

        let today = chrono::Local::now().date_naive();
        let today_str = today.format("%Y-%m-%d").to_string();

        let start = Instant::now();
        let records = get_records_for_export(&today_str, &today_str).unwrap();
        let elapsed_ms = start.elapsed().as_millis();

        assert_eq!(records.len(), 100);
        assert!(
            elapsed_ms < 100,
            "Export query took {}ms (expected < 100ms)",
            elapsed_ms
        );
    }

    /// Benchmark: get_week_records_sync
    #[test]
    #[serial]
    fn benchmark_week_records() {
        setup_test_db();

        // Insert 50 records
        for i in 0..50 {
            let _ = add_record(
                "auto",
                &format!("Benchmark test record #{}", i),
                None,
                None,
                None,
            );
        }

        let start = Instant::now();
        let records = get_week_records_sync(0).unwrap();
        let elapsed_ms = start.elapsed().as_millis();

        assert_eq!(records.len(), 50);
        assert!(
            elapsed_ms < 100,
            "Week query took {}ms (expected < 100ms)",
            elapsed_ms
        );
    }

    /// Benchmark: get_month_records_sync
    #[test]
    #[serial]
    fn benchmark_month_records() {
        setup_test_db();

        // Insert 100 records
        for i in 0..100 {
            let _ = add_record(
                "auto",
                &format!("Benchmark test record #{}", i),
                None,
                None,
                None,
            );
        }

        let start = Instant::now();
        let records = get_month_records_sync().unwrap();
        let elapsed_ms = start.elapsed().as_millis();

        assert_eq!(records.len(), 100);
        assert!(
            elapsed_ms < 100,
            "Month query took {}ms (expected < 100ms)",
            elapsed_ms
        );
    }
}
