use once_cell::sync::Lazy;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::command;

use crate::crypto;

pub static DB_CONNECTION: Lazy<Mutex<Option<Connection>>> = Lazy::new(|| Mutex::new(None));

fn get_app_data_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("DailyLogger")
}

fn get_db_path() -> PathBuf {
    get_app_data_dir().join("data").join("local.db")
}

pub fn init_database() -> Result<(), String> {
    let db_dir = get_app_data_dir().join("data");
    std::fs::create_dir_all(&db_dir)
        .map_err(|e| format!("Failed to create data directory: {}", e))?;

    let db_path = get_db_path();
    let conn = Connection::open(&db_path).map_err(|e| format!("Failed to open database: {}", e))?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS records (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            timestamp TEXT NOT NULL,
            source_type TEXT NOT NULL,
            content TEXT NOT NULL,
            screenshot_path TEXT
        )",
        [],
    )
    .map_err(|e| format!("Failed to create records table: {}", e))?;

    // Migrate: add screenshot_path column if not exists (for existing databases)
    let _ = conn.execute("ALTER TABLE records ADD COLUMN screenshot_path TEXT", []);

    conn.execute(
        "CREATE TABLE IF NOT EXISTS settings (
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
            summary_prompt TEXT
        )",
        [],
    )
    .map_err(|e| format!("Failed to create settings table: {}", e))?;

    conn.execute("INSERT OR IGNORE INTO settings (id) VALUES (1)", [])
        .map_err(|e| format!("Failed to initialize settings: {}", e))?;

    // Migrate: add new columns for split model/prompt config
    let _ = conn.execute(
        "ALTER TABLE settings ADD COLUMN summary_model_name TEXT",
        [],
    );
    let _ = conn.execute("ALTER TABLE settings ADD COLUMN analysis_prompt TEXT", []);
    let _ = conn.execute("ALTER TABLE settings ADD COLUMN summary_prompt TEXT", []);
    let _ = conn.execute(
        "ALTER TABLE settings ADD COLUMN change_threshold INTEGER DEFAULT 3",
        [],
    );
    let _ = conn.execute(
        "ALTER TABLE settings ADD COLUMN max_silent_minutes INTEGER DEFAULT 30",
        [],
    );
    // 新增字段：日报标题格式和是否包含手动记录
    let _ = conn.execute(
        "ALTER TABLE settings ADD COLUMN summary_title_format TEXT DEFAULT '工作日报 - {date}'",
        [],
    );
    let _ = conn.execute(
        "ALTER TABLE settings ADD COLUMN include_manual_records INTEGER DEFAULT 1",
        [],
    );
    // SMART-001: 窗口白名单/黑名单配置
    let _ = conn.execute(
        "ALTER TABLE settings ADD COLUMN window_whitelist TEXT DEFAULT '[]'",
        [],
    );
    let _ = conn.execute(
        "ALTER TABLE settings ADD COLUMN window_blacklist TEXT DEFAULT '[]'",
        [],
    );
    let _ = conn.execute(
        "ALTER TABLE settings ADD COLUMN use_whitelist_only INTEGER DEFAULT 0",
        [],
    );
    // SMART-002: 自动调整静默阈值配置
    let _ = conn.execute(
        "ALTER TABLE settings ADD COLUMN auto_adjust_silent INTEGER DEFAULT 1",
        [],
    );
    let _ = conn.execute(
        "ALTER TABLE settings ADD COLUMN silent_adjustment_paused_until TEXT DEFAULT NULL",
        [],
    );
    // SMART-003: 工作时间自动识别配置
    let _ = conn.execute(
        "ALTER TABLE settings ADD COLUMN auto_detect_work_time INTEGER DEFAULT 1",
        [],
    );
    let _ = conn.execute(
        "ALTER TABLE settings ADD COLUMN use_custom_work_time INTEGER DEFAULT 0",
        [],
    );
    let _ = conn.execute(
        "ALTER TABLE settings ADD COLUMN custom_work_time_start TEXT DEFAULT '09:00'",
        [],
    );
    let _ = conn.execute(
        "ALTER TABLE settings ADD COLUMN custom_work_time_end TEXT DEFAULT '18:00'",
        [],
    );
    let _ = conn.execute(
        "ALTER TABLE settings ADD COLUMN learned_work_time TEXT DEFAULT NULL",
        [],
    );
    // SMART-004: 多显示器支持配置
    let _ = conn.execute(
        "ALTER TABLE settings ADD COLUMN capture_mode TEXT DEFAULT 'primary'",
        [],
    );
    let _ = conn.execute(
        "ALTER TABLE settings ADD COLUMN selected_monitor_index INTEGER DEFAULT 0",
        [],
    );
    // SMART-004: records表添加monitor_info字段
    let _ = conn.execute("ALTER TABLE records ADD COLUMN monitor_info TEXT", []);

    // DATA-002: FTS5 全文搜索虚拟表
    // 使用 unicode61 tokenchars 选项支持中文字符
    conn.execute(
        "CREATE VIRTUAL TABLE IF NOT EXISTS records_fts USING fts5(
            content,
            content='records',
            content_rowid='id',
            tokenize='unicode61 tokenchars \"-_\"'
        )",
        [],
    )
    .map_err(|e| format!("Failed to create FTS5 table: {}", e))?;

    // FTS5 triggers for automatic index sync
    conn.execute(
        "CREATE TRIGGER IF NOT EXISTS records_ai AFTER INSERT ON records BEGIN
            INSERT INTO records_fts(rowid, content) VALUES (new.id, new.content);
        END",
        [],
    )
    .map_err(|e| format!("Failed to create FTS5 insert trigger: {}", e))?;

    conn.execute(
        "CREATE TRIGGER IF NOT EXISTS records_ad AFTER DELETE ON records BEGIN
            INSERT INTO records_fts(records_fts, rowid, content)
            VALUES ('delete', old.id, old.content);
        END",
        [],
    )
    .map_err(|e| format!("Failed to create FTS5 delete trigger: {}", e))?;

    conn.execute(
        "CREATE TRIGGER IF NOT EXISTS records_au AFTER UPDATE ON records BEGIN
            INSERT INTO records_fts(records_fts, rowid, content)
            VALUES ('delete', old.id, old.content);
            INSERT INTO records_fts(rowid, content) VALUES (new.id, new.content);
        END",
        [],
    )
    .map_err(|e| format!("Failed to create FTS5 update trigger: {}", e))?;

    let mut db = DB_CONNECTION
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;
    *db = Some(conn);

    // Migrate plain text API key to encrypted storage
    migrate_plain_api_key()?;

    tracing::info!("Database initialized at {:?}", db_path);
    Ok(())
}

