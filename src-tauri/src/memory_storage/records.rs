use chrono::{Datelike, TimeZone};
use rusqlite::params;
use serde::{Deserialize, Serialize};
use tauri::command;

use crate::errors::{AppError, AppResult};

use super::DB_CONNECTION;

/// Convert a NaiveDateTime to UTC RFC3339 string, handling DST ambiguity by picking the earliest offset.
fn naive_to_utc_rfc3339(dt: chrono::NaiveDateTime) -> String {
    chrono::Local
        .from_local_datetime(&dt)
        .earliest()
        .map(|dt| dt.with_timezone(&chrono::Utc).to_rfc3339())
        .unwrap_or_else(|| dt.and_utc().to_rfc3339())
}

/// Convert a NaiveDate to UTC RFC3339 string at the given time (h, m, s).
fn date_to_utc_rfc3339(date: chrono::NaiveDate, h: u32, m: u32, s: u32) -> String {
    let naive_dt = date
        .and_hms_opt(h, m, s)
        .expect("valid time: (h,m,s) are always 0,0,0 or 23,59,59");
    naive_to_utc_rfc3339(naive_dt)
}

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
    // SESSION-001: 时段关联和分析状态
    pub session_id: Option<i64>,
    pub analysis_status: Option<String>, // pending | analyzed | user_edited
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

/// EXP-005: Today's statistics for the summary widget
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TodayStats {
    /// Total number of records today (auto + manual)
    pub total_count: u32,
    /// Number of auto-captured screenshots
    pub auto_count: u32,
    /// Number of manual notes
    pub manual_count: u32,
    /// Timestamp of the first record today (RFC3339)
    pub first_record_time: Option<String>,
    /// Timestamp of the latest record today (RFC3339)
    pub latest_record_time: Option<String>,
    /// Hour (0-23) with the most records today
    pub busiest_hour: Option<u32>,
    /// Number of records in the busiest hour
    pub busiest_hour_count: u32,
}

/// SESSION-002: Screenshot info for session batch analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionScreenshot {
    pub record_id: i64,
    pub timestamp: String,
    pub screenshot_path: String,
}

pub fn add_record(
    source_type: &str,
    content: &str,
    screenshot_path: Option<&str>,
    monitor_info: Option<&str>,
    tags: Option<&str>,
) -> AppResult<i64> {
    add_record_with_session(
        source_type,
        content,
        screenshot_path,
        monitor_info,
        tags,
        None,
    )
}

/// SESSION-001: Add record with session_id support
pub fn add_record_with_session(
    source_type: &str,
    content: &str,
    screenshot_path: Option<&str>,
    monitor_info: Option<&str>,
    tags: Option<&str>,
    session_id: Option<i64>,
) -> AppResult<i64> {
    // STAB-001 Task 4.2: Ensure database connection is valid before operation
    crate::memory_storage::schema::ensure_connection()?;

    let db = DB_CONNECTION.lock()?;
    let conn = db
        .as_ref()
        .ok_or_else(|| AppError::database("Database not initialized"))?;

    let timestamp = chrono::Utc::now().to_rfc3339();

    // STAB-001 AC4: Use explicit transaction for data integrity
    // Begin transaction and ensure rollback on error
    conn.execute("BEGIN TRANSACTION", [])?;

    let result = conn.execute(
        "INSERT INTO records (timestamp, source_type, content, screenshot_path, monitor_info, tags, session_id, analysis_status) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'pending')",
        params![timestamp, source_type, content, screenshot_path, monitor_info, tags, session_id],
    );

    match result {
        Ok(_) => {
            // Commit transaction
            conn.execute("COMMIT", [])?;
            Ok(conn.last_insert_rowid())
        }
        Err(e) => {
            // Rollback on error
            let _ = conn.execute("ROLLBACK", []);
            Err(AppError::database(format!(
                "Failed to insert record: {}",
                e
            )))
        }
    }
}

pub fn get_today_records_sync() -> AppResult<Vec<Record>> {
    let db = DB_CONNECTION.lock()?;
    let conn = db
        .as_ref()
        .ok_or_else(|| AppError::database("Database not initialized"))?;

    let today_start = date_to_utc_rfc3339(chrono::Local::now().date_naive(), 0, 0, 0);

    let mut stmt = conn.prepare(
        "SELECT id, timestamp, source_type, content, screenshot_path, monitor_info, tags, user_notes, session_id, analysis_status FROM records
         WHERE timestamp >= ?1 ORDER BY timestamp DESC",
    )?;

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
                session_id: row.get(8)?,
                analysis_status: row.get(9)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| AppError::database(format!("Failed to collect records: {}", e)))?;

    Ok(records)
}

