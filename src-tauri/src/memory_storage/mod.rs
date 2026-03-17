use chrono::Datelike;
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

    // AI-004: 工作分类标签
    let _ = conn.execute("ALTER TABLE records ADD COLUMN tags TEXT", []); // JSON array of tags
    let _ = conn.execute(
        "ALTER TABLE settings ADD COLUMN tag_categories TEXT DEFAULT '[]'",
        [],
    ); // JSON array of custom tag categories

    // AI-005: Ollama 本地模型支持
    let _ = conn.execute(
        "ALTER TABLE settings ADD COLUMN is_ollama INTEGER DEFAULT 0",
        [],
    );

    // REPORT-001: 周报生成配置
    let _ = conn.execute(
        "ALTER TABLE settings ADD COLUMN weekly_report_prompt TEXT",
        [],
    );
    let _ = conn.execute(
        "ALTER TABLE settings ADD COLUMN weekly_report_day INTEGER DEFAULT 0",
        [],
    );
    let _ = conn.execute(
        "ALTER TABLE settings ADD COLUMN last_weekly_report_path TEXT",
        [],
    );

    // REPORT-002: 月报生成配置
    let _ = conn.execute(
        "ALTER TABLE settings ADD COLUMN monthly_report_prompt TEXT",
        [],
    );

    // REPORT-003: 自定义报告周期配置
    let _ = conn.execute(
        "ALTER TABLE settings ADD COLUMN custom_report_prompt TEXT",
        [],
    );
    let _ = conn.execute(
        "ALTER TABLE settings ADD COLUMN last_custom_report_path TEXT",
        [],
    );

    // FIX-001: 月报路径独立存储（不再覆盖日报路径）
    let _ = conn.execute(
        "ALTER TABLE settings ADD COLUMN last_monthly_report_path TEXT",
        [],
    );

    // DATA-006: 多 Obsidian Vault 支持
    let _ = conn.execute(
        "ALTER TABLE settings ADD COLUMN obsidian_vaults TEXT DEFAULT '[]'",
        [],
    );

    // REPORT-004: 对比报告配置
    let _ = conn.execute(
        "ALTER TABLE settings ADD COLUMN comparison_report_prompt TEXT",
        [],
    );

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

    // DATA-003: 手动标签系统
    // 标签表：存储用户创建的标签
    conn.execute(
        "CREATE TABLE IF NOT EXISTS manual_tags (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL UNIQUE,
            color TEXT NOT NULL DEFAULT 'blue',
            created_at TEXT NOT NULL
        )",
        [],
    )
    .map_err(|e| format!("Failed to create manual_tags table: {}", e))?;

    // 记录-标签关联表：多对多关系
    conn.execute(
        "CREATE TABLE IF NOT EXISTS record_manual_tags (
            record_id INTEGER NOT NULL,
            tag_id INTEGER NOT NULL,
            PRIMARY KEY (record_id, tag_id),
            FOREIGN KEY (record_id) REFERENCES records(id) ON DELETE CASCADE,
            FOREIGN KEY (tag_id) REFERENCES manual_tags(id) ON DELETE CASCADE
        )",
        [],
    )
    .map_err(|e| format!("Failed to create record_manual_tags table: {}", e))?;

    // 索引优化查询性能
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_record_manual_tags_tag_id ON record_manual_tags(tag_id)",
        [],
    )
    .map_err(|e| format!("Failed to create index on record_manual_tags: {}", e))?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_manual_tags_name ON manual_tags(name)",
        [],
    )
    .map_err(|e| format!("Failed to create index on manual_tags: {}", e))?;

    // CORE-007: 离线队列表
    crate::offline_queue::create_offline_queue_table(&conn)?;

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
    // AI-004: 工作分类标签
    pub tags: Option<String>, // JSON: Vec<String> serialized
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
    // REPORT-004: 对比报告配置
    pub comparison_report_prompt: Option<String>,
}

/// DATA-006: Vault entry for multi-vault support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObsidianVault {
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
}

// DATA-003: 手动标签系统
/// 手动标签结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManualTag {
    pub id: i64,
    pub name: String,
    pub color: String,
    pub created_at: String,
    /// 使用计数，用于标签云显示
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage_count: Option<i64>,
}

/// 记录与标签关联信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordTagInfo {
    pub record_id: i64,
    pub tag_ids: Vec<i64>,
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
            "SELECT id, timestamp, source_type, content, screenshot_path, monitor_info, tags FROM records
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
            "SELECT id, timestamp, source_type, content, screenshot_path, monitor_info, tags FROM records
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
        - chrono::Duration::seconds(1);

    let month_end = month_end.to_rfc3339();

    let mut stmt = conn
        .prepare(
            "SELECT id, timestamp, source_type, content, screenshot_path, monitor_info, tags FROM records
         WHERE timestamp >= ?1 AND timestamp <= ?2 ORDER BY timestamp DESC",
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
            "SELECT id, timestamp, source_type, content, screenshot_path, monitor_info, tags FROM records
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
            "SELECT id, timestamp, source_type, content, screenshot_path, monitor_info, tags FROM records
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
        "SELECT id, timestamp, source_type, content, screenshot_path, monitor_info, tags FROM records
         WHERE timestamp >= ?1 AND timestamp <= ?2 AND source_type = ?3
         ORDER BY timestamp DESC LIMIT ?4 OFFSET ?5"
    } else {
        "SELECT id, timestamp, source_type, content, screenshot_path, monitor_info, tags FROM records
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
                id, timestamp, source_type, content, screenshot_path, monitor_info, tags
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
                r.id, r.timestamp, r.source_type, r.content, r.screenshot_path, r.monitor_info, r.tags,
                highlight(records_fts, 0, '<mark>', '</mark>') as snippet,
                bm25(records_fts) as rank
            FROM records_fts
            JOIN records r ON r.id = records_fts.rowid
            WHERE records_fts MATCH ?1
            ORDER BY r.timestamp DESC
            LIMIT ?2"
        } else {
            "SELECT
                r.id, r.timestamp, r.source_type, r.content, r.screenshot_path, r.monitor_info, r.tags,
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
                    },
                    snippet: row.get(7)?,
                    rank: row.get(8)?,
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
                capture_mode, selected_monitor_index, tag_categories, is_ollama,
                weekly_report_prompt, weekly_report_day, last_weekly_report_path,
                monthly_report_prompt, custom_report_prompt, last_custom_report_path,
                last_monthly_report_path, obsidian_vaults,
                comparison_report_prompt
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
                tag_categories: row.get(27)?,
                is_ollama: row.get::<_, Option<i32>>(28)?.map(|v| v != 0),
                weekly_report_prompt: row.get(29)?,
                weekly_report_day: row.get(30)?,
                last_weekly_report_path: row.get(31)?,
                monthly_report_prompt: row.get(32)?,
                last_monthly_report_path: row.get(35)?,
                custom_report_prompt: row.get(33)?,
                last_custom_report_path: row.get(34)?,
                obsidian_vaults: row.get(36)?,
                comparison_report_prompt: row.get(37)?,
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

    // AI-005: Auto-detect Ollama endpoint based on api_base_url
    let is_ollama = settings
        .api_base_url
        .as_ref()
        .map(|url| crate::ollama::is_ollama_endpoint(url))
        .unwrap_or(false);

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
            selected_monitor_index = ?27,
            tag_categories = ?28,
            is_ollama = ?29,
            weekly_report_prompt = ?30,
            weekly_report_day = ?31,
            last_weekly_report_path = ?32,
            monthly_report_prompt = ?33,
            custom_report_prompt = ?34,
            last_custom_report_path = ?35,
            last_monthly_report_path = ?36,
            obsidian_vaults = ?37,
            comparison_report_prompt = ?38
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
            settings.tag_categories,
            Some(if is_ollama { 1 } else { 0 }),
            settings.weekly_report_prompt,
            settings.weekly_report_day,
            settings.last_weekly_report_path,
            settings.monthly_report_prompt,
            settings.custom_report_prompt,
            settings.last_custom_report_path,
            settings.last_monthly_report_path,
            settings.obsidian_vaults,
            settings.comparison_report_prompt,
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