/// Migrate plain text API key to encrypted storage
fn migrate_plain_api_key() -> Result<(), String> {
    let db = DB_CONNECTION
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;
    let conn = db.as_ref().ok_or("Database not initialized")?;

    // Query current API key
    let api_key: Option<String> = conn
        .query_row("SELECT api_key FROM settings WHERE id = 1", [], |row| {
            row.get::<_, Option<String>>(0)
        })
        .optional()
        .map_err(|e| format!("Failed to query API key: {}", e))?
        .flatten();

    if let Some(key) = api_key {
        if !key.is_empty() && !crypto::is_encrypted(&key) {
            // Plain text key, encrypt it
            let encrypted = crypto::encrypt_api_key(&key)?;
            conn.execute(
                "UPDATE settings SET api_key = ?1 WHERE id = 1",
                params![encrypted],
            )
            .map_err(|e| format!("Failed to update encrypted API key: {}", e))?;
            tracing::info!("Migrated plain API key to encrypted storage");
        }
    }

    Ok(())
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

#[derive(Debug, Clone, Serialize, Deserialize)]
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
}

pub fn add_record(
    source_type: &str,
    content: &str,
    screenshot_path: Option<&str>,
    monitor_info: Option<&str>,
) -> Result<i64, String> {
    let db = DB_CONNECTION
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;
    let conn = db.as_ref().ok_or("Database not initialized")?;

    let timestamp = chrono::Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO records (timestamp, source_type, content, screenshot_path, monitor_info) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![timestamp, source_type, content, screenshot_path, monitor_info],
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
            "SELECT id, timestamp, source_type, content, screenshot_path, monitor_info FROM records
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
            "SELECT id, timestamp, source_type, content, screenshot_path, monitor_info FROM records
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
        "SELECT id, timestamp, source_type, content, screenshot_path, monitor_info FROM records
         WHERE timestamp >= ?1 AND timestamp <= ?2 AND source_type = ?3
         ORDER BY timestamp DESC LIMIT ?4 OFFSET ?5"
    } else {
        "SELECT id, timestamp, source_type, content, screenshot_path, monitor_info FROM records
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
                id, timestamp, source_type, content, screenshot_path, monitor_info
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
                r.id, r.timestamp, r.source_type, r.content, r.screenshot_path, r.monitor_info,
                highlight(records_fts, 0, '<mark>', '</mark>') as snippet,
                bm25(records_fts) as rank
            FROM records_fts
            JOIN records r ON r.id = records_fts.rowid
            WHERE records_fts MATCH ?1
            ORDER BY r.timestamp DESC
            LIMIT ?2"
        } else {
            "SELECT
                r.id, r.timestamp, r.source_type, r.content, r.screenshot_path, r.monitor_info,
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
                    },
                    snippet: row.get(6)?,
                    rank: row.get(7)?,
                })
            })
            .map_err(|e| format!("Failed to search records: {}", e))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Failed to collect search results: {}", e))?;

        Ok(results)
    }
}

pub fn get_settings_sync() -> Result<Settings, String> {
    let db = DB_CONNECTION
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;
    let conn = db.as_ref().ok_or("Database not initialized")?;

    let mut stmt = conn
        .prepare(
            "SELECT api_base_url, api_key, model_name, screenshot_interval,
                summary_time, obsidian_path, auto_capture_enabled, last_summary_path,
                summary_model_name, analysis_prompt, summary_prompt,
                change_threshold, max_silent_minutes, summary_title_format,
                include_manual_records, window_whitelist, window_blacklist, use_whitelist_only,
                auto_adjust_silent, silent_adjustment_paused_until,
                auto_detect_work_time, use_custom_work_time,
                custom_work_time_start, custom_work_time_end, learned_work_time,
                capture_mode, selected_monitor_index
         FROM settings WHERE id = 1",
        )
        .map_err(|e| format!("Failed to prepare query: {}", e))?;

    let settings = stmt
        .query_row([], |row| {
            Ok(Settings {
                api_base_url: row.get(0)?,
                api_key: row.get(1)?,
                model_name: row.get(2)?,
                screenshot_interval: row.get(3)?,
                summary_time: row.get(4)?,
                obsidian_path: row.get(5)?,
                auto_capture_enabled: row.get::<_, Option<i32>>(6)?.map(|v| v != 0),
                last_summary_path: row.get(7)?,
                summary_model_name: row.get(8)?,
                analysis_prompt: row.get(9)?,
                summary_prompt: row.get(10)?,
                change_threshold: row.get(11)?,
                max_silent_minutes: row.get(12)?,
                summary_title_format: row.get(13)?,
                include_manual_records: row.get::<_, Option<i32>>(14)?.map(|v| v != 0),
                window_whitelist: row.get(15)?,
                window_blacklist: row.get(16)?,
                use_whitelist_only: row.get::<_, Option<i32>>(17)?.map(|v| v != 0),
                auto_adjust_silent: row.get::<_, Option<i32>>(18)?.map(|v| v != 0),
                silent_adjustment_paused_until: row.get(19)?,
                auto_detect_work_time: row.get::<_, Option<i32>>(20)?.map(|v| v != 0),
                use_custom_work_time: row.get::<_, Option<i32>>(21)?.map(|v| v != 0),
                custom_work_time_start: row.get(22)?,
                custom_work_time_end: row.get(23)?,
                learned_work_time: row.get(24)?,
                capture_mode: row.get(25)?,
                selected_monitor_index: row.get(26)?,
            })
        })
        .map_err(|e| format!("Failed to get settings: {}", e))?;

    // Decrypt API key if it's encrypted
    let settings = if let Some(ref api_key) = settings.api_key {
        if !api_key.is_empty() {
            let mut decrypted_settings = settings.clone();
            decrypted_settings.api_key = Some(
                crypto::decrypt_api_key(api_key)
                    .map_err(|e| format!("Failed to decrypt API key: {}", e))?,
            );
            decrypted_settings
        } else {
            settings
        }
    } else {
        settings
    };

    Ok(settings)
}