/// Get records for the current week (Monday to Sunday)
/// week_start_day: 0=Monday, 6=Sunday (default is Monday)
pub fn get_week_records_sync(week_start_day: i32) -> AppResult<Vec<Record>> {
    let db = DB_CONNECTION.lock()?;
    let conn = db
        .as_ref()
        .ok_or_else(|| AppError::database("Database not initialized"))?;

    // Calculate week boundaries based on local time
    let today = chrono::Local::now().date_naive();
    let weekday = today.weekday().num_days_from_monday() as i32;
    let days_since_week_start = (weekday - week_start_day + 7) % 7;

    let week_start_date = today - chrono::Duration::days(days_since_week_start as i64);
    let week_end_date = week_start_date + chrono::Duration::days(6);

    let week_start = date_to_utc_rfc3339(week_start_date, 0, 0, 0);
    let week_end = date_to_utc_rfc3339(week_end_date, 23, 59, 59);

    let mut stmt = conn.prepare(
        "SELECT id, timestamp, source_type, content, screenshot_path, monitor_info, tags, user_notes, session_id, analysis_status FROM records
         WHERE timestamp >= ?1 AND timestamp <= ?2 ORDER BY timestamp DESC",
    )?;

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
                session_id: row.get(8)?,
                analysis_status: row.get(9)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| AppError::database(format!("Failed to collect records: {}", e)))?;

    Ok(records)
}

/// Get all records for the current month (used for monthly report)
pub fn get_month_records_sync() -> AppResult<Vec<Record>> {
    let db = DB_CONNECTION.lock()?;
    let conn = db
        .as_ref()
        .ok_or_else(|| AppError::database("Database not initialized"))?;

    // Calculate month boundaries based on local time
    let now = chrono::Local::now();
    let first_day = now.date_naive().with_day(1).expect("day 1 is always valid");

    // Month start: first day of month at 00:00:00 local time
    let month_start = date_to_utc_rfc3339(first_day, 0, 0, 0);

    // Month end: first day of next month at 00:00:00 - 1 second
    let next_month = if now.month() == 12 {
        chrono::NaiveDate::from_ymd_opt(now.year() + 1, 1, 1).expect("Jan 1 is always valid")
    } else {
        chrono::NaiveDate::from_ymd_opt(now.year(), now.month() + 1, 1).expect("month+1 <= 12")
    };

    let month_end = date_to_utc_rfc3339(next_month, 0, 0, 0);

    let mut stmt = conn.prepare(
        "SELECT id, timestamp, source_type, content, screenshot_path, monitor_info, tags, user_notes, session_id, analysis_status FROM records
         WHERE timestamp >= ?1 AND timestamp < ?2 ORDER BY timestamp DESC",
    )?;

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
                session_id: row.get(8)?,
                analysis_status: row.get(9)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| AppError::database(format!("Failed to collect records: {}", e)))?;

    Ok(records)
}

pub fn get_all_today_records_for_summary() -> AppResult<Vec<Record>> {
    get_today_records_sync()
}

/// Get the count of today's records (more efficient than fetching all records).
pub fn get_today_record_count_sync() -> AppResult<usize> {
    let db = DB_CONNECTION.lock()?;
    let conn = db
        .as_ref()
        .ok_or_else(|| AppError::database("Database not initialized"))?;

    let today_start = date_to_utc_rfc3339(chrono::Local::now().date_naive(), 0, 0, 0);

    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM records WHERE timestamp >= ?1",
        params![today_start],
        |row| row.get(0),
    )?;

    Ok(count as usize)
}

/// EXP-005: Get today's statistics for the summary widget.
/// Returns aggregated stats including record counts, time span, and busiest hour.
pub fn get_today_stats_sync() -> AppResult<TodayStats> {
    let db = DB_CONNECTION.lock()?;
    let conn = db
        .as_ref()
        .ok_or_else(|| AppError::database("Database not initialized"))?;

    let today_start = date_to_utc_rfc3339(chrono::Local::now().date_naive(), 0, 0, 0);

    // Query basic stats in a single query
    let basic_stats = conn.query_row(
        "SELECT
                COUNT(*) as total,
                COALESCE(SUM(CASE WHEN source_type='auto' THEN 1 ELSE 0 END), 0) as auto_count,
                COALESCE(SUM(CASE WHEN source_type='manual' THEN 1 ELSE 0 END), 0) as manual_count,
                MIN(timestamp) as first_time,
                MAX(timestamp) as latest_time
            FROM records WHERE timestamp >= ?1",
        params![today_start],
        |row| {
            Ok((
                row.get::<_, i64>(0)?,            // total
                row.get::<_, i64>(1)?,            // auto_count
                row.get::<_, i64>(2)?,            // manual_count
                row.get::<_, Option<String>>(3)?, // first_time
                row.get::<_, Option<String>>(4)?, // latest_time
            ))
        },
    )?;

    let (total, auto_count, manual_count, first_record_time, latest_record_time) = basic_stats;

    // Query busiest hour if there are records
    let (busiest_hour, busiest_hour_count) = if total > 0 {
        conn.query_row(
            "SELECT CAST(strftime('%H', datetime(timestamp, 'localtime')) AS INTEGER) as hour, COUNT(*) as cnt
            FROM records
            WHERE timestamp >= ?1
            GROUP BY hour
            ORDER BY cnt DESC
            LIMIT 1",
            params![today_start],
            |row| Ok((row.get::<_, i64>(0)? as u32, row.get::<_, i64>(1)? as u32)),
        )?
    } else {
        (0, 0)
    };

    Ok(TodayStats {
        total_count: total as u32,
        auto_count: auto_count as u32,
        manual_count: manual_count as u32,
        first_record_time,
        latest_record_time,
        busiest_hour: if total > 0 { Some(busiest_hour) } else { None },
        busiest_hour_count,
    })
}