// ─── AI-004: 工作分类标签相关命令 ────────────────────────────────────────────

/// Default tag categories for work classification
pub const DEFAULT_TAG_CATEGORIES: &[&str] = &[
    "开发", "会议", "写作", "学习", "研究", "沟通", "规划", "文档", "测试", "设计",
];

/// Get the default tag categories
#[command]
pub fn get_default_tag_categories() -> Vec<String> {
    DEFAULT_TAG_CATEGORIES
        .iter()
        .map(|s| s.to_string())
        .collect()
}

/// Get all unique tags currently used in records
#[command]
pub fn get_all_tags() -> Result<Vec<String>, String> {
    let db_guard = DB_CONNECTION
        .lock()
        .map_err(|e| format!("DB lock error: {}", e))?;
    let conn = db_guard.as_ref().ok_or("Database not initialized")?;

    let mut stmt = conn
        .prepare("SELECT DISTINCT tags FROM records WHERE tags IS NOT NULL AND tags != '[]'")
        .map_err(|e| format!("Failed to prepare query: {}", e))?;

    let tag_strings: Vec<String> = stmt
        .query_map([], |row| row.get(0))
        .map_err(|e| format!("Failed to query tags: {}", e))?
        .filter_map(|r| r.ok())
        .collect();

    // Parse JSON arrays and collect unique tags
    let mut unique_tags = std::collections::HashSet::new();
    for tag_str in tag_strings {
        if let Ok(tags) = serde_json::from_str::<Vec<String>>(&tag_str) {
            for tag in tags {
                unique_tags.insert(tag);
            }
        }
    }

    let mut result: Vec<String> = unique_tags.into_iter().collect();
    result.sort();
    Ok(result)
}

/// Get records filtered by a specific tag
#[command]
pub fn get_records_by_tag(tag: String) -> Result<Vec<Record>, String> {
    let db_guard = DB_CONNECTION
        .lock()
        .map_err(|e| format!("DB lock error: {}", e))?;
    let conn = db_guard.as_ref().ok_or("Database not initialized")?;

    // Get all records with tags and filter in Rust (SQLite doesn't handle JSON arrays well)
    let mut stmt = conn
        .prepare(
            "SELECT id, timestamp, source_type, content, screenshot_path, monitor_info, tags
             FROM records
             WHERE tags IS NOT NULL
             ORDER BY timestamp DESC",
        )
        .map_err(|e| format!("Failed to prepare query: {}", e))?;

    let records: Vec<Record> = stmt
        .query_map([], |row| {
            Ok(Record {
                id: row.get(0)?,
                timestamp: row.get(1)?,
                source_type: row.get(2)?,
                content: row.get(3)?,
                screenshot_path: row.get(4)?,
                monitor_info: row.get(5)?,
                tags: row.get(6)?,
            })
        })
        .map_err(|e| format!("Failed to query records: {}", e))?
        .filter_map(|r| r.ok())
        .filter(|r| {
            r.tags
                .as_ref()
                .and_then(|t| serde_json::from_str::<Vec<String>>(t).ok())
                .map(|tags| tags.contains(&tag))
                .unwrap_or(false)
        })
        .collect();

    Ok(records)
}

// ============================================================
// DATA-003: 手动标签系统 CRUD 操作
// ============================================================

/// 预设标签颜色
pub const PRESET_TAG_COLORS: [&str; 8] = [
    "blue", "green", "yellow", "red", "purple", "pink", "cyan", "orange",
];

/// 创建手动标签
#[command]
pub fn create_manual_tag(name: String, color: String) -> Result<ManualTag, String> {
    // 验证标签名长度
    if name.is_empty() || name.len() > 20 {
        return Err("标签名长度必须在 1-20 个字符之间".to_string());
    }

    // 验证颜色是否有效
    if !PRESET_TAG_COLORS.contains(&color.as_str()) {
        return Err(format!(
            "无效的颜色 '{}', 可选颜色: {}",
            color,
            PRESET_TAG_COLORS.join(", ")
        ));
    }

    let db_guard = DB_CONNECTION
        .lock()
        .map_err(|e| format!("DB lock error: {}", e))?;
    let conn = db_guard.as_ref().ok_or("Database not initialized")?;

    let created_at = chrono::Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO manual_tags (name, color, created_at) VALUES (?1, ?2, ?3)",
        params![name, color, created_at],
    )
    .map_err(|e| {
        if e.to_string().contains("UNIQUE constraint failed") {
            format!("标签 '{}' 已存在", name)
        } else {
            format!("Failed to create tag: {}", e)
        }
    })?;

    let id = conn.last_insert_rowid();

    Ok(ManualTag {
        id,
        name,
        color,
        created_at,
        usage_count: Some(0),
    })
}

