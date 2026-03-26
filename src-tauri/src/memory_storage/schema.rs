use rusqlite::{params, Connection, OptionalExtension};
use std::path::PathBuf;

use crate::crypto;

use super::DB_CONNECTION;

fn get_db_path() -> PathBuf {
    crate::get_app_data_dir().join("data").join("local.db")
}

pub fn init_database() -> Result<(), String> {
    crate::write_diagnostic_file("init_database: Starting");
    tracing::info!("init_database: Starting");

    let db_dir = crate::get_app_data_dir().join("data");
    crate::write_diagnostic_file(&format!(
        "init_database: Creating data directory: {:?}",
        db_dir
    ));
    tracing::info!("init_database: Creating data directory: {:?}", db_dir);

    std::fs::create_dir_all(&db_dir).map_err(|e| {
        let msg = format!("Failed to create data directory {:?}: {}", db_dir, e);
        crate::write_diagnostic_file(&format!("init_database: FAILED to create data dir: {}", e));
        tracing::error!("{}", msg);
        msg
    })?;
    crate::write_diagnostic_file("init_database: Data directory ready");
    tracing::info!("init_database: Data directory ready");

    let db_path = get_db_path();
    crate::write_diagnostic_file(&format!(
        "init_database: Opening database at: {:?}",
        db_path
    ));
    tracing::info!("init_database: Opening database at: {:?}", db_path);

    let conn = Connection::open(&db_path).map_err(|e| {
        let msg = format!("Failed to open database at {:?}: {}", db_path, e);
        crate::write_diagnostic_file(&format!("init_database: FAILED to open database: {}", e));
        tracing::error!("{}", msg);
        msg
    })?;
    crate::write_diagnostic_file("init_database: Database connection opened");
    tracing::info!("init_database: Database connection opened");

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
    .map_err(|e| {
        let msg = format!("Failed to create records table: {}", e);
        tracing::error!("{}", msg);
        msg
    })?;
    tracing::info!("init_database: records table ready");

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

    // FEAT-005: 用户手动备注字段 (#66)
    let _ = conn.execute("ALTER TABLE records ADD COLUMN user_notes TEXT", []);

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

    // INT-004: Slack 通知配置
    let _ = conn.execute("ALTER TABLE settings ADD COLUMN slack_webhook_url TEXT", []);

    // FEAT-006: 仅截图模式 (#65)
    let _ = conn.execute(
        "ALTER TABLE settings ADD COLUMN capture_only_mode INTEGER DEFAULT 0",
        [],
    );

    // AI-006: 自定义 API Headers (#68)
    let _ = conn.execute(
        "ALTER TABLE settings ADD COLUMN custom_headers TEXT DEFAULT '[]'",
        [],
    );

    // INT-004: DingTalk 通知配置
    let _ = conn.execute(
        "ALTER TABLE settings ADD COLUMN dingtalk_webhook_url TEXT",
        [],
    );

    // EXP-002: 截图质量过滤配置
    let _ = conn.execute(
        "ALTER TABLE settings ADD COLUMN quality_filter_enabled INTEGER DEFAULT 1",
        [],
    );
    let _ = conn.execute(
        "ALTER TABLE settings ADD COLUMN quality_filter_threshold REAL DEFAULT 0.3",
        [],
    );

    // SESSION-001: 工作时段管理配置
    let _ = conn.execute(
        "ALTER TABLE settings ADD COLUMN session_gap_minutes INTEGER DEFAULT 30",
        [],
    );

    // PERF-001: 代理配置
    let _ = conn.execute(
        "ALTER TABLE settings ADD COLUMN proxy_enabled INTEGER DEFAULT 0",
        [],
    );
    let _ = conn.execute("ALTER TABLE settings ADD COLUMN proxy_host TEXT", []);
    let _ = conn.execute(
        "ALTER TABLE settings ADD COLUMN proxy_port INTEGER DEFAULT 8080",
        [],
    );
    let _ = conn.execute("ALTER TABLE settings ADD COLUMN proxy_username TEXT", []);
    let _ = conn.execute("ALTER TABLE settings ADD COLUMN proxy_password TEXT", []);
    // PERF-001: 测试模型名称
    let _ = conn.execute("ALTER TABLE settings ADD COLUMN test_model_name TEXT", []);

    // PERF-002: 新用户引导完成标志
    let _ = conn.execute(
        "ALTER TABLE settings ADD COLUMN onboarding_completed INTEGER DEFAULT 0",
        [],
    );

    // SESSION-001: sessions 表 - 工作时段管理
    conn.execute(
        "CREATE TABLE IF NOT EXISTS sessions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            date TEXT NOT NULL,
            start_time TEXT NOT NULL,
            end_time TEXT,
            ai_summary TEXT,
            user_summary TEXT,
            context_for_next TEXT,
            status TEXT DEFAULT 'active'
        )",
        [],
    )
    .map_err(|e| format!("Failed to create sessions table: {}", e))?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_sessions_date ON sessions(date)",
        [],
    )
    .map_err(|e| format!("Failed to create idx_sessions_date index: {}", e))?;

    // SESSION-001: records 表扩展 - 时段关联和分析状态
    let _ = conn.execute(
        "ALTER TABLE records ADD COLUMN session_id INTEGER REFERENCES sessions(id)",
        [],
    );
    let _ = conn.execute(
        "ALTER TABLE records ADD COLUMN analysis_status TEXT DEFAULT 'pending'",
        [],
    );

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_session_id ON records(session_id)",
        [],
    )
    .map_err(|e| format!("Failed to create idx_session_id index: {}", e))?;

    // PERF-004: Add missing timestamp index and composite indexes for query optimization
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_timestamp ON records(timestamp DESC)",
        [],
    )
    .map_err(|e| format!("Failed to create idx_timestamp index: {}", e))?;

    // Composite index: time range + source type filtering (covers date筛选查询)
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_timestamp_source_type ON records(timestamp DESC, source_type)",
        [],
    )
    .map_err(|e| format!("Failed to create idx_timestamp_source_type index: {}", e))?;

    // Composite index: session + timestamp for session-scoped queries (覆盖时段内截图排序)
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_session_timestamp ON records(session_id, timestamp DESC)",
        [],
    )
    .map_err(|e| format!("Failed to create idx_session_timestamp index: {}", e))?;

    // Covering index: reduces table lookups for common select columns
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_timestamp_covering ON records(timestamp DESC, id, content, screenshot_path)",
        [],
    )
    .map_err(|e| format!("Failed to create idx_timestamp_covering index: {}", e))?;

    // DATA-002: FTS5 全文搜索虚拟表
    // 使用 unicode61 tokenizer（Windows 兼容性：移除 tokenchars 以避免解析错误）
    tracing::info!("init_database: Creating FTS5 table");
    conn.execute(
        "CREATE VIRTUAL TABLE IF NOT EXISTS records_fts USING fts5(
            content,
            content='records',
            content_rowid='id',
            tokenize='unicode61'
        )",
        [],
    )
    .map_err(|e| {
        let msg = format!("Failed to create FTS5 table: {}", e);
        tracing::error!("{}", msg);
        msg
    })?;
    tracing::info!("init_database: FTS5 table ready");

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
    crate::write_diagnostic_file("init_database: Creating offline queue table");
    tracing::info!("init_database: Creating offline queue table");
    crate::offline_queue::create_offline_queue_table(&conn)?;
    crate::write_diagnostic_file("init_database: Offline queue table ready");
    tracing::info!("init_database: Offline queue table ready");

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

    // Migrate plain text API key to encrypted storage BEFORE moving conn
    // This avoids deadlock (no need to lock DB_CONNECTION again)
    crate::write_diagnostic_file("init_database: Migrating API key if needed");
    tracing::info!("init_database: Migrating API key if needed");
    migrate_plain_api_key_with_conn(&conn)?;
    crate::write_diagnostic_file("init_database: API key migration complete");
    tracing::info!("init_database: API key migration complete");

    crate::write_diagnostic_file("init_database: Acquiring DB connection lock");
    let mut db = DB_CONNECTION.lock().map_err(|e| {
        let msg = format!("Lock error: {}", e);
        crate::write_diagnostic_file(&format!("init_database: Lock error: {}", e));
        tracing::error!("{}", msg);
        msg
    })?;
    crate::write_diagnostic_file("init_database: DB connection lock acquired");
    tracing::info!("init_database: DB connection lock acquired");
    *db = Some(conn);
    crate::write_diagnostic_file("init_database: DB connection stored");
    tracing::info!("init_database: DB connection stored");

    crate::write_diagnostic_file(&format!(
        "init_database: Database initialized at {:?}",
        db_path
    ));
    tracing::info!("Database initialized at {:?}", db_path);
    Ok(())
}