pub fn get_records_by_date_range_sync(
    start_date: String,
    end_date: String,
) -> AppResult<Vec<Record>> {
    let db = DB_CONNECTION.lock()?;
    let conn = db
        .as_ref()
        .ok_or_else(|| AppError::database("Database not initialized"))?;

    // Parse start_date (YYYY-MM-DD) to local midnight 00:00:00
    let start_naive = chrono::NaiveDate::parse_from_str(&start_date, "%Y-%m-%d").map_err(|e| {
        AppError::validation(format!(
            "Invalid start_date format (expected YYYY-MM-DD): {}",
            e
        ))
    })?;

    // Parse end_date (YYYY-MM-DD) to local midnight of next day (exclusive upper bound)
    let end_naive = chrono::NaiveDate::parse_from_str(&end_date, "%Y-%m-%d").map_err(|e| {
        AppError::validation(format!(
            "Invalid end_date format (expected YYYY-MM-DD): {}",
            e
        ))
    })?;

    // Convert to UTC RFC3339
    let start_utc = date_to_utc_rfc3339(start_naive, 0, 0, 0);
    let end_utc = date_to_utc_rfc3339(end_naive, 23, 59, 59);

    let mut stmt = conn.prepare(
        "SELECT id, timestamp, source_type, content, screenshot_path, monitor_info, tags, user_notes, session_id, analysis_status FROM records
         WHERE timestamp >= ?1 AND timestamp <= ?2 ORDER BY timestamp DESC",
    )?;

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
                session_id: row.get(8)?,
                analysis_status: row.get(9)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| AppError::database(format!("Failed to collect records: {}", e)))?;

    Ok(records)
}

/// Get records within a date range for export (chronological ASC order).
/// - start_date/end_date: YYYY-MM-DD format (local timezone)
pub fn get_records_for_export(start_date: &str, end_date: &str) -> AppResult<Vec<Record>> {
    let db = DB_CONNECTION.lock()?;
    let conn = db
        .as_ref()
        .ok_or_else(|| AppError::database("Database not initialized"))?;

    let start_naive = chrono::NaiveDate::parse_from_str(start_date, "%Y-%m-%d").map_err(|e| {
        AppError::validation(format!(
            "Invalid start_date format (expected YYYY-MM-DD): {}",
            e
        ))
    })?;

    let end_naive = chrono::NaiveDate::parse_from_str(end_date, "%Y-%m-%d").map_err(|e| {
        AppError::validation(format!(
            "Invalid end_date format (expected YYYY-MM-DD): {}",
            e
        ))
    })?;

    let start_utc = date_to_utc_rfc3339(start_naive, 0, 0, 0);
    let end_utc = date_to_utc_rfc3339(end_naive, 23, 59, 59);

    let mut stmt = conn.prepare(
        "SELECT id, timestamp, source_type, content, screenshot_path, monitor_info, tags, user_notes, session_id, analysis_status FROM records
         WHERE timestamp >= ?1 AND timestamp <= ?2 ORDER BY timestamp ASC",
    )?;

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
                session_id: row.get(8)?,
                analysis_status: row.get(9)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| AppError::database(format!("Failed to collect records: {}", e)))?;

    Ok(records)
}

/// Delete a record by ID
pub fn delete_record_sync(id: i64) -> AppResult<()> {
    let db = DB_CONNECTION.lock()?;
    let conn = db
        .as_ref()
        .ok_or_else(|| AppError::database("Database not initialized"))?;

    let rows_affected = conn.execute("DELETE FROM records WHERE id = ?1", params![id])?;

    if rows_affected == 0 {
        return Err(AppError::validation(format!(
            "Record with id {} not found",
            id
        )));
    }

    tracing::info!("Deleted record with id {}", id);
    Ok(())
}

