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
}

pub fn add_record(
    source_type: &str,
    content: &str,
    screenshot_path: Option<&str>,
) -> Result<i64, String> {
    let db = DB_CONNECTION
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;
    let conn = db.as_ref().ok_or("Database not initialized")?;

    let timestamp = chrono::Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO records (timestamp, source_type, content, screenshot_path) VALUES (?1, ?2, ?3, ?4)",
        params![timestamp, source_type, content, screenshot_path],
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
            "SELECT id, timestamp, source_type, content, screenshot_path FROM records 
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
            "SELECT id, timestamp, source_type, content, screenshot_path FROM records
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
            })
        })
        .map_err(|e| format!("Failed to query records: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to collect records: {}", e))?;

    Ok(records)
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
                auto_adjust_silent, silent_adjustment_paused_until
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
            })
        })
        .map_err(|e| format!("Failed to get settings: {}", e))?;

    // Decrypt API key if it's encrypted
    let settings = if let Some(ref api_key) = settings.api_key {
        if !api_key.is_empty() {
            let mut decrypted_settings = settings.clone();
            decrypted_settings.api_key = Some(crypto::decrypt_api_key(api_key).map_err(|e| {
                format!("Failed to decrypt API key: {}", e)
            })?);
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
            Some(crypto::encrypt_api_key(api_key).map_err(|e| {
                format!("Failed to encrypt API key: {}", e)
            })?)
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
            silent_adjustment_paused_until = ?20
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
                screenshot_path TEXT
            )",
            [],
        )
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
                screenshot_path TEXT
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
                silent_adjustment_paused_until TEXT DEFAULT NULL
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

        let id = add_record("manual", "e2e test note", None).unwrap();
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

        let id = add_record("auto", "screenshot analysis", Some("/tmp/shot.png")).unwrap();
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
}