/// Migrate plain text API key to encrypted storage
/// Takes a connection reference to avoid deadlock when called from init_database
fn migrate_plain_api_key_with_conn(conn: &Connection) -> Result<(), String> {
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
            tags TEXT,
            session_id INTEGER REFERENCES sessions(id),
            analysis_status TEXT DEFAULT 'pending'
        )",
        [],
    )
    .map_err(|e| format!("Failed to create records table: {}", e))?;

    // Create sessions table (SESSION-001)
    conn.execute(
        "CREATE TABLE IF NOT EXISTS sessions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            date TEXT NOT NULL,
            start_time TEXT NOT NULL,
            end_time TEXT,
            ai_summary TEXT,
            user_summary TEXT,
            context_for_next TEXT,
            status TEXT DEFAULT 'active'
        )",
        [],
    )
    .map_err(|e| format!("Failed to create sessions table: {}", e))?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_sessions_date ON sessions(date)",
        [],
    )
    .map_err(|e| format!("Failed to create idx_sessions_date index: {}", e))?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_session_id ON records(session_id)",
        [],
    )
    .map_err(|e| format!("Failed to create idx_session_id index: {}", e))?;

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
            github_repositories TEXT DEFAULT '[]',
            slack_webhook_url TEXT,
            dingtalk_webhook_url TEXT,
            capture_only_mode INTEGER DEFAULT 0,
            custom_headers TEXT DEFAULT '[]',
            quality_filter_enabled INTEGER DEFAULT 1,
            quality_filter_threshold REAL DEFAULT 0.3,
            session_gap_minutes INTEGER DEFAULT 30,
            proxy_enabled INTEGER DEFAULT 0,
            proxy_host TEXT,
            proxy_port INTEGER DEFAULT 8080,
            proxy_username TEXT,
            proxy_password TEXT,
            test_model_name TEXT,
            onboarding_completed INTEGER DEFAULT 0
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
