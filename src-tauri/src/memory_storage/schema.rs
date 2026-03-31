use rusqlite::{params, Connection, OptionalExtension};
use std::path::PathBuf;

use crate::crypto;

use super::migration::{self, CURRENT_SCHEMA_VERSION};
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

    // Initialize schema version tracking
    migration::init_schema_version_table(&conn)?;
    let current_version = migration::get_current_version(&conn)?;
    tracing::info!(
        "init_database: Current schema version: {}, target: {}",
        current_version,
        CURRENT_SCHEMA_VERSION
    );

    // Check if migrations have already been recorded
    let migrations_exist = {
        let mut stmt = conn
            .prepare("SELECT COUNT(*) FROM schema_migrations")
            .map_err(|e| format!("Failed to prepare migration check: {}", e))?;
        let count: i32 = stmt
            .query_row([], |row| row.get(0))
            .map_err(|e| format!("Failed to query migrations: {}", e))?;
        count > 0
    };

    // For databases with version at CURRENT_SCHEMA_VERSION, ensure migrations are recorded
    // This handles databases created by legacy code where version was bumped but migration wasn't recorded
    if migrations_exist && current_version >= CURRENT_SCHEMA_VERSION {
        tracing::info!(
            "init_database: Migrations exist and version is current ({}), verifying column completeness",
            current_version
        );
        // Safety net: legacy databases may report version = CURRENT_SCHEMA_VERSION
        // but still be missing columns (e.g. sessions.start_time).  Run the column
        // check regardless of version so upgrades are always repaired.
        migration::ensure_legacy_columns_exist(&conn)?;
    } else if migrations_exist && current_version < CURRENT_SCHEMA_VERSION {
        // Migrations recorded but version behind - this shouldn't happen normally
        // but we can recover by running migrations
        tracing::info!(
            "init_database: Migrations exist but version is behind, running migrations to fix"
        );
        migration::run_migrations(&conn)?;
    } else {
        // No migrations recorded - this is either a new database or a legacy database
        // Check if this is a new database by looking for existing schema
        let table_exists = {
            let mut stmt = conn
                .prepare("PRAGMA table_info(records)")
                .map_err(|e| format!("Failed to prepare table check: {}", e))?;
            let mut rows = stmt
                .query([])
                .map_err(|e| format!("Failed to query table info: {}", e))?;
            // If we can iterate and get at least one column (id), table exists
            rows.next()
                .map_err(|e| format!("Query error: {}", e))?
                .is_some()
        };

        if !table_exists {
            // New database - run migrations to create schema
            tracing::info!("init_database: New database detected, running migrations");
            migration::run_migrations(&conn)?;
        } else {
            // Existing database with legacy schema - run migrations to apply any pending changes
            // The migration system will handle all schema updates idempotently
            tracing::info!(
                "init_database: Legacy database detected (version={}, no migration record), running migrations",
                current_version
            );
            migration::run_migrations(&conn)?;
        }
    }

    // Base table creation is idempotent (CREATE TABLE IF NOT EXISTS) and needed for new databases
    // All column additions are handled by the migration system via run_migrations()

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

    // All schema creation and updates are now handled by the migration system via run_migrations()
    // which is called above for all database initialization paths (new, legacy, and version updates)

    // Migrate plain text API key to encrypted storage BEFORE moving conn
    // This avoids deadlock (no need to lock DB_CONNECTION again)
    crate::write_diagnostic_file("init_database: Migrating API key if needed");
    tracing::info!("init_database: Migrating API key if needed");
    migrate_plain_api_key_with_conn(&conn)?;
    crate::write_diagnostic_file("init_database: API key migration complete");
    tracing::info!("init_database: API key migration complete");

    // Update schema version to track migrations applied by legacy ALTER TABLE statements
    // This ensures new migration system knows current state
    if current_version < CURRENT_SCHEMA_VERSION {
        let applied_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        conn.execute(
            "UPDATE schema_version SET version = ?1, updated_at = ?2 WHERE id = 1",
            params![CURRENT_SCHEMA_VERSION, applied_at],
        )
        .map_err(|e| format!("Failed to update schema version: {}", e))?;

        // Record that we've applied all legacy migrations
        conn.execute(
            "INSERT OR IGNORE INTO schema_migrations (version, description, applied_at) VALUES (?1, ?2, ?3)",
            params![
                CURRENT_SCHEMA_VERSION,
                "Legacy migrations applied via schema.rs init_database",
                applied_at
            ],
        )
        .ok(); // Ignore if already exists

        tracing::info!(
            "init_database: Updated schema version to {} (legacy migrations)",
            CURRENT_SCHEMA_VERSION
        );
    }

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