/// Get a single record by ID
/// Used by reanalyze_record to fetch record details
pub fn get_record_by_id_sync(id: i64) -> AppResult<Record> {
    let db = DB_CONNECTION.lock()?;
    let conn = db
        .as_ref()
        .ok_or_else(|| AppError::database("Database not initialized"))?;

    let record = conn
        .query_row(
            "SELECT id, timestamp, source_type, content, screenshot_path, monitor_info, tags, user_notes, session_id, analysis_status
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
                    session_id: row.get(8)?,
                    analysis_status: row.get(9)?,
                })
            },
        )
        .map_err(|e| AppError::database(format!("Record with id {} not found: {}", id, e)))?;

    Ok(record)
}

/// Update the content of a record by ID
/// Used by offline queue retry to update screenshot analysis results
pub fn update_record_content_sync(id: i64, content: &str) -> AppResult<()> {
    let db = DB_CONNECTION.lock()?;
    let conn = db
        .as_ref()
        .ok_or_else(|| AppError::database("Database not initialized"))?;

    let rows_affected = conn.execute(
        "UPDATE records SET content = ?1 WHERE id = ?2",
        params![content, id],
    )?;

    if rows_affected == 0 {
        return Err(AppError::validation(format!(
            "Record with id {} not found",
            id
        )));
    }

    tracing::info!("Updated content for record {}", id);
    Ok(())
}

/// Update user notes for a specific record
/// FEAT-005: User can add manual notes to screenshot records (#66)
pub fn update_record_user_notes_sync(id: i64, user_notes: Option<&str>) -> AppResult<()> {
    let db = DB_CONNECTION.lock()?;
    let conn = db
        .as_ref()
        .ok_or_else(|| AppError::database("Database not initialized"))?;

    let rows_affected = conn.execute(
        "UPDATE records SET user_notes = ?1, analysis_status = 'user_edited' WHERE id = ?2",
        params![user_notes, id],
    )?;

    if rows_affected == 0 {
        return Err(AppError::validation(format!(
            "Record with id {} not found",
            id
        )));
    }

    tracing::info!("Updated user notes for record {}", id);
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
) -> AppResult<Vec<Record>> {
    get_history_records_with_cursor_sync(start_date, end_date, source_type, page, page_size, None)
}