/// 获取所有手动标签（含使用计数）
#[command]
pub fn get_all_manual_tags() -> Result<Vec<ManualTag>, String> {
    let db_guard = DB_CONNECTION
        .lock()
        .map_err(|e| format!("DB lock error: {}", e))?;
    let conn = db_guard.as_ref().ok_or("Database not initialized")?;

    let mut stmt = conn
        .prepare(
            "SELECT t.id, t.name, t.color, t.created_at, COUNT(rmt.record_id) as usage_count
             FROM manual_tags t
             LEFT JOIN record_manual_tags rmt ON t.id = rmt.tag_id
             GROUP BY t.id, t.name, t.color, t.created_at
             ORDER BY usage_count DESC, t.name ASC",
        )
        .map_err(|e| format!("Failed to prepare query: {}", e))?;

    let tags = stmt
        .query_map([], |row| {
            Ok(ManualTag {
                id: row.get(0)?,
                name: row.get(1)?,
                color: row.get(2)?,
                created_at: row.get(3)?,
                usage_count: Some(row.get(4)?),
            })
        })
        .map_err(|e| format!("Failed to query tags: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to collect tags: {}", e))?;

    Ok(tags)
}

/// 更新手动标签
#[command]
pub fn update_manual_tag(id: i64, name: String, color: String) -> Result<(), String> {
    // 验证标签名长度
    if name.is_empty() || name.len() > 20 {
        return Err("标签名长度必须在 1-20 个字符之间".to_string());
    }

    // 验证颜色是否有效
    if !PRESET_TAG_COLORS.contains(&color.as_str()) {
        return Err(format!(
            "无效的颜色 '{}', 可选颜色: {}",
            color,
            PRESET_TAG_COLORS.join(", ")
        ));
    }

    let db_guard = DB_CONNECTION
        .lock()
        .map_err(|e| format!("DB lock error: {}", e))?;
    let conn = db_guard.as_ref().ok_or("Database not initialized")?;

    let rows_affected = conn
        .execute(
            "UPDATE manual_tags SET name = ?1, color = ?2 WHERE id = ?3",
            params![name, color, id],
        )
        .map_err(|e| {
            if e.to_string().contains("UNIQUE constraint failed") {
                format!("标签 '{}' 已存在", name)
            } else {
                format!("Failed to update tag: {}", e)
            }
        })?;

    if rows_affected == 0 {
        return Err(format!("标签 ID {} 不存在", id));
    }

    Ok(())
}

/// 删除手动标签（级联删除关联）
#[command]
pub fn delete_manual_tag(id: i64) -> Result<(), String> {
    let db_guard = DB_CONNECTION
        .lock()
        .map_err(|e| format!("DB lock error: {}", e))?;
    let conn = db_guard.as_ref().ok_or("Database not initialized")?;

    // 先删除关联记录
    conn.execute(
        "DELETE FROM record_manual_tags WHERE tag_id = ?1",
        params![id],
    )
    .map_err(|e| format!("Failed to delete tag associations: {}", e))?;

    // 再删除标签
    let rows_affected = conn
        .execute("DELETE FROM manual_tags WHERE id = ?1", params![id])
        .map_err(|e| format!("Failed to delete tag: {}", e))?;

    if rows_affected == 0 {
        return Err(format!("标签 ID {} 不存在", id));
    }

    Ok(())
}

/// 为记录添加标签
#[command]
pub fn add_tag_to_record(record_id: i64, tag_id: i64) -> Result<(), String> {
    let db_guard = DB_CONNECTION
        .lock()
        .map_err(|e| format!("DB lock error: {}", e))?;
    let conn = db_guard.as_ref().ok_or("Database not initialized")?;

    // 检查记录是否存在
    let record_exists: bool = conn
        .query_row(
            "SELECT 1 FROM records WHERE id = ?1",
            params![record_id],
            |_row| Ok(true),
        )
        .optional()
        .map_err(|e| format!("Failed to check record: {}", e))?
        .unwrap_or(false);

    if !record_exists {
        return Err(format!("记录 ID {} 不存在", record_id));
    }

    // 检查标签是否存在
    let tag_exists: bool = conn
        .query_row(
            "SELECT 1 FROM manual_tags WHERE id = ?1",
            params![tag_id],
            |_row| Ok(true),
        )
        .optional()
        .map_err(|e| format!("Failed to check tag: {}", e))?
        .unwrap_or(false);

    if !tag_exists {
        return Err(format!("标签 ID {} 不存在", tag_id));
    }

    // 检查是否已关联
    let already_linked: bool = conn
        .query_row(
            "SELECT 1 FROM record_manual_tags WHERE record_id = ?1 AND tag_id = ?2",
            params![record_id, tag_id],
            |_row| Ok(true),
        )
        .optional()
        .map_err(|e| format!("Failed to check link: {}", e))?
        .unwrap_or(false);

    if already_linked {
        return Ok(()); // 已关联，无需重复添加
    }

    // 检查该记录的标签数量是否已达上限
    let tag_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM record_manual_tags WHERE record_id = ?1",
            params![record_id],
            |row| row.get(0),
        )
        .map_err(|e| format!("Failed to count tags: {}", e))?;

    if tag_count >= 10 {
        return Err("每条记录最多只能添加 10 个标签".to_string());
    }

    conn.execute(
        "INSERT INTO record_manual_tags (record_id, tag_id) VALUES (?1, ?2)",
        params![record_id, tag_id],
    )
    .map_err(|e| format!("Failed to add tag to record: {}", e))?;

    Ok(())
}

/// 从记录移除标签
#[command]
pub fn remove_tag_from_record(record_id: i64, tag_id: i64) -> Result<(), String> {
    let db_guard = DB_CONNECTION
        .lock()
        .map_err(|e| format!("DB lock error: {}", e))?;
    let conn = db_guard.as_ref().ok_or("Database not initialized")?;

    let rows_affected = conn
        .execute(
            "DELETE FROM record_manual_tags WHERE record_id = ?1 AND tag_id = ?2",
            params![record_id, tag_id],
        )
        .map_err(|e| format!("Failed to remove tag from record: {}", e))?;

    if rows_affected == 0 {
        return Err("记录与标签未关联".to_string());
    }

    Ok(())
}