pub fn save_settings_sync(settings: &Settings) -> Result<(), String> {
    // Encrypt API key before saving
    let encrypted_api_key = if let Some(ref api_key) = settings.api_key {
        if !api_key.is_empty() && !crypto::is_encrypted(api_key) {
            Some(
                crypto::encrypt_api_key(api_key)
                    .map_err(|e| format!("Failed to encrypt API key: {}", e))?,
            )
        } else {
            settings.api_key.clone()
        }
    } else {
        None
    };

    let db = DB_CONNECTION
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;
    let conn = db.as_ref().ok_or("Database not initialized")?;

    conn.execute(
        "UPDATE settings SET
            api_base_url = ?1,
            api_key = ?2,
            model_name = ?3,
            screenshot_interval = ?4,
            summary_time = ?5,
            obsidian_path = ?6,
            auto_capture_enabled = ?7,
            last_summary_path = ?8,
            summary_model_name = ?9,
            analysis_prompt = ?10,
            summary_prompt = ?11,
            change_threshold = ?12,
            max_silent_minutes = ?13,
            summary_title_format = ?14,
            include_manual_records = ?15,
            window_whitelist = ?16,
            window_blacklist = ?17,
            use_whitelist_only = ?18,
            auto_adjust_silent = ?19,
            silent_adjustment_paused_until = ?20,
            auto_detect_work_time = ?21,
            use_custom_work_time = ?22,
            custom_work_time_start = ?23,
            custom_work_time_end = ?24,
            learned_work_time = ?25,
            capture_mode = ?26,
            selected_monitor_index = ?27
         WHERE id = 1",
        params![
            settings.api_base_url,
            encrypted_api_key,
            settings.model_name,
            settings.screenshot_interval,
            settings.summary_time,
            settings.obsidian_path,
            settings.auto_capture_enabled.map(|v| if v { 1 } else { 0 }),
            settings.last_summary_path,
            settings.summary_model_name,
            settings.analysis_prompt,
            settings.summary_prompt,
            settings.change_threshold,
            settings.max_silent_minutes,
            settings.summary_title_format,
            settings
                .include_manual_records
                .map(|v| if v { 1 } else { 0 }),
            settings.window_whitelist,
            settings.window_blacklist,
            settings.use_whitelist_only.map(|v| if v { 1 } else { 0 }),
            settings.auto_adjust_silent.map(|v| if v { 1 } else { 0 }),
            settings.silent_adjustment_paused_until,
            settings
                .auto_detect_work_time
                .map(|v| if v { 1 } else { 0 }),
            settings.use_custom_work_time.map(|v| if v { 1 } else { 0 }),
            settings.custom_work_time_start,
            settings.custom_work_time_end,
            settings.learned_work_time,
            settings.capture_mode,
            settings.selected_monitor_index,
        ],
    )
    .map_err(|e| format!("Failed to save settings: {}", e))?;

    tracing::info!("Settings saved");
    Ok(())
}

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

#[command]
pub async fn get_settings() -> Result<Settings, String> {
    get_settings_sync()
}

#[command]
pub async fn save_settings(settings: Settings) -> Result<(), String> {
    save_settings_sync(&settings)
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

/// Result of testing API connection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionTestResult {
    pub success: bool,
    pub message: String,
    pub latency_ms: Option<u64>,
}

/// Model information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub model_id: String,
    pub context_window: Option<u64>,
    pub error: Option<String>,
}

/// Test API connection by sending a simple request
#[command]
pub async fn test_api_connection(
    api_base_url: String,
    api_key: String,
    model_name: String,
) -> Result<ConnectionTestResult, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let start = std::time::Instant::now();

    // Send a simple "Say 'ok'" request
    let request_body = serde_json::json!({
        "model": model_name,
        "messages": [{"role": "user", "content": "Say 'ok'"}],
        "max_tokens": 5
    });

    let url = if api_base_url.ends_with('/') {
        format!("{}chat/completions", api_base_url)
    } else {
        format!("{}/chat/completions", api_base_url)
    };

    let response = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&request_body)
        .send()
        .await;

    match response {
        Ok(resp) if resp.status().is_success() => Ok(ConnectionTestResult {
            success: true,
            message: "连接成功".to_string(),
            latency_ms: Some(start.elapsed().as_millis() as u64),
        }),
        Ok(resp) => {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            let message = if status.as_u16() == 401 {
                "API Key 无效".to_string()
            } else if status.as_u16() == 404 {
                "API 端点不存在或模型不支持".to_string()
            } else {
                format!("API 错误 ({}): {}", status, body)
            };
            Ok(ConnectionTestResult {
                success: false,
                message,
                latency_ms: Some(start.elapsed().as_millis() as u64),
            })
        }
        Err(e) => {
            let message = if e.is_timeout() {
                "连接超时，请检查网络或 API 地址".to_string()
            } else if e.is_connect() {
                format!("无法连接到服务器: {}", e)
            } else {
                format!("连接失败: {}", e)
            };
            Ok(ConnectionTestResult {
                success: false,
                message,
                latency_ms: None,
            })
        }
    }
}