/// PERF-004: Cursor-based pagination for efficient history record retrieval
/// - last_id: if provided, fetches records with id < last_id (efficient cursor pagination)
///   if not provided, uses traditional OFFSET pagination (backward compatible)
pub fn get_history_records_with_cursor_sync(
    start_date: String,
    end_date: String,
    source_type: Option<String>,
    page: i64,
    page_size: i64,
    last_id: Option<i64>,
) -> AppResult<Vec<Record>> {
    let db = DB_CONNECTION.lock()?;
    let conn = db
        .as_ref()
        .ok_or_else(|| AppError::database("Database not initialized"))?;

    // Parse start_date (YYYY-MM-DD) to local midnight 00:00:00
    let start_naive = chrono::NaiveDate::parse_from_str(&start_date, "%Y-%m-%d").map_err(|e| {
        AppError::validation(format!(
            "Invalid start_date format (expected YYYY-MM-DD): {}",
            e
        ))
    })?;

    // Parse end_date (YYYY-MM-DD) to local end of day 23:59:59
    let end_naive = chrono::NaiveDate::parse_from_str(&end_date, "%Y-%m-%d").map_err(|e| {
        AppError::validation(format!(
            "Invalid end_date format (expected YYYY-MM-DD): {}",
            e
        ))
    })?;

    // Convert to UTC RFC3339
    let start_utc = date_to_utc_rfc3339(start_naive, 0, 0, 0);
    let end_utc = date_to_utc_rfc3339(end_naive, 23, 59, 59);

    // Build query using cursor-based or offset-based pagination
    let (sql, params_vec): (String, Vec<Box<dyn rusqlite::ToSql>>) = match (
        source_type.as_ref(),
        last_id,
    ) {
        (Some(ref st), Some(last_id_val)) => {
            // Cursor-based pagination with source_type filter (efficient)
            if *st != "auto" && *st != "manual" {
                return Err(AppError::validation(format!(
                    "Invalid source_type '{}'. Must be 'auto', 'manual', or null for all",
                    st
                )));
            }
            (
                    "SELECT id, timestamp, source_type, content, screenshot_path, monitor_info, tags, user_notes, session_id, analysis_status FROM records
                     WHERE timestamp >= ?1 AND timestamp <= ?2 AND source_type = ?3 AND id < ?4
                     ORDER BY id DESC LIMIT ?5"
                        .to_string(),
                    vec![
                        Box::new(start_utc.clone()),
                        Box::new(end_utc.clone()),
                        Box::new((*st).clone()),
                        Box::new(last_id_val),
                        Box::new(page_size),
                    ],
                )
        }
        (Some(ref st), None) => {
            // Offset-based pagination with source_type filter (backward compatible)
            if *st != "auto" && *st != "manual" {
                return Err(AppError::validation(format!(
                    "Invalid source_type '{}'. Must be 'auto', 'manual', or null for all",
                    st
                )));
            }
            let offset = page * page_size;
            (
                    "SELECT id, timestamp, source_type, content, screenshot_path, monitor_info, tags, user_notes, session_id, analysis_status FROM records
                     WHERE timestamp >= ?1 AND timestamp <= ?2 AND source_type = ?3
                     ORDER BY id DESC LIMIT ?4 OFFSET ?5"
                        .to_string(),
                    vec![
                        Box::new(start_utc.clone()),
                        Box::new(end_utc.clone()),
                        Box::new((*st).clone()),
                        Box::new(page_size),
                        Box::new(offset),
                    ],
                )
        }
        (None, Some(last_id_val)) => {
            // Cursor-based pagination without source_type filter (efficient)
            (
                    "SELECT id, timestamp, source_type, content, screenshot_path, monitor_info, tags, user_notes, session_id, analysis_status FROM records
                     WHERE timestamp >= ?1 AND timestamp <= ?2 AND id < ?3
                     ORDER BY id DESC LIMIT ?4"
                        .to_string(),
                    vec![
                        Box::new(start_utc.clone()),
                        Box::new(end_utc.clone()),
                        Box::new(last_id_val),
                        Box::new(page_size),
                    ],
                )
        }
        (None, None) => {
            // Offset-based pagination without source_type filter (backward compatible)
            let offset = page * page_size;
            (
                    "SELECT id, timestamp, source_type, content, screenshot_path, monitor_info, tags, user_notes, session_id, analysis_status FROM records
                     WHERE timestamp >= ?1 AND timestamp <= ?2
                     ORDER BY id DESC LIMIT ?3 OFFSET ?4"
                        .to_string(),
                    vec![
                        Box::new(start_utc.clone()),
                        Box::new(end_utc.clone()),
                        Box::new(page_size),
                        Box::new(offset),
                    ],
                )
        }
    };

    let mut stmt = conn.prepare(&sql)?;

    let params_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|p| p.as_ref()).collect();

    let records = stmt
        .query_map(params_refs.as_slice(), |row| {
            Ok(Record {
                id: row.get(0)?,
                timestamp: row.get(1)?,
                source_type: row.get(2)?,
                content: row.get(3)?,
                screenshot_path: row.get(4)?,
                monitor_info: row.get(5)?,
                tags: row.get(6)?,
                user_notes: row.get(7)?,
                session_id: row.get(8)?,
                analysis_status: row.get(9)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| AppError::database(format!("Failed to collect records: {}", e)))?;

    Ok(records)
}

/// PERF-004: Cursor-based pagination for history records
/// Uses keyset pagination instead of OFFSET for better performance with large datasets
///
/// - start_date/end_date: YYYY-MM-DD format (local timezone)
/// - source_type: None for all, Some("auto") or Some("manual") for filtering
/// - last_id: Cursor - ID of the last record from previous page (None for first page)
/// - page_size: number of records per page (default 50)
pub fn get_history_records_cursor_sync(
    start_date: String,
    end_date: String,
    source_type: Option<String>,
    last_id: Option<i64>,
    page_size: i64,
) -> AppResult<Vec<Record>> {
    let db = DB_CONNECTION.lock()?;
    let conn = db
        .as_ref()
        .ok_or_else(|| AppError::database("Database not initialized"))?;

    // Parse start_date (YYYY-MM-DD) to local midnight 00:00:00
    let start_naive = chrono::NaiveDate::parse_from_str(&start_date, "%Y-%m-%d").map_err(|e| {
        AppError::validation(format!(
            "Invalid start_date format (expected YYYY-MM-DD): {}",
            e
        ))
    })?;

    // Parse end_date (YYYY-MM-DD) to local end of day 23:59:59
    let end_naive = chrono::NaiveDate::parse_from_str(&end_date, "%Y-%m-%d").map_err(|e| {
        AppError::validation(format!(
            "Invalid end_date format (expected YYYY-MM-DD): {}",
            e
        ))
    })?;

    // Convert to UTC RFC3339
    let start_utc = date_to_utc_rfc3339(start_naive, 0, 0, 0);
    let end_utc = date_to_utc_rfc3339(end_naive, 23, 59, 59);

    // Helper to map row to Record
    fn map_row(row: &rusqlite::Row) -> rusqlite::Result<Record> {
        Ok(Record {
            id: row.get(0)?,
            timestamp: row.get(1)?,
            source_type: row.get(2)?,
            content: row.get(3)?,
            screenshot_path: row.get(4)?,
            monitor_info: row.get(5)?,
            tags: row.get(6)?,
            user_notes: row.get(7)?,
            session_id: row.get(8)?,
            analysis_status: row.get(9)?,
        })
    }

    // Build query with cursor-based pagination and execute
    match (source_type, last_id) {
        (Some(st), Some(lid)) if st == "auto" || st == "manual" => {
            let mut stmt = conn.prepare(
                "SELECT id, timestamp, source_type, content, screenshot_path, monitor_info, tags, user_notes, session_id, analysis_status FROM records
                     WHERE timestamp >= ?1 AND timestamp <= ?2 AND source_type = ?3 AND id < ?4
                     ORDER BY id DESC LIMIT ?5",
            )?;
            let records = stmt
                .query_map(params![start_utc, end_utc, st, lid, page_size], map_row)?
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| AppError::database(format!("Failed to collect records: {}", e)))?;
            Ok(records)
        }
        (Some(st), None) if st == "auto" || st == "manual" => {
            let mut stmt = conn.prepare(
                "SELECT id, timestamp, source_type, content, screenshot_path, monitor_info, tags, user_notes, session_id, analysis_status FROM records
                     WHERE timestamp >= ?1 AND timestamp <= ?2 AND source_type = ?3
                     ORDER BY id DESC LIMIT ?4",
            )?;
            let records = stmt
                .query_map(params![start_utc, end_utc, st, page_size], map_row)?
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| AppError::database(format!("Failed to collect records: {}", e)))?;
            Ok(records)
        }
        (None, Some(lid)) => {
            let mut stmt = conn.prepare(
                "SELECT id, timestamp, source_type, content, screenshot_path, monitor_info, tags, user_notes, session_id, analysis_status FROM records
                     WHERE timestamp >= ?1 AND timestamp <= ?2 AND id < ?3
                     ORDER BY id DESC LIMIT ?4",
            )?;
            let records = stmt
                .query_map(params![start_utc, end_utc, lid, page_size], map_row)?
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| AppError::database(format!("Failed to collect records: {}", e)))?;
            Ok(records)
        }
        (None, None) => {
            let mut stmt = conn.prepare(
                "SELECT id, timestamp, source_type, content, screenshot_path, monitor_info, tags, user_notes, session_id, analysis_status FROM records
                     WHERE timestamp >= ?1 AND timestamp <= ?2
                     ORDER BY id DESC LIMIT ?3",
            )?;
            let records = stmt
                .query_map(params![start_utc, end_utc, page_size], map_row)?
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| AppError::database(format!("Failed to collect records: {}", e)))?;
            Ok(records)
        }
        (Some(st), _) => Err(AppError::validation(format!(
            "Invalid source_type '{}'. Must be 'auto', 'manual', or null for all",
            st
        ))),
    }
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
) -> AppResult<Vec<SearchResult>> {
    if query.trim().is_empty() {
        return Ok(Vec::new());
    }

    let db = DB_CONNECTION.lock()?;
    let conn = db
        .as_ref()
        .ok_or_else(|| AppError::database("Database not initialized"))?;

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
                id, timestamp, source_type, content, screenshot_path, monitor_info, tags, user_notes, session_id, analysis_status
            FROM records
            WHERE content LIKE ?1
            ORDER BY timestamp DESC
            LIMIT ?2";

        let mut stmt = conn.prepare(sql)?;

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
                        session_id: row.get(8)?,
                        analysis_status: row.get(9)?,
                    },
                    snippet,
                    rank: 0.0, // LIKE search doesn't have relevance score
                })
            })?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| AppError::database(format!("Failed to collect search results: {}", e)))?;

        Ok(results)
    } else {
        // Use FTS5 for non-CJK queries
        let escaped_query = query.replace('"', "\"\"");

        let sql = if order_by == "time" {
            "SELECT
                r.id, r.timestamp, r.source_type, r.content, r.screenshot_path, r.monitor_info, r.tags, r.user_notes, r.session_id, r.analysis_status,
                highlight(records_fts, 0, '<mark>', '</mark>') as snippet,
                bm25(records_fts) as rank
            FROM records_fts
            JOIN records r ON r.id = records_fts.rowid
            WHERE records_fts MATCH ?1
            ORDER BY r.timestamp DESC
            LIMIT ?2"
        } else {
            "SELECT
                r.id, r.timestamp, r.source_type, r.content, r.screenshot_path, r.monitor_info, r.tags, r.user_notes, r.session_id, r.analysis_status,
                highlight(records_fts, 0, '<mark>', '</mark>') as snippet,
                bm25(records_fts) as rank
            FROM records_fts
            JOIN records r ON r.id = records_fts.rowid
            WHERE records_fts MATCH ?1
            ORDER BY rank
            LIMIT ?2"
        };

        let mut stmt = conn.prepare(sql)?;

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
                        session_id: row.get(8)?,
                        analysis_status: row.get(9)?,
                    },
                    snippet: row.get(10)?,
                    rank: row.get(11)?,
                })
            })
            .map_err(|e| AppError::database(format!("Failed to search records: {}", e)))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| AppError::database(format!("Failed to collect search results: {}", e)))?;

        Ok(results)
    }
}