/// STAB-001 Task 4.2: Check if the database connection is still valid
/// Returns Ok(true) if connection is valid, Ok(false) if reconnect needed, Err on error
pub fn check_connection() -> Result<bool, String> {
    let db = DB_CONNECTION
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;
    Ok(connection_is_valid(db.as_ref()))
}

pub(crate) fn connection_is_valid(conn: Option<&Connection>) -> bool {
    let Some(conn) = conn else {
        return false;
    };

    // Execute a simple query to check if connection is still alive
    conn.query_row("SELECT 1", [], |_| Ok(())).is_ok()
}

/// STAB-001 Task 4.2: Ensure database connection is valid, reconnect if needed
/// This should be called before critical database operations
pub fn ensure_connection() -> Result<(), String> {
    if check_connection()? {
        return Ok(());
    }

    tracing::warn!("Database connection lost, attempting to reconnect...");
    crate::write_diagnostic_file("ensure_connection: Connection lost, reconnecting...");

    // Clear the old connection
    {
        let mut db = DB_CONNECTION
            .lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        *db = None;
    }

    // Reinitialize the database
    init_database()
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
            user_notes TEXT,
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

    // Migrate: add date column if not exists (for existing test databases)
    let _ = conn.execute("ALTER TABLE sessions ADD COLUMN date TEXT", []);

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

    // PERF-004: Composite indexes for query optimization (test DB)
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_timestamp_source_type ON records(timestamp DESC, source_type)",
        [],
    )
    .map_err(|e| format!("Failed to create idx_timestamp_source_type index: {}", e))?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_session_timestamp ON records(session_id, timestamp DESC)",
        [],
    )
    .map_err(|e| format!("Failed to create idx_session_timestamp index: {}", e))?;

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
            onboarding_completed INTEGER DEFAULT 0,
            language TEXT DEFAULT 'en',
            preferred_language TEXT DEFAULT 'zh-CN',
            supported_languages TEXT DEFAULT '[\"zh-CN\",\"en\",\"ja\"]',
            auto_backup_enabled INTEGER DEFAULT 0,
            auto_backup_interval TEXT DEFAULT 'daily',
            auto_backup_retention INTEGER DEFAULT 5,
            last_auto_backup_at TEXT,
            auto_detect_vault_by_window INTEGER DEFAULT 0,
            custom_export_template TEXT
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

    // DEBT-001: Ensure test isolation by clearing data tables after schema creation.
    // This prevents leftover data from previous tests affecting current test results.
    // Tables are recreated above, so this only clears data, not schema.
    let _ = conn.execute("DELETE FROM records", []);
    let _ = conn.execute("DELETE FROM sessions", []);
    let _ = conn.execute("DELETE FROM manual_tags", []);
    let _ = conn.execute("DELETE FROM record_manual_tags", []);
    let _ = conn.execute("DELETE FROM offline_queue", []);
    let _ = conn.execute("DELETE FROM silent_pattern_stats", []);
    let _ = conn.execute("DELETE FROM work_time_activity", []);
    let _ = conn.execute("DELETE FROM schema_migrations", []);
    let _ = conn.execute("DELETE FROM schema_version", []);
    // Reset settings to default (keep row with id=1)
    let _ = conn.execute("DELETE FROM settings WHERE id != 1", []);
    let _ = conn.execute("INSERT OR IGNORE INTO settings (id) VALUES (1)", []);

    Ok(())
}