/// Get model information including context window
#[command]
pub async fn get_model_info(
    api_base_url: String,
    api_key: String,
    model_name: String,
) -> Result<ModelInfo, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    // OpenAI compatible API /models endpoint
    let url = if api_base_url.ends_with('/') {
        format!("{}models/{}", api_base_url, model_name)
    } else {
        format!("{}/models/{}", api_base_url, model_name)
    };

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
mod tests {
    use super::*;
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
                monitor_info TEXT
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
                selected_monitor_index INTEGER DEFAULT 0
            )",
            [],
        )
        .unwrap();
        conn.execute("INSERT OR IGNORE INTO settings (id) VALUES (1)", [])
            .unwrap();
        let mut db = DB_CONNECTION.lock().unwrap();
        *db = Some(conn);
    }

    /// Initializes an in-memory database with settings table for testing settings operations.
    fn setup_test_db_with_settings() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute(
            "CREATE TABLE records (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp TEXT NOT NULL,
                source_type TEXT NOT NULL,
                content TEXT NOT NULL,
                screenshot_path TEXT,
                monitor_info TEXT
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
                selected_monitor_index INTEGER DEFAULT 0
            )",
            [],
        )
        .unwrap();
        conn.execute("INSERT OR IGNORE INTO settings (id) VALUES (1)", [])
            .unwrap();
        let mut db = DB_CONNECTION.lock().unwrap();
        *db = Some(conn);
    }

    /// Helper: insert a record with a specific UTC timestamp string.
    fn insert_record_with_ts(ts: &str, content: &str) {
        let db = DB_CONNECTION.lock().unwrap();
        let conn = db.as_ref().unwrap();
        conn.execute(
            "INSERT INTO records (timestamp, source_type, content) VALUES (?1, ?2, ?3)",
            params![ts, "manual", content],
        )
        .unwrap();
    }

    /// Helper: convert a local NaiveDateTime to UTC RFC3339 string.
    fn local_to_utc_rfc3339(naive: chrono::NaiveDateTime) -> String {
        naive
            .and_local_timezone(chrono::Local)
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

        let id = add_record("manual", "e2e test note", None, None).unwrap();
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

        let id = add_record("auto", "screenshot analysis", Some("/tmp/shot.png"), None).unwrap();
        assert!(id > 0);

        let records = get_today_records_sync().unwrap();
        let rec = records
            .iter()
            .find(|r| r.content == "screenshot analysis")
            .expect("Record with screenshot must be queryable");
        assert_eq!(rec.screenshot_path.as_deref(), Some("/tmp/shot.png"));
        assert_eq!(rec.source_type, "auto");
    }

    #[test]
    #[serial]
    fn records_ordered_by_timestamp_descending() {
        setup_test_db();

        // Insert two records with known order
        let today = chrono::Local::now().date_naive();
        let ts_early = local_to_utc_rfc3339(today.and_hms_opt(9, 0, 0).unwrap());
        let ts_late = local_to_utc_rfc3339(today.and_hms_opt(15, 0, 0).unwrap());

        insert_record_with_ts(&ts_early, "morning");
        insert_record_with_ts(&ts_late, "afternoon");

        let records = get_today_records_sync().unwrap();
        // Find positions of our two records (other tests may have added records
        // to the shared global DB_CONNECTION when running in parallel).
        let pos_afternoon = records.iter().position(|r| r.content == "afternoon");
        let pos_morning = records.iter().position(|r| r.content == "morning");
        assert!(
            pos_afternoon.is_some() && pos_morning.is_some(),
            "Both records must be present"
        );
        assert!(
            pos_afternoon.unwrap() < pos_morning.unwrap(),
            "afternoon (15:00) should appear before morning (09:00) in DESC order"
        );
    }

    // ── Tests for new settings fields: summary_title_format and include_manual_records ──

    #[test]
    #[serial]
    fn get_settings_returns_default_title_format() {
        setup_test_db_with_settings();

        let settings = get_settings_sync().unwrap();
        assert_eq!(
            settings.summary_title_format,
            Some("工作日报 - {date}".to_string()),
            "Default summary_title_format should be '工作日报 - {{date}}'"
        );
    }

    #[test]
    #[serial]
    fn get_settings_returns_default_include_manual_records() {
        setup_test_db_with_settings();

        let settings = get_settings_sync().unwrap();
        assert_eq!(
            settings.include_manual_records,
            Some(true),
            "Default include_manual_records should be true"
        );
    }

    #[test]
    #[serial]
    fn save_settings_persists_title_format() {
        setup_test_db_with_settings();

        let mut settings = get_settings_sync().unwrap();
        settings.summary_title_format = Some("Daily Report - {date}".to_string());
        save_settings_sync(&settings).unwrap();

        let reloaded = get_settings_sync().unwrap();
        assert_eq!(
            reloaded.summary_title_format,
            Some("Daily Report - {date}".to_string()),
            "Saved title format should be persisted"
        );
    }

    #[test]
    #[serial]
    fn save_settings_persists_include_manual_records_false() {
        setup_test_db_with_settings();

        let mut settings = get_settings_sync().unwrap();
        settings.include_manual_records = Some(false);
        save_settings_sync(&settings).unwrap();

        let reloaded = get_settings_sync().unwrap();
        assert_eq!(
            reloaded.include_manual_records,
            Some(false),
            "Saved include_manual_records=false should be persisted"
        );
    }

    #[test]
    #[serial]
    fn save_settings_persists_include_manual_records_true() {
        setup_test_db_with_settings();

        // First set to false, then to true
        let mut settings = get_settings_sync().unwrap();
        settings.include_manual_records = Some(false);
        save_settings_sync(&settings).unwrap();

        let mut settings = get_settings_sync().unwrap();
        settings.include_manual_records = Some(true);
        save_settings_sync(&settings).unwrap();

        let reloaded = get_settings_sync().unwrap();
        assert_eq!(
            reloaded.include_manual_records,
            Some(true),
            "Saved include_manual_records=true should be persisted"
        );
    }

    // ── Tests for window whitelist/blacklist settings (SMART-001 Task 2) ──

    #[test]
    #[serial]
    fn get_settings_returns_default_window_whitelist() {
        setup_test_db_with_settings();

        let settings = get_settings_sync().unwrap();
        assert_eq!(
            settings.window_whitelist,
            Some("[]".to_string()),
            "Default window_whitelist should be empty JSON array '[]'"
        );
    }

    #[test]
    #[serial]
    fn get_settings_returns_default_window_blacklist() {
        setup_test_db_with_settings();

        let settings = get_settings_sync().unwrap();
        assert_eq!(
            settings.window_blacklist,
            Some("[]".to_string()),
            "Default window_blacklist should be empty JSON array '[]'"
        );
    }

    #[test]
    #[serial]
    fn get_settings_returns_default_use_whitelist_only() {
        setup_test_db_with_settings();

        let settings = get_settings_sync().unwrap();
        assert_eq!(
            settings.use_whitelist_only,
            Some(false),
            "Default use_whitelist_only should be false"
        );
    }

    #[test]
    #[serial]
    fn save_settings_persists_window_whitelist() {
        setup_test_db_with_settings();

        let mut settings = get_settings_sync().unwrap();
        settings.window_whitelist = Some(r#"["VS Code", "IntelliJ IDEA"]"#.to_string());
        save_settings_sync(&settings).unwrap();

        let reloaded = get_settings_sync().unwrap();
        assert_eq!(
            reloaded.window_whitelist,
            Some(r#"["VS Code", "IntelliJ IDEA"]"#.to_string()),
            "Saved window_whitelist should be persisted"
        );
    }

    #[test]
    #[serial]
    fn save_settings_persists_window_blacklist() {
        setup_test_db_with_settings();

        let mut settings = get_settings_sync().unwrap();
        settings.window_blacklist = Some(r#"["浏览器", "Slack"]"#.to_string());
        save_settings_sync(&settings).unwrap();

        let reloaded = get_settings_sync().unwrap();
        assert_eq!(
            reloaded.window_blacklist,
            Some(r#"["浏览器", "Slack"]"#.to_string()),
            "Saved window_blacklist should be persisted"
        );
    }

    #[test]
    #[serial]
    fn save_settings_persists_use_whitelist_only_true() {
        setup_test_db_with_settings();

        let mut settings = get_settings_sync().unwrap();
        settings.use_whitelist_only = Some(true);
        save_settings_sync(&settings).unwrap();

        let reloaded = get_settings_sync().unwrap();
        assert_eq!(
            reloaded.use_whitelist_only,
            Some(true),
            "Saved use_whitelist_only=true should be persisted"
        );
    }

    #[test]
    #[serial]
    fn save_settings_persists_use_whitelist_only_false() {
        setup_test_db_with_settings();

        // First set to true, then to false
        let mut settings = get_settings_sync().unwrap();
        settings.use_whitelist_only = Some(true);
        save_settings_sync(&settings).unwrap();

        let mut settings = get_settings_sync().unwrap();
        settings.use_whitelist_only = Some(false);
        save_settings_sync(&settings).unwrap();

        let reloaded = get_settings_sync().unwrap();
        assert_eq!(
            reloaded.use_whitelist_only,
            Some(false),
            "Saved use_whitelist_only=false should be persisted"
        );
    }

    #[test]
    #[serial]
    fn window_whitelist_accepts_complex_json() {
        setup_test_db_with_settings();

        let mut settings = get_settings_sync().unwrap();
        // Test with special characters and unicode
        settings.window_whitelist =
            Some(r#"["VS Code", "微信", "企业微信", "Chrome - 工作"]"#.to_string());
        save_settings_sync(&settings).unwrap();

        let reloaded = get_settings_sync().unwrap();
        assert_eq!(
            reloaded.window_whitelist,
            Some(r#"["VS Code", "微信", "企业微信", "Chrome - 工作"]"#.to_string()),
            "window_whitelist should handle unicode and special characters"
        );
    }

    #[test]
    #[serial]
    fn window_blacklist_accepts_empty_array() {
        setup_test_db_with_settings();

        let mut settings = get_settings_sync().unwrap();
        settings.window_blacklist = Some("[]".to_string());
        save_settings_sync(&settings).unwrap();

        let reloaded = get_settings_sync().unwrap();
        assert_eq!(
            reloaded.window_blacklist,
            Some("[]".to_string()),
            "window_blacklist should accept empty array"
        );
    }

    // ── Tests for get_records_by_date_range ──

    #[test]
    #[serial]
    fn get_records_by_date_range_finds_records_in_range() {
        setup_test_db();

        // Insert records at specific dates
        let day1 = chrono::NaiveDate::from_ymd_opt(2026, 3, 10).unwrap();
        let day2 = chrono::NaiveDate::from_ymd_opt(2026, 3, 12).unwrap();
        let day3 = chrono::NaiveDate::from_ymd_opt(2026, 3, 15).unwrap();

        let ts_day1 = local_to_utc_rfc3339(day1.and_hms_opt(10, 0, 0).unwrap());
        let ts_day2 = local_to_utc_rfc3339(day2.and_hms_opt(10, 0, 0).unwrap());
        let ts_day3 = local_to_utc_rfc3339(day3.and_hms_opt(10, 0, 0).unwrap());

        insert_record_with_ts(&ts_day1, "day 1 record");
        insert_record_with_ts(&ts_day2, "day 2 record");
        insert_record_with_ts(&ts_day3, "day 3 record");

        // Query range: 2026-03-10 to 2026-03-12 (inclusive)
        let start_date = "2026-03-10".to_string();
        let end_date = "2026-03-12".to_string();

        let records = get_records_by_date_range_sync(start_date, end_date).unwrap();

        assert!(
            records.iter().any(|r| r.content == "day 1 record"),
            "Day 1 record should be in range"
        );
        assert!(
            records.iter().any(|r| r.content == "day 2 record"),
            "Day 2 record should be in range"
        );
        assert!(
            !records.iter().any(|r| r.content == "day 3 record"),
            "Day 3 record should NOT be in range"
        );
    }

    #[test]
    #[serial]
    fn get_records_by_date_range_includes_end_date_boundary() {
        setup_test_db();

        // Insert record at end of end_date (23:59:59)
        let end_day = chrono::NaiveDate::from_ymd_opt(2026, 3, 15).unwrap();
        let ts_end = local_to_utc_rfc3339(end_day.and_hms_opt(23, 59, 59).unwrap());
        insert_record_with_ts(&ts_end, "end of end day");

        let records =
            get_records_by_date_range_sync("2026-03-15".to_string(), "2026-03-15".to_string())
                .unwrap();

        assert!(
            records.iter().any(|r| r.content == "end of end day"),
            "Record at 23:59:59 on end date should be included"
        );
    }

    #[test]
    #[serial]
    fn get_records_by_date_range_includes_start_date_boundary() {
        setup_test_db();

        // Insert record at start of start_date (00:00:00)
        let start_day = chrono::NaiveDate::from_ymd_opt(2026, 3, 10).unwrap();
        let ts_start = local_to_utc_rfc3339(start_day.and_hms_opt(0, 0, 0).unwrap());
        insert_record_with_ts(&ts_start, "start of start day");

        let records =
            get_records_by_date_range_sync("2026-03-10".to_string(), "2026-03-10".to_string())
                .unwrap();

        assert!(
            records.iter().any(|r| r.content == "start of start day"),
            "Record at 00:00:00 on start date should be included"
        );
    }

    #[test]
    #[serial]
    fn get_records_by_date_range_excludes_outside_range() {
        setup_test_db();

        let day_before = chrono::NaiveDate::from_ymd_opt(2026, 3, 9).unwrap();
        let day_after = chrono::NaiveDate::from_ymd_opt(2026, 3, 16).unwrap();

        let ts_before = local_to_utc_rfc3339(day_before.and_hms_opt(23, 59, 59).unwrap());
        let ts_after = local_to_utc_rfc3339(day_after.and_hms_opt(0, 0, 0).unwrap());

        insert_record_with_ts(&ts_before, "day before range");
        insert_record_with_ts(&ts_after, "day after range");

        let records =
            get_records_by_date_range_sync("2026-03-10".to_string(), "2026-03-15".to_string())
                .unwrap();

        assert!(
            !records.iter().any(|r| r.content == "day before range"),
            "Record before range should NOT be included"
        );
        assert!(
            !records.iter().any(|r| r.content == "day after range"),
            "Record after range should NOT be included"
        );
    }

    #[test]
    #[serial]
    fn get_records_by_date_range_returns_empty_for_no_matches() {
        setup_test_db();

        // Insert record outside the range
        let day = chrono::NaiveDate::from_ymd_opt(2026, 3, 1).unwrap();
        let ts = local_to_utc_rfc3339(day.and_hms_opt(10, 0, 0).unwrap());
        insert_record_with_ts(&ts, "outside record");

        let records =
            get_records_by_date_range_sync("2026-03-10".to_string(), "2026-03-15".to_string())
                .unwrap();

        // Only check that our specific record is not there (other tests may have records)
        assert!(
            !records.iter().any(|r| r.content == "outside record"),
            "No records should match the range"
        );
    }

    #[test]
    #[serial]
    fn get_records_by_date_range_orders_descending() {
        setup_test_db();

        let day1 = chrono::NaiveDate::from_ymd_opt(2026, 3, 10).unwrap();
        let day2 = chrono::NaiveDate::from_ymd_opt(2026, 3, 11).unwrap();

        let ts_early = local_to_utc_rfc3339(day1.and_hms_opt(9, 0, 0).unwrap());
        let ts_late = local_to_utc_rfc3339(day2.and_hms_opt(15, 0, 0).unwrap());

        insert_record_with_ts(&ts_early, "early record");
        insert_record_with_ts(&ts_late, "late record");

        let records =
            get_records_by_date_range_sync("2026-03-10".to_string(), "2026-03-11".to_string())
                .unwrap();

        let pos_early = records.iter().position(|r| r.content == "early record");
        let pos_late = records.iter().position(|r| r.content == "late record");

        assert!(
            pos_late.unwrap() < pos_early.unwrap(),
            "Records should be in descending timestamp order"
        );
    }

    // ── Tests for get_today_record_count_sync ──

    #[test]
    #[serial]
    fn get_today_record_count_sync_returns_zero_for_empty_db() {
        setup_test_db();

        let count = get_today_record_count_sync().unwrap();
        assert_eq!(count, 0, "Empty database should have 0 records");
    }

    #[test]
    #[serial]
    fn get_today_record_count_sync_counts_today_records() {
        setup_test_db();

        // Insert 3 records for today
        let today = chrono::Local::now().date_naive();
        let ts1 = local_to_utc_rfc3339(today.and_hms_opt(9, 0, 0).unwrap());
        let ts2 = local_to_utc_rfc3339(today.and_hms_opt(12, 0, 0).unwrap());
        let ts3 = local_to_utc_rfc3339(today.and_hms_opt(15, 0, 0).unwrap());

        insert_record_with_ts(&ts1, "record 1");
        insert_record_with_ts(&ts2, "record 2");
        insert_record_with_ts(&ts3, "record 3");

        let count = get_today_record_count_sync().unwrap();
        assert_eq!(count, 3, "Should count 3 records for today");
    }

    #[test]
    #[serial]
    fn get_today_record_count_sync_excludes_yesterday_records() {
        setup_test_db();

        // Insert 2 records for today
        let today = chrono::Local::now().date_naive();
        let ts1 = local_to_utc_rfc3339(today.and_hms_opt(10, 0, 0).unwrap());
        insert_record_with_ts(&ts1, "today record");

        // Insert 2 records for yesterday
        let yesterday = today - chrono::Duration::days(1);
        let ts2 = local_to_utc_rfc3339(yesterday.and_hms_opt(10, 0, 0).unwrap());
        insert_record_with_ts(&ts2, "yesterday record");

        let count = get_today_record_count_sync().unwrap();
        assert_eq!(
            count, 1,
            "Should only count today's records, not yesterday's"
        );
    }

    // ── Tests for auto_adjust_silent and silent_adjustment_paused_until (SMART-002 Task 3) ──

    #[test]
    #[serial]
    fn get_settings_returns_default_auto_adjust_silent() {
        setup_test_db_with_settings();

        let settings = get_settings_sync().unwrap();
        assert_eq!(
            settings.auto_adjust_silent,
            Some(true),
            "Default auto_adjust_silent should be true"
        );
    }

    #[test]
    #[serial]
    fn get_settings_returns_default_silent_adjustment_paused_until() {
        setup_test_db_with_settings();

        let settings = get_settings_sync().unwrap();
        assert_eq!(
            settings.silent_adjustment_paused_until, None,
            "Default silent_adjustment_paused_until should be None"
        );
    }

    #[test]
    #[serial]
    fn save_settings_persists_auto_adjust_silent_false() {
        setup_test_db_with_settings();

        let mut settings = get_settings_sync().unwrap();
        settings.auto_adjust_silent = Some(false);
        save_settings_sync(&settings).unwrap();

        let reloaded = get_settings_sync().unwrap();
        assert_eq!(
            reloaded.auto_adjust_silent,
            Some(false),
            "Saved auto_adjust_silent=false should be persisted"
        );
    }

    #[test]
    #[serial]
    fn save_settings_persists_auto_adjust_silent_true() {
        setup_test_db_with_settings();

        // First set to false, then to true
        let mut settings = get_settings_sync().unwrap();
        settings.auto_adjust_silent = Some(false);
        save_settings_sync(&settings).unwrap();

        let mut settings = get_settings_sync().unwrap();
        settings.auto_adjust_silent = Some(true);
        save_settings_sync(&settings).unwrap();

        let reloaded = get_settings_sync().unwrap();
        assert_eq!(
            reloaded.auto_adjust_silent,
            Some(true),
            "Saved auto_adjust_silent=true should be persisted"
        );
    }

    #[test]
    #[serial]
    fn save_settings_persists_silent_adjustment_paused_until() {
        setup_test_db_with_settings();

        let mut settings = get_settings_sync().unwrap();
        let paused_until = "2026-03-16T18:00:00+08:00".to_string();
        settings.silent_adjustment_paused_until = Some(paused_until.clone());
        save_settings_sync(&settings).unwrap();

        let reloaded = get_settings_sync().unwrap();
        assert_eq!(
            reloaded.silent_adjustment_paused_until,
            Some(paused_until),
            "Saved silent_adjustment_paused_until should be persisted"
        );
    }

    #[test]
    #[serial]
    fn save_settings_clears_silent_adjustment_paused_until() {
        setup_test_db_with_settings();

        // First set a value
        let mut settings = get_settings_sync().unwrap();
        settings.silent_adjustment_paused_until = Some("2026-03-16T18:00:00+08:00".to_string());
        save_settings_sync(&settings).unwrap();

        // Then clear it
        let mut settings = get_settings_sync().unwrap();
        settings.silent_adjustment_paused_until = None;
        save_settings_sync(&settings).unwrap();

        let reloaded = get_settings_sync().unwrap();
        assert_eq!(
            reloaded.silent_adjustment_paused_until, None,
            "Cleared silent_adjustment_paused_until should be None"
        );
    }

    #[test]
    #[serial]
    fn silent_adjustment_paused_until_accepts_rfc3339_format() {
        setup_test_db_with_settings();

        let mut settings = get_settings_sync().unwrap();
        // Test various RFC3339 formats
        let test_cases = vec![
            "2026-03-15T12:00:00Z",
            "2026-03-15T12:00:00+08:00",
            "2026-03-15T12:00:00-05:00",
        ];

        for ts in test_cases {
            settings.silent_adjustment_paused_until = Some(ts.to_string());
            save_settings_sync(&settings).unwrap();

            let reloaded = get_settings_sync().unwrap();
            assert_eq!(
                reloaded.silent_adjustment_paused_until,
                Some(ts.to_string()),
                "silent_adjustment_paused_until should accept RFC3339 format: {}",
                ts
            );
        }
    }

    // ── Tests for delete_record_sync (DATA-001) ──

    #[test]
    #[serial]
    fn delete_record_removes_existing_record() {
        setup_test_db();

        // Insert a record
        let today = chrono::Local::now().date_naive();
        let ts = local_to_utc_rfc3339(today.and_hms_opt(10, 0, 0).unwrap());
        insert_record_with_ts(&ts, "record to delete");
        insert_record_with_ts(&ts, "record to keep");

        // Get the id of the record to delete
        let records = get_today_records_sync().unwrap();
        let record_to_delete = records
            .iter()
            .find(|r| r.content == "record to delete")
            .unwrap();

        // Delete it
        let result = delete_record_sync(record_to_delete.id);
        assert!(
            result.is_ok(),
            "delete_record should succeed for existing record"
        );

        // Verify it's deleted
        let remaining = get_today_records_sync().unwrap();
        assert!(
            !remaining.iter().any(|r| r.content == "record to delete"),
            "Record should be deleted"
        );
        assert!(
            remaining.iter().any(|r| r.content == "record to keep"),
            "Other records should remain"
        );
    }

    #[test]
    #[serial]
    fn delete_record_returns_error_for_nonexistent_id() {
        setup_test_db();

        let result = delete_record_sync(999999);
        assert!(
            result.is_err(),
            "delete_record should return error for nonexistent id"
        );
        assert!(
            result.unwrap_err().contains("not found"),
            "Error message should mention record not found"
        );
    }

    // ── Tests for get_history_records_sync (DATA-001) ──

    #[test]
    #[serial]
    fn get_history_records_filters_by_source_type() {
        setup_test_db();

        let today = chrono::Local::now().date_naive();
        let ts = local_to_utc_rfc3339(today.and_hms_opt(10, 0, 0).unwrap());

        // Insert records with different source types
        {
            let db = DB_CONNECTION.lock().unwrap();
            let conn = db.as_ref().unwrap();
            conn.execute(
                "INSERT INTO records (timestamp, source_type, content) VALUES (?1, ?2, ?3)",
                params![ts, "auto", "auto record"],
            )
            .unwrap();
            conn.execute(
                "INSERT INTO records (timestamp, source_type, content) VALUES (?1, ?2, ?3)",
                params![ts, "manual", "manual record"],
            )
            .unwrap();
        }

        let date_str = today.format("%Y-%m-%d").to_string();

        // Test filter by auto
        let auto_records = get_history_records_sync(
            date_str.clone(),
            date_str.clone(),
            Some("auto".to_string()),
            0,
            50,
        )
        .unwrap();
        assert!(
            auto_records.iter().any(|r| r.content == "auto record"),
            "Should find auto record"
        );
        assert!(
            !auto_records.iter().any(|r| r.content == "manual record"),
            "Should not find manual record when filtering by auto"
        );

        // Test filter by manual
        let manual_records = get_history_records_sync(
            date_str.clone(),
            date_str.clone(),
            Some("manual".to_string()),
            0,
            50,
        )
        .unwrap();
        assert!(
            manual_records.iter().any(|r| r.content == "manual record"),
            "Should find manual record"
        );
        assert!(
            !manual_records.iter().any(|r| r.content == "auto record"),
            "Should not find auto record when filtering by manual"
        );
    }

    #[test]
    #[serial]
    fn get_history_records_returns_all_when_no_filter() {
        setup_test_db();

        let today = chrono::Local::now().date_naive();
        let ts = local_to_utc_rfc3339(today.and_hms_opt(10, 0, 0).unwrap());

        {
            let db = DB_CONNECTION.lock().unwrap();
            let conn = db.as_ref().unwrap();
            conn.execute(
                "INSERT INTO records (timestamp, source_type, content) VALUES (?1, ?2, ?3)",
                params![ts, "auto", "auto record"],
            )
            .unwrap();
            conn.execute(
                "INSERT INTO records (timestamp, source_type, content) VALUES (?1, ?2, ?3)",
                params![ts, "manual", "manual record"],
            )
            .unwrap();
        }

        let date_str = today.format("%Y-%m-%d").to_string();
        let records = get_history_records_sync(date_str.clone(), date_str, None, 0, 50).unwrap();

        assert!(
            records.iter().any(|r| r.content == "auto record"),
            "Should find auto record when no filter"
        );
        assert!(
            records.iter().any(|r| r.content == "manual record"),
            "Should find manual record when no filter"
        );
    }

    #[test]
    #[serial]
    fn get_history_records_paginates_correctly() {
        setup_test_db();

        let today = chrono::Local::now().date_naive();
        let date_str = today.format("%Y-%m-%d").to_string();

        // Insert 5 records
        {
            let db = DB_CONNECTION.lock().unwrap();
            let conn = db.as_ref().unwrap();
            for i in 0..5 {
                let ts = local_to_utc_rfc3339(today.and_hms_opt(10 + i, 0, 0).unwrap());
                conn.execute(
                    "INSERT INTO records (timestamp, source_type, content) VALUES (?1, ?2, ?3)",
                    params![ts, "auto", format!("record {}", i)],
                )
                .unwrap();
            }
        }

        // First page (2 records)
        let page1 =
            get_history_records_sync(date_str.clone(), date_str.clone(), None, 0, 2).unwrap();
        assert_eq!(page1.len(), 2, "First page should have 2 records");

        // Second page (2 records)
        let page2 =
            get_history_records_sync(date_str.clone(), date_str.clone(), None, 1, 2).unwrap();
        assert_eq!(page2.len(), 2, "Second page should have 2 records");

        // Third page (1 record)
        let page3 =
            get_history_records_sync(date_str.clone(), date_str.clone(), None, 2, 2).unwrap();
        assert_eq!(page3.len(), 1, "Third page should have 1 record");

        // Verify no overlap between pages
        let page1_contents: std::collections::HashSet<_> =
            page1.iter().map(|r| r.content.clone()).collect();
        let page2_contents: std::collections::HashSet<_> =
            page2.iter().map(|r| r.content.clone()).collect();
        let page3_contents: std::collections::HashSet<_> =
            page3.iter().map(|r| r.content.clone()).collect();

        assert!(
            page1_contents.is_disjoint(&page2_contents),
            "Pages should not overlap"
        );
        assert!(
            page2_contents.is_disjoint(&page3_contents),
            "Pages should not overlap"
        );
    }

    #[test]
    #[serial]
    fn get_history_records_rejects_invalid_source_type() {
        setup_test_db();

        let today = chrono::Local::now().date_naive();
        let date_str = today.format("%Y-%m-%d").to_string();

        let result = get_history_records_sync(
            date_str.clone(),
            date_str,
            Some("invalid".to_string()),
            0,
            50,
        );

        assert!(result.is_err(), "Should reject invalid source_type");
        assert!(
            result.unwrap_err().contains("Invalid source_type"),
            "Error message should mention invalid source_type"
        );
    }

    // ── Tests for search_records_sync (DATA-002) ──

    /// Helper to setup test DB with FTS5 table for search tests
    fn setup_test_db_with_fts() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute(
            "CREATE TABLE records (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp TEXT NOT NULL,
                source_type TEXT NOT NULL,
                content TEXT NOT NULL,
                screenshot_path TEXT,
                monitor_info TEXT
            )",
            [],
        )
        .unwrap();

        // Create settings table with all columns
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
                selected_monitor_index INTEGER DEFAULT 0
            )",
            [],
        )
        .unwrap();
        conn.execute("INSERT OR IGNORE INTO settings (id) VALUES (1)", [])
            .unwrap();

        // Create FTS5 virtual table
        conn.execute(
            "CREATE VIRTUAL TABLE records_fts USING fts5(
                content,
                content='records',
                content_rowid='id',
                tokenize='unicode61'
            )",
            [],
        )
        .unwrap();

        // Create FTS5 triggers
        conn.execute(
            "CREATE TRIGGER records_ai AFTER INSERT ON records BEGIN
                INSERT INTO records_fts(rowid, content) VALUES (new.id, new.content);
            END",
            [],
        )
        .unwrap();
        conn.execute(
            "CREATE TRIGGER records_ad AFTER DELETE ON records BEGIN
                INSERT INTO records_fts(records_fts, rowid, content)
                VALUES ('delete', old.id, old.content);
            END",
            [],
        )
        .unwrap();
        conn.execute(
            "CREATE TRIGGER records_au AFTER UPDATE ON records BEGIN
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

    #[test]
    #[serial]
    fn search_records_finds_matching_keyword() {
        setup_test_db_with_fts();

        // Insert records with different content
        let today = chrono::Local::now().date_naive();
        let ts = local_to_utc_rfc3339(today.and_hms_opt(10, 0, 0).unwrap());
        insert_record_with_ts(&ts, "Working on Rust project today");
        insert_record_with_ts(&ts, "Meeting with team about design");

        let results = search_records_sync("Rust", "rank", 50).unwrap();
        assert!(
            results.iter().any(|r| r.record.content.contains("Rust")),
            "Should find record with 'Rust' keyword"
        );
        assert!(
            !results.iter().any(|r| r.record.content.contains("design")),
            "Should not return record without 'Rust' keyword"
        );
    }

    #[test]
    #[serial]
    fn search_records_supports_chinese_keyword() {
        // Note: For CJK queries, uses LIKE search as fallback
        setup_test_db_with_fts(); // FTS5 needed for non-CJK "Rust" search

        let today = chrono::Local::now().date_naive();
        let ts = local_to_utc_rfc3339(today.and_hms_opt(10, 0, 0).unwrap());
        insert_record_with_ts(&ts, "今天学习了 Rust 编程语言");
        insert_record_with_ts(&ts, "Tomorrow I will study Python");

        let results = search_records_sync("Rust", "rank", 50).unwrap();
        assert!(
            results.iter().any(|r| r.record.content.contains("Rust")),
            "Should find record with 'Rust' in Chinese content"
        );

        // CJK search uses LIKE fallback which supports substring matching
        let results_cn = search_records_sync("学习", "rank", 50).unwrap();
        assert!(
            results_cn.iter().any(|r| r.record.content.contains("学习")),
            "Should find record with Chinese keyword '学习'"
        );

        // Verify highlight is applied
        assert!(
            results_cn[0].snippet.contains("<mark>学习</mark>"),
            "Snippet should contain highlighted Chinese keyword"
        );
    }

    #[test]
    #[serial]
    fn search_records_returns_empty_for_empty_query() {
        setup_test_db_with_fts();

        let today = chrono::Local::now().date_naive();
        let ts = local_to_utc_rfc3339(today.and_hms_opt(10, 0, 0).unwrap());
        insert_record_with_ts(&ts, "Some content here");

        let results = search_records_sync("", "rank", 50).unwrap();
        assert!(
            results.is_empty(),
            "Empty query should return empty results"
        );

        let results_whitespace = search_records_sync("   ", "rank", 50).unwrap();
        assert!(
            results_whitespace.is_empty(),
            "Whitespace-only query should return empty results"
        );
    }

    #[test]
    #[serial]
    fn search_records_orders_by_relevance() {
        setup_test_db_with_fts();

        let today = chrono::Local::now().date_naive();
        let ts = local_to_utc_rfc3339(today.and_hms_opt(10, 0, 0).unwrap());

        // Insert records with varying relevance to "Rust"
        insert_record_with_ts(&ts, "Rust Rust Rust"); // Higher relevance (more matches)
        insert_record_with_ts(&ts, "Rust programming language"); // Lower relevance

        let results = search_records_sync("Rust", "rank", 50).unwrap();
        assert!(
            results.len() >= 2,
            "Should find at least 2 records with 'Rust'"
        );

        // With bm25, lower rank is better. The record with more "Rust" occurrences
        // should have a lower (better) rank.
        let rust_rust_rust = results
            .iter()
            .find(|r| r.record.content == "Rust Rust Rust");
        let rust_programming = results
            .iter()
            .find(|r| r.record.content == "Rust programming language");

        assert!(
            rust_rust_rust.is_some() && rust_programming.is_some(),
            "Both records should be found"
        );

        // Verify ordering: more relevant (lower rank) should come first
        assert!(
            rust_rust_rust.unwrap().rank < rust_programming.unwrap().rank,
            "Record with more keyword occurrences should have better (lower) rank"
        );
    }

    #[test]
    #[serial]
    fn search_records_orders_by_time() {
        setup_test_db_with_fts();

        let today = chrono::Local::now().date_naive();
        let ts_early = local_to_utc_rfc3339(today.and_hms_opt(9, 0, 0).unwrap());
        let ts_late = local_to_utc_rfc3339(today.and_hms_opt(15, 0, 0).unwrap());

        insert_record_with_ts(&ts_early, "Rust early morning");
        insert_record_with_ts(&ts_late, "Rust afternoon work");

        let results = search_records_sync("Rust", "time", 50).unwrap();
        assert!(
            results.len() >= 2,
            "Should find at least 2 records with 'Rust'"
        );

        // With time ordering, later timestamp should come first
        let pos_early = results
            .iter()
            .position(|r| r.record.content == "Rust early morning");
        let pos_late = results
            .iter()
            .position(|r| r.record.content == "Rust afternoon work");

        assert!(
            pos_late.unwrap() < pos_early.unwrap(),
            "Later record should appear first with time ordering"
        );
    }

    #[test]
    #[serial]
    fn search_records_includes_highlight_snippet() {
        setup_test_db_with_fts();

        let today = chrono::Local::now().date_naive();
        let ts = local_to_utc_rfc3339(today.and_hms_opt(10, 0, 0).unwrap());
        insert_record_with_ts(&ts, "Working on Rust project with Cargo");

        let results = search_records_sync("Rust", "rank", 50).unwrap();
        assert!(!results.is_empty(), "Should find matching record");

        let snippet = &results[0].snippet;
        assert!(
            snippet.contains("<mark>Rust</mark>"),
            "Snippet should contain highlighted keyword: {}",
            snippet
        );
    }

    #[test]
    #[serial]
    fn search_records_respects_limit() {
        setup_test_db_with_fts();

        let today = chrono::Local::now().date_naive();
        let ts = local_to_utc_rfc3339(today.and_hms_opt(10, 0, 0).unwrap());

        // Insert 10 records all with the same keyword
        for i in 0..10 {
            insert_record_with_ts(&ts, &format!("Rust record number {}", i));
        }

        let results = search_records_sync("Rust", "rank", 5).unwrap();
        assert_eq!(
            results.len(),
            5,
            "Should return only 5 results when limit is 5"
        );
    }
}