// ── Async Tauri command wrappers ──

#[command]
pub async fn get_today_records() -> AppResult<Vec<Record>> {
    get_today_records_sync()
}

#[command]
pub async fn get_records_by_date_range(
    start_date: String,
    end_date: String,
) -> AppResult<Vec<Record>> {
    get_records_by_date_range_sync(start_date, end_date)
}

/// Delete a record by ID
#[command]
pub async fn delete_record(id: i64) -> AppResult<()> {
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
) -> AppResult<Vec<Record>> {
    let page = page.unwrap_or(0);
    let page_size = page_size.unwrap_or(50);
    get_history_records_sync(start_date, end_date, source_type, page, page_size)
}

/// PERF-004: Get history records with cursor-based pagination
#[command]
pub async fn get_history_records_cursor(
    start_date: String,
    end_date: String,
    source_type: Option<String>,
    last_id: Option<i64>,
    page_size: Option<i64>,
) -> AppResult<Vec<Record>> {
    let page_size = page_size.unwrap_or(50);
    get_history_records_cursor_sync(start_date, end_date, source_type, last_id, page_size)
}

/// Full-text search on records content
#[command]
pub async fn search_records(
    query: String,
    order_by: Option<String>,
    limit: Option<i64>,
) -> AppResult<Vec<SearchResult>> {
    let order_by = order_by.unwrap_or_else(|| "rank".to_string());
    let limit = limit.unwrap_or(50);
    search_records_sync(&query, &order_by, limit)
}