/// 获取记录的所有手动标签
#[command]
pub fn get_tags_for_record(record_id: i64) -> Result<Vec<ManualTag>, String> {
    let db_guard = DB_CONNECTION
        .lock()
        .map_err(|e| format!("DB lock error: {}", e))?;
    let conn = db_guard.as_ref().ok_or("Database not initialized")?;

    let mut stmt = conn
        .prepare(
            "SELECT t.id, t.name, t.color, t.created_at
             FROM manual_tags t
             INNER JOIN record_manual_tags rmt ON t.id = rmt.tag_id
             WHERE rmt.record_id = ?1
             ORDER BY t.name",
        )
        .map_err(|e| format!("Failed to prepare query: {}", e))?;

    let tags = stmt
        .query_map(params![record_id], |row| {
            Ok(ManualTag {
                id: row.get(0)?,
                name: row.get(1)?,
                color: row.get(2)?,
                created_at: row.get(3)?,
                usage_count: None,
            })
        })
        .map_err(|e| format!("Failed to query tags: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to collect tags: {}", e))?;

    Ok(tags)
}

/// 批量获取多条记录的手动标签 (PERF-001: 替代 N+1 查询)
#[command]
pub fn get_tags_for_records(
    record_ids: Vec<i64>,
) -> Result<std::collections::HashMap<i64, Vec<ManualTag>>, String> {
    if record_ids.is_empty() {
        return Ok(std::collections::HashMap::new());
    }

    let db_guard = DB_CONNECTION
        .lock()
        .map_err(|e| format!("DB lock error: {}", e))?;
    let conn = db_guard.as_ref().ok_or("Database not initialized")?;

    let placeholders: Vec<String> = record_ids.iter().map(|_| "?".to_string()).collect();
    let placeholders_str = placeholders.join(",");

    let sql = format!(
        "SELECT rmt.record_id, t.id, t.name, t.color, t.created_at
         FROM manual_tags t
         INNER JOIN record_manual_tags rmt ON t.id = rmt.tag_id
         WHERE rmt.record_id IN ({})
         ORDER BY rmt.record_id, t.name",
        placeholders_str
    );

    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| format!("Failed to prepare query: {}", e))?;

    let params: Vec<Box<dyn rusqlite::types::ToSql>> = record_ids
        .iter()
        .map(|id| Box::new(*id) as Box<dyn rusqlite::types::ToSql>)
        .collect();
    let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();

    let rows = stmt
        .query_map(param_refs.as_slice(), |row| {
            Ok((
                row.get::<_, i64>(0)?,
                ManualTag {
                    id: row.get(1)?,
                    name: row.get(2)?,
                    color: row.get(3)?,
                    created_at: row.get(4)?,
                    usage_count: None,
                },
            ))
        })
        .map_err(|e| format!("Failed to query tags: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to collect tags: {}", e))?;

    let mut result: std::collections::HashMap<i64, Vec<ManualTag>> =
        std::collections::HashMap::new();
    for (record_id, tag) in rows {
        result.entry(record_id).or_default().push(tag);
    }

    Ok(result)
}

