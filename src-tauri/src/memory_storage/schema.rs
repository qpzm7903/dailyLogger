use rusqlite::{params, Connection, OptionalExtension};
use std::path::PathBuf;

use crate::crypto;

use super::DB_CONNECTION;

fn get_db_path() -> PathBuf {
    crate::get_app_data_dir().join("data").join("local.db")
}

pub fn init_database() -> Result<(), String> {
    let db_dir = crate::get_app_data_dir().join("data");
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

    // INT-002: Logseq 导出支持
    let _ = conn.execute(
        "ALTER TABLE settings ADD COLUMN logseq_graphs TEXT DEFAULT '[]'",
        [],
    );

    // INT-001: Notion 导出支持
    let _ = conn.execute("ALTER TABLE settings ADD COLUMN notion_api_key TEXT", []);
    let _ = conn.execute(
        "ALTER TABLE settings ADD COLUMN notion_database_id TEXT",
        [],
    );

    // INT-003: GitHub 工时统计配置
    let _ = conn.execute("ALTER TABLE settings ADD COLUMN github_token TEXT", []);
    let _ = conn.execute(
        "ALTER TABLE settings ADD COLUMN github_repositories TEXT DEFAULT '[]'",
        [],
    );

    // DATA-002: FTS5 全文搜索虚拟表
    // 使用 unicode61 tokenizer（Windows 兼容性：移除 tokenchars 以避免解析错误）
    conn.execute(
        "CREATE VIRTUAL TABLE IF NOT EXISTS records_fts USING fts5(
            content,
            content='records',
            content_rowid='id',
            tokenize='unicode61'
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

    // Create offline queue table
    crate::offline_queue::create_offline_queue_table(&conn)?;

    // DEBT-005: Learning data persistence tables
    // Silent pattern stats for SMART-002 auto-threshold adjustment
    conn.execute(
        "CREATE TABLE IF NOT EXISTS silent_pattern_stats (
            date TEXT NOT NULL,
            hour INTEGER NOT NULL,
            silent_captures INTEGER NOT NULL DEFAULT 0,
            change_captures INTEGER NOT NULL DEFAULT 0,
            PRIMARY KEY (date, hour)
        )",
        [],
    )
    .map_err(|e| format!("Failed to create silent_pattern_stats table: {}", e))?;

    // Work time activity for SMART-003 work time detection
    conn.execute(
        "CREATE TABLE IF NOT EXISTS work_time_activity (
            date TEXT NOT NULL,
            hour INTEGER NOT NULL,
            capture_count INTEGER NOT NULL DEFAULT 1,
            PRIMARY KEY (date, hour)
        )",
        [],
    )
    .map_err(|e| format!("Failed to create work_time_activity table: {}", e))?;

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

#[cfg(test)]
pub fn init_test_database(conn: &Connection) -> Result<(), String> {
    // Create records table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS records (
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
    .map_err(|e| format!("Failed to create records table: {}", e))?;

    // Create settings table
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
            last_monthly_report_path TEXT,
            custom_report_prompt TEXT,
            last_custom_report_path TEXT,
            obsidian_vaults TEXT DEFAULT '[]',
            comparison_report_prompt TEXT,
            logseq_graphs TEXT DEFAULT '[]',
            notion_api_key TEXT,
            notion_database_id TEXT,
            github_token TEXT,
            github_repositories TEXT DEFAULT '[]'
        )",
        [],
    )
    .map_err(|e| format!("Failed to create settings table: {}", e))?;

    conn.execute("INSERT OR IGNORE INTO settings (id) VALUES (1)", [])
        .map_err(|e| format!("Failed to initialize settings: {}", e))?;

    // Create FTS5 table
    conn.execute(
        "CREATE VIRTUAL TABLE IF NOT EXISTS records_fts USING fts5(
            content,
            content='records',
            content_rowid='id',
            tokenize='unicode61'
        )",
        [],
    )
    .map_err(|e| format!("Failed to create FTS5 table: {}", e))?;

    // FTS5 triggers
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

    // Create manual tags tables
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

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_record_manual_tags_tag_id ON record_manual_tags(tag_id)",
        [],
    )
    .map_err(|e| format!("Failed to create index: {}", e))?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_manual_tags_name ON manual_tags(name)",
        [],
    )
    .map_err(|e| format!("Failed to create index: {}", e))?;

    // Create offline queue table
    crate::offline_queue::create_offline_queue_table(conn)?;

    // DEBT-005: Learning data persistence tables
    conn.execute(
        "CREATE TABLE IF NOT EXISTS silent_pattern_stats (
            date TEXT NOT NULL,
            hour INTEGER NOT NULL,
            silent_captures INTEGER NOT NULL DEFAULT 0,
            change_captures INTEGER NOT NULL DEFAULT 0,
            PRIMARY KEY (date, hour)
        )",
        [],
    )
    .map_err(|e| format!("Failed to create silent_pattern_stats table: {}", e))?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS work_time_activity (
            date TEXT NOT NULL,
            hour INTEGER NOT NULL,
            capture_count INTEGER NOT NULL DEFAULT 1,
            PRIMARY KEY (date, hour)
        )",
        [],
    )
    .map_err(|e| format!("Failed to create work_time_activity table: {}", e))?;

    Ok(())
}