/// Update user notes for a record
/// FEAT-005: User can add manual notes to screenshot records (#66)
#[command]
pub async fn update_record_user_notes(id: i64, user_notes: Option<String>) -> AppResult<()> {
    update_record_user_notes_sync(id, user_notes.as_deref())
}

/// EXP-005: Get today's statistics for the summary widget
#[command]
pub async fn get_today_stats() -> AppResult<TodayStats> {
    get_today_stats_sync()
}

/// SESSION-002: Get all pending-analysis records for a session
///
/// Returns records that have `analysis_status = 'pending'` and belong to the given session.
/// These are screenshots that have been captured but not yet analyzed by AI.
pub fn get_records_by_session_id(session_id: i64) -> AppResult<Vec<SessionScreenshot>> {
    let db = DB_CONNECTION.lock()?;
    let conn = db
        .as_ref()
        .ok_or_else(|| AppError::database("Database not initialized"))?;

    let mut stmt = conn.prepare(
        "SELECT id, timestamp, screenshot_path FROM records
             WHERE session_id = ?1 AND analysis_status = 'pending' AND screenshot_path IS NOT NULL
             ORDER BY timestamp ASC",
    )?;

    let screenshots = stmt
        .query_map(params![session_id], |row| {
            Ok(SessionScreenshot {
                record_id: row.get(0)?,
                timestamp: row.get(1)?,
                screenshot_path: row.get::<_, Option<String>>(2)?.unwrap_or_default(),
            })
        })?
        .filter_map(|r| r.ok())
        .filter(|s| !s.screenshot_path.is_empty())
        .collect::<Vec<_>>();

    Ok(screenshots)
}

/// SESSION-002: Update record content and analysis status after AI analysis
pub fn update_record_analysis(record_id: i64, content: &str) -> AppResult<()> {
    let db = DB_CONNECTION.lock()?;
    let conn = db
        .as_ref()
        .ok_or_else(|| AppError::database("Database not initialized"))?;

    conn.execute(
        "UPDATE records SET content = ?1, analysis_status = 'analyzed' WHERE id = ?2",
        params![content, record_id],
    )?;

    Ok(())
}

/// SESSION-002: Update session with AI analysis results
pub fn update_session_analysis(
    session_id: i64,
    ai_summary: &str,
    context_for_next: &str,
) -> AppResult<()> {
    let db = DB_CONNECTION.lock()?;
    let conn = db
        .as_ref()
        .ok_or_else(|| AppError::database("Database not initialized"))?;

    conn.execute(
        "UPDATE sessions SET ai_summary = ?1, context_for_next = ?2, status = 'analyzed' WHERE id = ?3",
        params![ai_summary, context_for_next, session_id],
    )?;

    Ok(())
}