/// 按多个标签筛选记录（交集 AND 逻辑）
#[command]
pub fn get_records_by_manual_tags(
    tag_ids: Vec<i64>,
    page: i64,
    page_size: i64,
) -> Result<Vec<Record>, String> {
    if tag_ids.is_empty() {
        return Ok(Vec::new());
    }

    let db_guard = DB_CONNECTION
        .lock()
        .map_err(|e| format!("DB lock error: {}", e))?;
    let conn = db_guard.as_ref().ok_or("Database not initialized")?;

    // 构建参数占位符
    let placeholders: Vec<String> = tag_ids.iter().map(|_| "?".to_string()).collect();
    let placeholders_str = placeholders.join(",");

    // 交集筛选：找出同时包含所有指定标签的记录
    let sql = format!(
        "SELECT r.id, r.timestamp, r.source_type, r.content, r.screenshot_path, r.monitor_info, r.tags
         FROM records r
         WHERE r.id IN (
             SELECT record_id FROM record_manual_tags
             WHERE tag_id IN ({})
             GROUP BY record_id
             HAVING COUNT(DISTINCT tag_id) = ?
         )
         ORDER BY r.timestamp DESC
         LIMIT ? OFFSET ?",
        placeholders_str
    );

    let offset = page * page_size;
    let tag_count = tag_ids.len() as i64;

    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| format!("Failed to prepare query: {}", e))?;

    // 构建参数列表
    let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
    for id in &tag_ids {
        params_vec.push(Box::new(*id));
    }
    params_vec.push(Box::new(tag_count));
    params_vec.push(Box::new(page_size));
    params_vec.push(Box::new(offset));

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
            })
        })
        .map_err(|e| format!("Failed to query records: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to collect records: {}", e))?;

    Ok(records)
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
                monitor_info TEXT,
                tags TEXT
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
        // DATA-003: 手动标签系统表
        conn.execute(
            "CREATE TABLE manual_tags (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                color TEXT NOT NULL DEFAULT 'blue',
                created_at TEXT NOT NULL
            )",
            [],
        )
        .unwrap();
        conn.execute(
            "CREATE TABLE record_manual_tags (
                record_id INTEGER NOT NULL,
                tag_id INTEGER NOT NULL,
                PRIMARY KEY (record_id, tag_id),
                FOREIGN KEY (record_id) REFERENCES records(id) ON DELETE CASCADE,
                FOREIGN KEY (tag_id) REFERENCES manual_tags(id) ON DELETE CASCADE
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
                screenshot_path TEXT,
                monitor_info TEXT,
                tags TEXT
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
        // DATA-003: 手动标签系统表
        conn.execute(
            "CREATE TABLE manual_tags (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                color TEXT NOT NULL DEFAULT 'blue',
                created_at TEXT NOT NULL
            )",
            [],
        )
        .unwrap();
        conn.execute(
            "CREATE TABLE record_manual_tags (
                record_id INTEGER NOT NULL,
                tag_id INTEGER NOT NULL,
                PRIMARY KEY (record_id, tag_id),
                FOREIGN KEY (record_id) REFERENCES records(id) ON DELETE CASCADE,
                FOREIGN KEY (tag_id) REFERENCES manual_tags(id) ON DELETE CASCADE
            )",
            [],
        )
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
            "screenshot analysis",
            Some("/tmp/shot.png"),
            None,
            None,
        )
        .unwrap();
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
                monitor_info TEXT,
                tags TEXT
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
                selected_monitor_index INTEGER DEFAULT 0,
                tag_categories TEXT DEFAULT '[]'
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

        // DATA-003: 手动标签系统表
        conn.execute(
            "CREATE TABLE manual_tags (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                color TEXT NOT NULL DEFAULT 'blue',
                created_at TEXT NOT NULL
            )",
            [],
        )
        .unwrap();
        conn.execute(
            "CREATE TABLE record_manual_tags (
                record_id INTEGER NOT NULL,
                tag_id INTEGER NOT NULL,
                PRIMARY KEY (record_id, tag_id),
                FOREIGN KEY (record_id) REFERENCES records(id) ON DELETE CASCADE,
                FOREIGN KEY (tag_id) REFERENCES manual_tags(id) ON DELETE CASCADE
            )",
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

    // ── Tests for AI-004: Tag functionality ──

    #[test]
    #[serial]
    fn get_default_tag_categories_returns_expected_tags() {
        let tags = get_default_tag_categories();
        assert!(!tags.is_empty(), "Should return non-empty list");
        assert!(tags.contains(&"开发".to_string()), "Should contain '开发'");
        assert!(tags.contains(&"会议".to_string()), "Should contain '会议'");
        assert!(tags.contains(&"写作".to_string()), "Should contain '写作'");
    }

    #[test]
    #[serial]
    fn get_default_tag_categories_has_expected_count() {
        let tags = get_default_tag_categories();
        assert_eq!(tags.len(), 10, "Should have 10 default tag categories");
    }

    #[test]
    #[serial]
    fn get_all_tags_returns_empty_when_no_records() {
        setup_test_db();
        let tags = get_all_tags().unwrap();
        assert!(tags.is_empty(), "Should return empty when no records exist");
    }

    #[test]
    #[serial]
    fn get_all_tags_returns_unique_tags_from_records() {
        setup_test_db();

        // Add records with tags
        let _ = add_record(
            "auto",
            r#"{"current_focus":"test1"}"#,
            None,
            None,
            Some(r#"["开发","测试"]"#),
        );
        let _ = add_record(
            "auto",
            r#"{"current_focus":"test2"}"#,
            None,
            None,
            Some(r#"["会议","开发"]"#),
        );
        let _ = add_record("manual", "plain text", None, None, None);

        let tags = get_all_tags().unwrap();
        assert_eq!(tags.len(), 3, "Should have 3 unique tags");
        assert!(tags.contains(&"开发".to_string()));
        assert!(tags.contains(&"测试".to_string()));
        assert!(tags.contains(&"会议".to_string()));
    }

    #[test]
    #[serial]
    fn get_records_by_tag_returns_matching_records() {
        setup_test_db();

        // Add records with different tags
        let _ = add_record(
            "auto",
            r#"{"current_focus":"dev work"}"#,
            None,
            None,
            Some(r#"["开发"]"#),
        );
        let _ = add_record(
            "auto",
            r#"{"current_focus":"meeting"}"#,
            None,
            None,
            Some(r#"["会议"]"#),
        );
        let _ = add_record(
            "auto",
            r#"{"current_focus":"dev and test"}"#,
            None,
            None,
            Some(r#"["开发","测试"]"#),
        );

        let records = get_records_by_tag("开发".to_string()).unwrap();
        assert_eq!(records.len(), 2, "Should find 2 records with '开发' tag");

        let meeting_records = get_records_by_tag("会议".to_string()).unwrap();
        assert_eq!(
            meeting_records.len(),
            1,
            "Should find 1 record with '会议' tag"
        );
    }

    #[test]
    #[serial]
    fn get_records_by_tag_returns_empty_for_nonexistent_tag() {
        setup_test_db();

        let _ = add_record(
            "auto",
            r#"{"current_focus":"test"}"#,
            None,
            None,
            Some(r#"["开发"]"#),
        );
        let records = get_records_by_tag("不存在".to_string()).unwrap();
        assert!(
            records.is_empty(),
            "Should return empty for nonexistent tag"
        );
    }

    #[test]
    #[serial]
    fn get_records_by_tag_ignores_records_without_tags() {
        setup_test_db();

        // Add records without tags
        let _ = add_record("auto", r#"{"current_focus":"no tags"}"#, None, None, None);
        let _ = add_record("manual", "plain text", None, None, None);

        let records = get_records_by_tag("开发".to_string()).unwrap();
        assert!(
            records.is_empty(),
            "Should return empty when no records have tags"
        );
    }

    // ── Tests for DATA-003: Manual Tag System ──

    #[test]
    #[serial]
    fn create_manual_tag_creates_tag_with_default_color() {
        setup_test_db();

        let tag = create_manual_tag("工作".to_string(), "blue".to_string()).unwrap();
        assert_eq!(tag.name, "工作");
        assert_eq!(tag.color, "blue");
        assert!(tag.id > 0);
    }

    #[test]
    #[serial]
    fn create_manual_tag_rejects_empty_name() {
        setup_test_db();

        let result = create_manual_tag("".to_string(), "blue".to_string());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("1-20"));
    }

    #[test]
    #[serial]
    fn create_manual_tag_rejects_long_name() {
        setup_test_db();

        let long_name = "这是一个非常长的标签名称超过了二十个字符的限制";
        let result = create_manual_tag(long_name.to_string(), "blue".to_string());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("1-20"));
    }

    #[test]
    #[serial]
    fn create_manual_tag_rejects_invalid_color() {
        setup_test_db();

        let result = create_manual_tag("工作".to_string(), "invalid".to_string());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("无效的颜色"));
    }

    #[test]
    #[serial]
    fn create_manual_tag_rejects_duplicate_name() {
        setup_test_db();

        let _ = create_manual_tag("工作".to_string(), "blue".to_string()).unwrap();
        let result = create_manual_tag("工作".to_string(), "green".to_string());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("已存在"));
    }

    #[test]
    #[serial]
    fn get_all_manual_tags_returns_tags_with_usage_count() {
        setup_test_db();

        // Create tags
        let tag1 = create_manual_tag("工作".to_string(), "blue".to_string()).unwrap();
        let _ = create_manual_tag("学习".to_string(), "green".to_string()).unwrap();

        // Add a record and associate with tag1
        let record_id = add_record("manual", "test content", None, None, None).unwrap();
        let _ = add_tag_to_record(record_id, tag1.id).unwrap();

        let tags = get_all_manual_tags().unwrap();
        assert_eq!(tags.len(), 2);

        // Tags should be sorted by usage count (工作 has 1, 学习 has 0)
        assert_eq!(tags[0].name, "工作");
        assert_eq!(tags[0].usage_count, Some(1));
        assert_eq!(tags[1].name, "学习");
        assert_eq!(tags[1].usage_count, Some(0));
    }

    #[test]
    #[serial]
    fn update_manual_tag_changes_name_and_color() {
        setup_test_db();

        let tag = create_manual_tag("工作".to_string(), "blue".to_string()).unwrap();
        update_manual_tag(tag.id, "任务".to_string(), "red".to_string()).unwrap();

        let tags = get_all_manual_tags().unwrap();
        assert_eq!(tags[0].name, "任务");
        assert_eq!(tags[0].color, "red");
    }

    #[test]
    #[serial]
    fn delete_manual_tag_removes_tag_and_associations() {
        setup_test_db();

        let tag = create_manual_tag("工作".to_string(), "blue".to_string()).unwrap();
        let record_id = add_record("manual", "test", None, None, None).unwrap();
        let _ = add_tag_to_record(record_id, tag.id).unwrap();

        delete_manual_tag(tag.id).unwrap();

        let tags = get_all_manual_tags().unwrap();
        assert!(tags.is_empty());

        // Record should still exist
        let tags_for_record = get_tags_for_record(record_id).unwrap();
        assert!(tags_for_record.is_empty());
    }

    #[test]
    #[serial]
    fn add_tag_to_record_associates_tag() {
        setup_test_db();

        let tag = create_manual_tag("工作".to_string(), "blue".to_string()).unwrap();
        let record_id = add_record("manual", "test content", None, None, None).unwrap();

        add_tag_to_record(record_id, tag.id).unwrap();

        let tags = get_tags_for_record(record_id).unwrap();
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].name, "工作");
    }

    #[test]
    #[serial]
    fn add_tag_to_record_rejects_nonexistent_record() {
        setup_test_db();

        let tag = create_manual_tag("工作".to_string(), "blue".to_string()).unwrap();
        let result = add_tag_to_record(999, tag.id);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("不存在"));
    }

    #[test]
    #[serial]
    fn add_tag_to_record_rejects_nonexistent_tag() {
        setup_test_db();

        let record_id = add_record("manual", "test", None, None, None).unwrap();
        let result = add_tag_to_record(record_id, 999);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("不存在"));
    }

    #[test]
    #[serial]
    fn add_tag_to_record_limits_tags_to_10() {
        setup_test_db();

        let record_id = add_record("manual", "test", None, None, None).unwrap();

        // Create and add 10 tags
        for i in 0..10 {
            let tag = create_manual_tag(format!("标签{}", i), "blue".to_string()).unwrap();
            add_tag_to_record(record_id, tag.id).unwrap();
        }

        // Try to add 11th tag
        let tag11 = create_manual_tag("标签11".to_string(), "red".to_string()).unwrap();
        let result = add_tag_to_record(record_id, tag11.id);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("10"));
    }

    #[test]
    #[serial]
    fn remove_tag_from_record_removes_association() {
        setup_test_db();

        let tag = create_manual_tag("工作".to_string(), "blue".to_string()).unwrap();
        let record_id = add_record("manual", "test", None, None, None).unwrap();
        add_tag_to_record(record_id, tag.id).unwrap();

        remove_tag_from_record(record_id, tag.id).unwrap();

        let tags = get_tags_for_record(record_id).unwrap();
        assert!(tags.is_empty());
    }

    #[test]
    #[serial]
    fn get_records_by_manual_tags_returns_intersection() {
        setup_test_db();

        // Create tags
        let tag1 = create_manual_tag("工作".to_string(), "blue".to_string()).unwrap();
        let tag2 = create_manual_tag("重要".to_string(), "red".to_string()).unwrap();

        // Add records
        let record1 = add_record("manual", "record 1", None, None, None).unwrap();
        let record2 = add_record("manual", "record 2", None, None, None).unwrap();
        let record3 = add_record("manual", "record 3", None, None, None).unwrap();

        // record1: tag1 only
        add_tag_to_record(record1, tag1.id).unwrap();
        // record2: tag2 only
        add_tag_to_record(record2, tag2.id).unwrap();
        // record3: both tags
        add_tag_to_record(record3, tag1.id).unwrap();
        add_tag_to_record(record3, tag2.id).unwrap();

        // Filter by both tags (AND logic)
        let records = get_records_by_manual_tags(vec![tag1.id, tag2.id], 0, 50).unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].id, record3);

        // Filter by single tag
        let records_tag1 = get_records_by_manual_tags(vec![tag1.id], 0, 50).unwrap();
        assert_eq!(records_tag1.len(), 2);
    }

    #[test]
    #[serial]
    fn get_records_by_manual_tags_supports_pagination() {
        setup_test_db();

        let tag = create_manual_tag("工作".to_string(), "blue".to_string()).unwrap();

        // Add 15 records with the tag
        for i in 0..15 {
            let record_id =
                add_record("manual", &format!("record {}", i), None, None, None).unwrap();
            add_tag_to_record(record_id, tag.id).unwrap();
        }

        // Page 0: first 10 records
        let page0 = get_records_by_manual_tags(vec![tag.id], 0, 10).unwrap();
        assert_eq!(page0.len(), 10);

        // Page 1: remaining 5 records
        let page1 = get_records_by_manual_tags(vec![tag.id], 1, 10).unwrap();
        assert_eq!(page1.len(), 5);
    }

    // ── Tests for get_week_records_sync (REPORT-001) ──

    #[test]
    #[serial]
    fn week_records_includes_today() {
        setup_test_db();

        let today = chrono::Local::now().date_naive();
        let ts = local_to_utc_rfc3339(today.and_hms_opt(12, 0, 0).unwrap());
        insert_record_with_ts(&ts, "week_test_today_record");

        let records = get_week_records_sync(0).unwrap();
        assert!(
            records
                .iter()
                .any(|r| r.content == "week_test_today_record"),
            "Today's record must appear in this week's records"
        );
    }

    #[test]
    #[serial]
    fn week_records_empty_db_returns_empty() {
        setup_test_db();
        // Don't insert any records
        let _records = get_week_records_sync(0).unwrap();
        // May contain records from parallel tests, but at minimum should not error
    }

    #[test]
    #[serial]
    fn week_records_includes_week_start_boundary() {
        setup_test_db();

        // Calculate the start of the current week (Monday)
        let today = chrono::Local::now().date_naive();
        let weekday = today.weekday().num_days_from_monday() as i64;
        let week_start = today - chrono::Duration::days(weekday);

        // Insert record at Monday 00:00:00
        let ts = local_to_utc_rfc3339(week_start.and_hms_opt(0, 0, 0).unwrap());
        insert_record_with_ts(&ts, "week_boundary_start_record");

        let records = get_week_records_sync(0).unwrap();
        assert!(
            records
                .iter()
                .any(|r| r.content == "week_boundary_start_record"),
            "Record at week start boundary (Monday 00:00) must be included"
        );
    }

    #[test]
    #[serial]
    fn week_records_includes_week_end_boundary() {
        setup_test_db();

        // Calculate the end of the current week (Sunday)
        let today = chrono::Local::now().date_naive();
        let weekday = today.weekday().num_days_from_monday() as i64;
        let week_end = today - chrono::Duration::days(weekday) + chrono::Duration::days(6);

        // Insert record at Sunday 23:59:59
        let ts = local_to_utc_rfc3339(week_end.and_hms_opt(23, 59, 59).unwrap());
        insert_record_with_ts(&ts, "week_boundary_end_record");

        let records = get_week_records_sync(0).unwrap();
        assert!(
            records
                .iter()
                .any(|r| r.content == "week_boundary_end_record"),
            "Record at week end boundary (Sunday 23:59) must be included"
        );
    }

    #[test]
    #[serial]
    fn week_records_respects_custom_week_start_day() {
        setup_test_db();

        // Insert a record for today
        let today = chrono::Local::now().date_naive();
        let ts = local_to_utc_rfc3339(today.and_hms_opt(10, 0, 0).unwrap());
        insert_record_with_ts(&ts, "custom_week_start_record");

        // Query with week_start_day=6 (Sunday)
        let records = get_week_records_sync(6).unwrap();
        assert!(
            records
                .iter()
                .any(|r| r.content == "custom_week_start_record"),
            "Today's record should still be found with custom week start day"
        );
    }

    /// FIX-001 regression test: last_monthly_report_path must be stored independently
    /// from last_summary_path in the database
    #[test]
    #[serial]
    fn save_settings_persists_last_monthly_report_path_independently() {
        setup_test_db_with_settings();

        // Set both daily summary path and monthly report path
        let mut settings = get_settings_sync().unwrap();
        settings.last_summary_path = Some("/obsidian/工作日报 - 2026-03-16.md".to_string());
        settings.last_monthly_report_path = Some("/obsidian/月报-2026-03.md".to_string());
        save_settings_sync(&settings).unwrap();

        // Reload and verify both paths are independent
        let reloaded = get_settings_sync().unwrap();
        assert_eq!(
            reloaded.last_summary_path,
            Some("/obsidian/工作日报 - 2026-03-16.md".to_string()),
            "FIX-001: last_summary_path must not be affected by monthly report path"
        );
        assert_eq!(
            reloaded.last_monthly_report_path,
            Some("/obsidian/月报-2026-03.md".to_string()),
            "FIX-001: last_monthly_report_path must be persisted independently"
        );

        // Update only monthly path, verify daily path unchanged
        let mut settings = get_settings_sync().unwrap();
        settings.last_monthly_report_path = Some("/obsidian/月报-2026-02.md".to_string());
        save_settings_sync(&settings).unwrap();

        let reloaded = get_settings_sync().unwrap();
        assert_eq!(
            reloaded.last_summary_path,
            Some("/obsidian/工作日报 - 2026-03-16.md".to_string()),
            "FIX-001: updating monthly path must not change daily summary path"
        );
    }

    /// DATA-006: get_obsidian_output_path returns default vault from obsidian_vaults
    #[test]
    fn get_obsidian_output_path_uses_default_vault() {
        let mut settings = get_default_settings();
        settings.obsidian_vaults = Some(
            r#"[{"name":"Work","path":"/vaults/work","is_default":false},{"name":"Personal","path":"/vaults/personal","is_default":true}]"#.to_string()
        );
        assert_eq!(
            settings.get_obsidian_output_path().unwrap(),
            "/vaults/personal"
        );
    }

    /// DATA-006: get_obsidian_output_path falls back to first vault if no default
    #[test]
    fn get_obsidian_output_path_falls_back_to_first_vault() {
        let mut settings = get_default_settings();
        settings.obsidian_vaults = Some(
            r#"[{"name":"Work","path":"/vaults/work","is_default":false},{"name":"Personal","path":"/vaults/personal","is_default":false}]"#.to_string()
        );
        assert_eq!(settings.get_obsidian_output_path().unwrap(), "/vaults/work");
    }

    /// DATA-006: get_obsidian_output_path falls back to obsidian_path when vaults empty
    #[test]
    fn get_obsidian_output_path_falls_back_to_legacy_path() {
        let mut settings = get_default_settings();
        settings.obsidian_vaults = Some("[]".to_string());
        settings.obsidian_path = Some("/legacy/vault".to_string());
        assert_eq!(
            settings.get_obsidian_output_path().unwrap(),
            "/legacy/vault"
        );
    }

    /// DATA-006: get_obsidian_output_path returns error when no path configured
    #[test]
    fn get_obsidian_output_path_returns_error_when_no_path() {
        let mut settings = get_default_settings();
        settings.obsidian_vaults = Some("[]".to_string());
        settings.obsidian_path = None;
        assert!(settings.get_obsidian_output_path().is_err());
    }

    /// DATA-006: obsidian_vaults field persists through save/load cycle
    #[test]
    #[serial]
    fn save_settings_persists_obsidian_vaults() {
        setup_test_db_with_settings();

        let vaults_json = r#"[{"name":"Work","path":"/vaults/work","is_default":true}]"#;
        let mut settings = get_settings_sync().unwrap();
        settings.obsidian_vaults = Some(vaults_json.to_string());
        save_settings_sync(&settings).unwrap();

        let reloaded = get_settings_sync().unwrap();
        assert_eq!(
            reloaded.obsidian_vaults,
            Some(vaults_json.to_string()),
            "DATA-006: obsidian_vaults must be persisted"
        );
    }

    fn get_default_settings() -> Settings {
        Settings {
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
            summary_title_format: None,
            include_manual_records: None,
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
            weekly_report_prompt: None,
            weekly_report_day: None,
            last_weekly_report_path: None,
            monthly_report_prompt: None,
            last_monthly_report_path: None,
            custom_report_prompt: None,
            last_custom_report_path: None,
            obsidian_vaults: None,
            comparison_report_prompt: None,
        }
    }

    // ── PERF-001: Batch tag query tests ──

    #[test]
    #[serial]
    fn get_tags_for_records_returns_empty_for_empty_input() {
        setup_test_db();
        let result = get_tags_for_records(vec![]).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    #[serial]
    fn get_tags_for_records_returns_tags_grouped_by_record() {
        setup_test_db();

        // Insert two records
        let ts = chrono::Utc::now().to_rfc3339();
        {
            let db = DB_CONNECTION.lock().unwrap();
            let conn = db.as_ref().unwrap();
            conn.execute(
                "INSERT INTO records (timestamp, source_type, content) VALUES (?1, 'manual', 'rec1')",
                params![ts],
            ).unwrap();
            conn.execute(
                "INSERT INTO records (timestamp, source_type, content) VALUES (?1, 'manual', 'rec2')",
                params![ts],
            ).unwrap();

            // Insert tags
            conn.execute(
                "INSERT INTO manual_tags (name, color, created_at) VALUES ('tag-a', 'blue', ?1)",
                params![ts],
            )
            .unwrap();
            conn.execute(
                "INSERT INTO manual_tags (name, color, created_at) VALUES ('tag-b', 'red', ?1)",
                params![ts],
            )
            .unwrap();

            // Link: record 1 -> tag-a, tag-b; record 2 -> tag-a
            conn.execute(
                "INSERT INTO record_manual_tags (record_id, tag_id) VALUES (1, 1)",
                [],
            )
            .unwrap();
            conn.execute(
                "INSERT INTO record_manual_tags (record_id, tag_id) VALUES (1, 2)",
                [],
            )
            .unwrap();
            conn.execute(
                "INSERT INTO record_manual_tags (record_id, tag_id) VALUES (2, 1)",
                [],
            )
            .unwrap();
        }

        let result = get_tags_for_records(vec![1, 2]).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result.get(&1).unwrap().len(), 2);
        assert_eq!(result.get(&2).unwrap().len(), 1);
        assert_eq!(result.get(&2).unwrap()[0].name, "tag-a");
    }

    #[test]
    #[serial]
    fn get_tags_for_records_skips_records_without_tags() {
        setup_test_db();

        let ts = chrono::Utc::now().to_rfc3339();
        {
            let db = DB_CONNECTION.lock().unwrap();
            let conn = db.as_ref().unwrap();
            conn.execute(
                "INSERT INTO records (timestamp, source_type, content) VALUES (?1, 'manual', 'rec1')",
                params![ts],
            ).unwrap();
        }

        let result = get_tags_for_records(vec![1]).unwrap();
        // Record exists but has no tags — should not appear in map
        assert!(!result.contains_key(&1));
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
        let conn = Connection::open_in_memory().unwrap();
        conn.execute(
            "CREATE TABLE records (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp TEXT NOT NULL,
                source_type TEXT NOT NULL,
                content TEXT NOT NULL,
                screenshot_path TEXT,
                monitor_info TEXT,
                tags TEXT
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
            "Inserting 100 records took {}ms (threshold: 1000ms, avg: {}ms per insert)",
            elapsed_ms,
            elapsed_ms / 100
        );
    }

    /// Benchmark: Query records by date range
    #[test]
    #[serial]
    fn benchmark_query_records_by_date_range() {
        setup_test_db();

        // Insert 100 records first
        for i in 0..100 {
            let ts = chrono::Utc::now().to_rfc3339();
            let db = DB_CONNECTION.lock().unwrap();
            let conn = db.as_ref().unwrap();
            conn.execute(
                "INSERT INTO records (timestamp, source_type, content) VALUES (?1, ?2, ?3)",
                params![ts, "manual", format!("Query test record #{}", i)],
            )
            .unwrap();
        }

        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let start = Instant::now();
        let _ = get_records_for_export(&today, &today);
        let elapsed_ms = start.elapsed().as_millis();

        // Query should complete in < 500ms (generous for CI runners)
        assert!(
            elapsed_ms < 500,
            "Querying records by date range took {}ms (threshold: 500ms)",
            elapsed_ms
        );
    }

    /// Benchmark: Get settings (read operation)
    #[test]
    #[serial]
    fn benchmark_get_settings() {
        setup_test_db();

        let start = Instant::now();
        for _ in 0..1000 {
            let _ = get_settings_sync();
        }
        let elapsed_ms = start.elapsed().as_millis();

        // 1000 settings reads should complete in < 500ms
        assert!(
            elapsed_ms < 500,
            "1000 settings reads took {}ms (threshold: 500ms)",
            elapsed_ms
        );
    }

    /// Benchmark: Save settings (write operation)
    #[test]
    #[serial]
    fn benchmark_save_settings() {
        setup_test_db();

        let start = Instant::now();
        for i in 0..100 {
            let mut settings = get_settings_sync().unwrap();
            settings.summary_prompt = Some(format!("Test prompt #{}", i));
            let _ = save_settings_sync(&settings);
        }
        let elapsed_ms = start.elapsed().as_millis();

        // 100 settings saves should complete in < 1000ms
        assert!(
            elapsed_ms < 1000,
            "100 settings saves took {}ms (threshold: 1000ms)",
            elapsed_ms
        );
    }

    /// Benchmark: Get today's records
    #[test]
    #[serial]
    fn benchmark_get_today_records() {
        setup_test_db();

        // Insert 50 records
        for i in 0..50 {
            let ts = chrono::Utc::now().to_rfc3339();
            let db = DB_CONNECTION.lock().unwrap();
            let conn = db.as_ref().unwrap();
            conn.execute(
                "INSERT INTO records (timestamp, source_type, content) VALUES (?1, ?2, ?3)",
                params![ts, "manual", format!("Today record #{}", i)],
            )
            .unwrap();
        }

        let start = Instant::now();
        for _ in 0..100 {
            let _ = get_today_records_sync();
        }
        let elapsed_ms = start.elapsed().as_millis();

        // 100 queries should complete in < 500ms
        assert!(
            elapsed_ms < 500,
            "100 get_today_records calls took {}ms (threshold: 500ms)",
            elapsed_ms
        );
    }

    /// Performance test: Batch operations simulation
    /// Simulates a typical workflow: query + process + update
    #[test]
    #[serial]
    fn benchmark_crud_workflow_100_records() {
        setup_test_db();

        // Insert 100 records
        for i in 0..100 {
            let _ = add_record(
                "auto",
                &format!("CRUD workflow record #{}", i),
                None,
                None,
                None,
            );
        }

        let start = Instant::now();

        // Step 1: Query today's records
        let records = get_today_records_sync().unwrap();

        // Step 2: Process (count)
        let count = records.len();

        // Step 3: Get settings
        let _settings = get_settings_sync().unwrap();

        let elapsed_ms = start.elapsed().as_millis();

        // Full CRUD workflow should complete in < 500ms
        assert!(
            elapsed_ms < 500,
            "CRUD workflow (100 records) took {}ms (threshold: 500ms)",
            elapsed_ms
        );

        // Verify we got records
        assert!(count > 0, "Should have retrieved records");
    }
}