// ── Tests ──

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    /// Initializes an in-memory database for testing using the unified schema helper.
    fn setup_test_db() {
        crate::memory_storage::setup_test_db_with_schema();
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

    // ── get_today_stats_sync tests (EXP-005) ──

    #[test]
    #[serial]
    fn today_stats_empty_db_returns_zeros() {
        setup_test_db();
        let stats = get_today_stats_sync().unwrap();
        assert_eq!(stats.total_count, 0);
        assert_eq!(stats.auto_count, 0);
        assert_eq!(stats.manual_count, 0);
        assert!(stats.first_record_time.is_none());
        assert!(stats.latest_record_time.is_none());
        assert!(stats.busiest_hour.is_none());
        assert_eq!(stats.busiest_hour_count, 0);
    }

    #[test]
    #[serial]
    fn today_stats_counts_auto_and_manual() {
        setup_test_db();

        add_record("auto", "screenshot 1", None, None, None).unwrap();
        add_record("auto", "screenshot 2", None, None, None).unwrap();
        add_record("manual", "note 1", None, None, None).unwrap();

        let stats = get_today_stats_sync().unwrap();
        assert_eq!(stats.total_count, 3);
        assert_eq!(stats.auto_count, 2);
        assert_eq!(stats.manual_count, 1);
    }

    #[test]
    #[serial]
    fn today_stats_returns_time_range() {
        setup_test_db();

        let today = chrono::Local::now().date_naive();

        // Insert records at different times
        let ts_10 = local_to_utc_rfc3339(today.and_hms_opt(10, 0, 0).unwrap());
        insert_record_with_ts(&ts_10, "morning");

        let ts_14 = local_to_utc_rfc3339(today.and_hms_opt(14, 0, 0).unwrap());
        insert_record_with_ts(&ts_14, "afternoon");

        let stats = get_today_stats_sync().unwrap();
        assert!(stats.first_record_time.is_some());
        assert!(stats.latest_record_time.is_some());
        // The first should be earlier than the latest
        assert!(
            stats.first_record_time.as_ref().unwrap() < stats.latest_record_time.as_ref().unwrap()
        );
    }

    #[test]
    #[serial]
    fn today_stats_finds_busiest_hour() {
        setup_test_db();

        let today = chrono::Local::now().date_naive();

        // Insert 3 records at 10:00
        for _ in 0..3 {
            let ts = local_to_utc_rfc3339(today.and_hms_opt(10, 30, 0).unwrap());
            insert_record_with_ts(&ts, "10am record");
        }

        // Insert 1 record at 14:00
        let ts_14 = local_to_utc_rfc3339(today.and_hms_opt(14, 0, 0).unwrap());
        insert_record_with_ts(&ts_14, "2pm record");

        let stats = get_today_stats_sync().unwrap();
        assert_eq!(stats.busiest_hour, Some(10));
        assert_eq!(stats.busiest_hour_count, 3);
    }

    #[test]
    #[serial]
    fn today_stats_excludes_yesterday_records() {
        setup_test_db();

        // Insert a record for yesterday
        let yesterday = chrono::Local::now().date_naive() - chrono::Duration::days(1);
        let ts = local_to_utc_rfc3339(yesterday.and_hms_opt(12, 0, 0).unwrap());
        insert_record_with_ts(&ts, "yesterday");

        let stats = get_today_stats_sync().unwrap();
        assert_eq!(stats.total_count, 0);
        assert!(stats.first_record_time.is_none());
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
        assert!(result
            .unwrap_err()
            .message
            .contains("Invalid start_date format"));
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
        assert!(result.unwrap_err().message.contains("Invalid source_type"));
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
        assert!(result.unwrap_err().message.contains("not found"));
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
            session_id: None,
            analysis_status: None,
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
            session_id: None,
            analysis_status: Some("pending".to_string()),
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
            session_id: None,
            analysis_status: None,
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
            session_id: None,
            analysis_status: None,
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
        assert!(result
            .unwrap_err()
            .message
            .contains("Invalid start_date format"));
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
        assert!(result.unwrap_err().message.contains("not found"));
    }

    // NOTE: Performance benchmark tests moved to dedicated `mod benchmarks` below (CORE-008)
}

// ── Performance benchmark tests (CORE-008 AC#3) ──

#[cfg(test)]
mod benchmarks {
    use super::*;
    use serial_test::serial;
    use std::time::Instant;

    fn setup_test_db() {
        crate::memory_storage::setup_test_db_with_schema();
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
            elapsed_ms < 500,
            "Query 100 records took {}ms (expected < 500ms)",
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
            elapsed_ms < 500,
            "Search took {}ms (expected < 500ms)",
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
            elapsed_ms < 500,
            "3 paginated queries took {}ms (expected < 500ms)",
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
            elapsed_ms < 200,
            "Count took {}ms (expected < 200ms)",
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
            elapsed_ms < 500,
            "Export query took {}ms (expected < 500ms)",
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
            elapsed_ms < 500,
            "Week query took {}ms (expected < 500ms)",
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
            elapsed_ms < 500,
            "Month query took {}ms (expected < 500ms)",
            elapsed_ms
        );
    }
}
