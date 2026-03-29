//! Database migration system with version tracking
//!
//! This module provides a structured migration mechanism that:
//! - Tracks schema version in a `schema_version` table
//! - Records each migration in `schema_migrations` history table
//! - Supports idempotent migrations (safe to run multiple times)
//! - Applies migrations in order during database initialization

use rusqlite::{params, Connection};
use std::time::SystemTime;

/// Current schema version - increment when adding new migrations
pub const CURRENT_SCHEMA_VERSION: i32 = 1;

/// Represents a single database migration
#[derive(Debug, Clone)]
pub struct Migration {
    /// Version number for this migration (must be unique, incrementing)
    pub version: i32,
    /// Description of what this migration does
    pub description: &'static str,
    /// SQL statements to execute for this migration
    pub sql: &'static str,
}

impl Migration {
    /// Execute this migration on the given connection
    fn execute(&self, conn: &Connection) -> Result<(), String> {
        tracing::info!("Applying migration v{}: {}", self.version, self.description);

        // Begin transaction for atomic execution
        conn.execute("BEGIN IMMEDIATE", [])
            .map_err(|e| format!("Failed to begin migration transaction: {}", e))?;

        let result = (|| {
            // For v1 migration: handle legacy sessions table that may be missing the date column
            // (if sessions table existed before the date column was added)
            if self.version == 1 {
                // Check if sessions table exists and has date column
                let table_exists: bool = conn
                    .query_row(
                        "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='table' AND name='sessions'",
                        [],
                        |row| row.get(0),
                    )
                    .unwrap_or(false);

                if table_exists {
                    // Table exists - check if date column is missing
                    let column_exists: bool = conn
                        .query_row(
                            "SELECT COUNT(*) > 0 FROM PRAGMA table_info('sessions') WHERE name = 'date'",
                            [],
                            |row| row.get(0),
                        )
                        .unwrap_or(false);

                    if !column_exists {
                        // sessions table exists but is missing date column - add it
                        conn.execute(
                            "ALTER TABLE sessions ADD COLUMN date TEXT NOT NULL DEFAULT ''",
                            [],
                        )
                        .map_err(|e| format!("Failed to add date column to sessions: {}", e))?;
                    }
                }
                // If table doesn't exist, CREATE TABLE IF NOT EXISTS in SQL will create it

                // Helper to add a column if it doesn't exist (idempotent)
                // Uses "ALTER TABLE ADD COLUMN" which fails with "duplicate column name" if column exists
                let add_column_if_not_exists =
                    |conn: &Connection, table: &str, col_def: &str| -> Result<(), String> {
                        let sql = format!("ALTER TABLE {} ADD COLUMN {}", table, col_def);
                        match conn.execute(&sql, []) {
                            Ok(_) => tracing::debug!(
                                "Added column {} to {}",
                                col_def.split_whitespace().next().unwrap_or("?"),
                                table
                            ),
                            Err(e) => {
                                let e_str = e.to_string();
                                if e_str.contains("duplicate column name") {
                                    // Column already exists, that's fine
                                } else {
                                    return Err(e.to_string());
                                }
                            }
                        }
                        Ok(())
                    };

                // For legacy databases: add missing columns to records table
                let records_table_exists: bool = conn
                    .query_row(
                        "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='table' AND name='records'",
                        [],
                        |row| row.get(0),
                    )
                    .unwrap_or(false);

                if records_table_exists {
                    add_column_if_not_exists(conn, "records", "monitor_info TEXT")?;
                    add_column_if_not_exists(conn, "records", "tags TEXT")?;
                    add_column_if_not_exists(conn, "records", "user_notes TEXT")?;
                    add_column_if_not_exists(
                        conn,
                        "records",
                        "session_id INTEGER REFERENCES sessions(id)",
                    )?;
                    add_column_if_not_exists(
                        conn,
                        "records",
                        "analysis_status TEXT DEFAULT 'pending'",
                    )?;
                }

                // For legacy databases: add missing columns to settings table
                let settings_table_exists: bool = conn
                    .query_row(
                        "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='table' AND name='settings'",
                        [],
                        |row| row.get(0),
                    )
                    .unwrap_or(false);

                if settings_table_exists {
                    add_column_if_not_exists(conn, "settings", "summary_model_name TEXT")?;
                    add_column_if_not_exists(conn, "settings", "analysis_prompt TEXT")?;
                    add_column_if_not_exists(conn, "settings", "summary_prompt TEXT")?;
                    add_column_if_not_exists(
                        conn,
                        "settings",
                        "change_threshold INTEGER DEFAULT 3",
                    )?;
                    add_column_if_not_exists(
                        conn,
                        "settings",
                        "max_silent_minutes INTEGER DEFAULT 30",
                    )?;
                    add_column_if_not_exists(
                        conn,
                        "settings",
                        "summary_title_format TEXT DEFAULT '工作日报 - {date}'",
                    )?;
                    add_column_if_not_exists(
                        conn,
                        "settings",
                        "include_manual_records INTEGER DEFAULT 1",
                    )?;
                    add_column_if_not_exists(
                        conn,
                        "settings",
                        "window_whitelist TEXT DEFAULT '[]'",
                    )?;
                    add_column_if_not_exists(
                        conn,
                        "settings",
                        "window_blacklist TEXT DEFAULT '[]'",
                    )?;
                    add_column_if_not_exists(
                        conn,
                        "settings",
                        "use_whitelist_only INTEGER DEFAULT 0",
                    )?;
                    add_column_if_not_exists(
                        conn,
                        "settings",
                        "auto_adjust_silent INTEGER DEFAULT 1",
                    )?;
                    add_column_if_not_exists(
                        conn,
                        "settings",
                        "silent_adjustment_paused_until TEXT DEFAULT NULL",
                    )?;
                    add_column_if_not_exists(
                        conn,
                        "settings",
                        "auto_detect_work_time INTEGER DEFAULT 1",
                    )?;
                    add_column_if_not_exists(
                        conn,
                        "settings",
                        "use_custom_work_time INTEGER DEFAULT 0",
                    )?;
                    add_column_if_not_exists(
                        conn,
                        "settings",
                        "custom_work_time_start TEXT DEFAULT '09:00'",
                    )?;
                    add_column_if_not_exists(
                        conn,
                        "settings",
                        "custom_work_time_end TEXT DEFAULT '18:00'",
                    )?;
                    add_column_if_not_exists(
                        conn,
                        "settings",
                        "learned_work_time TEXT DEFAULT NULL",
                    )?;
                    add_column_if_not_exists(
                        conn,
                        "settings",
                        "capture_mode TEXT DEFAULT 'primary'",
                    )?;
                    add_column_if_not_exists(
                        conn,
                        "settings",
                        "selected_monitor_index INTEGER DEFAULT 0",
                    )?;
                    add_column_if_not_exists(conn, "settings", "tag_categories TEXT DEFAULT '[]'")?;
                    add_column_if_not_exists(conn, "settings", "is_ollama INTEGER DEFAULT 0")?;
                    add_column_if_not_exists(conn, "settings", "weekly_report_prompt TEXT")?;
                    add_column_if_not_exists(
                        conn,
                        "settings",
                        "weekly_report_day INTEGER DEFAULT 0",
                    )?;
                    add_column_if_not_exists(conn, "settings", "last_weekly_report_path TEXT")?;
                    add_column_if_not_exists(conn, "settings", "monthly_report_prompt TEXT")?;
                    add_column_if_not_exists(conn, "settings", "last_monthly_report_path TEXT")?;
                    add_column_if_not_exists(conn, "settings", "custom_report_prompt TEXT")?;
                    add_column_if_not_exists(conn, "settings", "last_custom_report_path TEXT")?;
                    add_column_if_not_exists(
                        conn,
                        "settings",
                        "obsidian_vaults TEXT DEFAULT '[]'",
                    )?;
                    add_column_if_not_exists(conn, "settings", "comparison_report_prompt TEXT")?;
                    add_column_if_not_exists(conn, "settings", "logseq_graphs TEXT DEFAULT '[]'")?;
                    add_column_if_not_exists(conn, "settings", "notion_api_key TEXT")?;
                    add_column_if_not_exists(conn, "settings", "notion_database_id TEXT")?;
                    add_column_if_not_exists(conn, "settings", "github_token TEXT")?;
                    add_column_if_not_exists(
                        conn,
                        "settings",
                        "github_repositories TEXT DEFAULT '[]'",
                    )?;
                    add_column_if_not_exists(conn, "settings", "slack_webhook_url TEXT")?;
                    add_column_if_not_exists(conn, "settings", "dingtalk_webhook_url TEXT")?;
                    add_column_if_not_exists(
                        conn,
                        "settings",
                        "capture_only_mode INTEGER DEFAULT 0",
                    )?;
                    add_column_if_not_exists(conn, "settings", "custom_headers TEXT DEFAULT '[]'")?;
                    add_column_if_not_exists(
                        conn,
                        "settings",
                        "quality_filter_enabled INTEGER DEFAULT 1",
                    )?;
                    add_column_if_not_exists(
                        conn,
                        "settings",
                        "quality_filter_threshold REAL DEFAULT 0.3",
                    )?;
                    add_column_if_not_exists(
                        conn,
                        "settings",
                        "session_gap_minutes INTEGER DEFAULT 30",
                    )?;
                    add_column_if_not_exists(conn, "settings", "proxy_enabled INTEGER DEFAULT 0")?;
                    add_column_if_not_exists(conn, "settings", "proxy_host TEXT")?;
                    add_column_if_not_exists(conn, "settings", "proxy_port INTEGER DEFAULT 8080")?;
                    add_column_if_not_exists(conn, "settings", "proxy_username TEXT")?;
                    add_column_if_not_exists(conn, "settings", "proxy_password TEXT")?;
                    add_column_if_not_exists(conn, "settings", "test_model_name TEXT")?;
                    add_column_if_not_exists(
                        conn,
                        "settings",
                        "onboarding_completed INTEGER DEFAULT 0",
                    )?;
                    add_column_if_not_exists(conn, "settings", "language TEXT DEFAULT 'en'")?;
                    add_column_if_not_exists(
                        conn,
                        "settings",
                        "preferred_language TEXT DEFAULT 'zh-CN'",
                    )?;
                    add_column_if_not_exists(
                        conn,
                        "settings",
                        "supported_languages TEXT DEFAULT '[\"zh-CN\",\"en\",\"ja\"]'",
                    )?;
                    add_column_if_not_exists(
                        conn,
                        "settings",
                        "auto_backup_enabled INTEGER DEFAULT 0",
                    )?;
                    add_column_if_not_exists(
                        conn,
                        "settings",
                        "auto_backup_interval TEXT DEFAULT 'daily'",
                    )?;
                    add_column_if_not_exists(
                        conn,
                        "settings",
                        "auto_backup_retention INTEGER DEFAULT 5",
                    )?;
                    add_column_if_not_exists(conn, "settings", "last_auto_backup_at TEXT")?;
                    add_column_if_not_exists(
                        conn,
                        "settings",
                        "auto_detect_vault_by_window INTEGER DEFAULT 0",
                    )?;
                    add_column_if_not_exists(conn, "settings", "custom_export_template TEXT")?;
                }
            }

            // Execute the migration SQL
            conn.execute_batch(self.sql)
                .map_err(|e| format!("Failed to execute migration v{}: {}", self.version, e))?;

            // Record the migration in history
            let applied_at = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0);

            conn.execute(
                "INSERT INTO schema_migrations (version, description, applied_at) VALUES (?1, ?2, ?3)",
                params![self.version, self.description, applied_at],
            )
            .map_err(|e| format!("Failed to record migration v{}: {}", self.version, e))?;

            // Update schema version
            conn.execute(
                "UPDATE schema_version SET version = ?1, updated_at = ?2 WHERE id = 1",
                params![self.version, applied_at],
            )
            .map_err(|e| format!("Failed to update schema version: {}", e))?;

            Ok(())
        })();

        match result {
            Ok(()) => {
                conn.execute("COMMIT", [])
                    .map_err(|e| format!("Failed to commit migration: {}", e))?;
                tracing::info!("Migration v{} applied successfully", self.version);
                Ok(())
            }
            Err(e) => {
                conn.execute("ROLLBACK", []).ok();
                tracing::error!("Migration v{} failed: {}", self.version, e);
                Err(e)
            }
        }
    }
}

/// Get all registered migrations in order
fn get_migrations() -> Vec<Migration> {
    vec![Migration {
        version: 1,
        description: "Initial schema - create all base tables and indexes",
        sql: r#"
            -- records table (base schema)
            CREATE TABLE IF NOT EXISTS records (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp TEXT NOT NULL,
                source_type TEXT NOT NULL,
                content TEXT NOT NULL,
                screenshot_path TEXT
            );

            -- Migrate: add screenshot_path column if not exists
            -- (already handled by CREATE TABLE IF NOT EXISTS, but kept for explicit idempotency)
            -- Note: In SQLite, CREATE TABLE IF NOT EXISTS doesn't add columns to existing tables

            -- settings table (base schema)
            CREATE TABLE IF NOT EXISTS settings (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                api_base_url TEXT,
                api_key TEXT,
                model_name TEXT,
                screenshot_interval INTEGER DEFAULT 5,
                summary_time TEXT DEFAULT '18:00',
                obsidian_path TEXT,
                auto_capture_enabled INTEGER DEFAULT 0,
                last_summary_path TEXT
            );

            -- Insert default settings row
            INSERT OR IGNORE INTO settings (id) VALUES (1);

            -- sessions table (SESSION-001)
            CREATE TABLE IF NOT EXISTS sessions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                date TEXT NOT NULL,
                start_time TEXT NOT NULL,
                end_time TEXT,
                ai_summary TEXT,
                user_summary TEXT,
                context_for_next TEXT,
                status TEXT DEFAULT 'active'
            );

            CREATE INDEX IF NOT EXISTS idx_sessions_date ON sessions(date);

            -- records table extensions
            ALTER TABLE records ADD COLUMN monitor_info TEXT;
            ALTER TABLE records ADD COLUMN tags TEXT;
            ALTER TABLE records ADD COLUMN user_notes TEXT;
            ALTER TABLE records ADD COLUMN session_id INTEGER REFERENCES sessions(id);
            ALTER TABLE records ADD COLUMN analysis_status TEXT DEFAULT 'pending';

            CREATE INDEX IF NOT EXISTS idx_session_id ON records(session_id);
            CREATE INDEX IF NOT EXISTS idx_timestamp ON records(timestamp DESC);
            CREATE INDEX IF NOT EXISTS idx_timestamp_source_type ON records(timestamp DESC, source_type);
            CREATE INDEX IF NOT EXISTS idx_session_timestamp ON records(session_id, timestamp DESC);
            CREATE INDEX IF NOT EXISTS idx_timestamp_covering ON records(timestamp DESC, id, content, screenshot_path);

            -- settings table extensions (field additions over time)
            ALTER TABLE settings ADD COLUMN summary_model_name TEXT;
            ALTER TABLE settings ADD COLUMN analysis_prompt TEXT;
            ALTER TABLE settings ADD COLUMN summary_prompt TEXT;
            ALTER TABLE settings ADD COLUMN change_threshold INTEGER DEFAULT 3;
            ALTER TABLE settings ADD COLUMN max_silent_minutes INTEGER DEFAULT 30;
            ALTER TABLE settings ADD COLUMN summary_title_format TEXT DEFAULT '工作日报 - {date}';
            ALTER TABLE settings ADD COLUMN include_manual_records INTEGER DEFAULT 1;
            ALTER TABLE settings ADD COLUMN window_whitelist TEXT DEFAULT '[]';
            ALTER TABLE settings ADD COLUMN window_blacklist TEXT DEFAULT '[]';
            ALTER TABLE settings ADD COLUMN use_whitelist_only INTEGER DEFAULT 0;
            ALTER TABLE settings ADD COLUMN auto_adjust_silent INTEGER DEFAULT 1;
            ALTER TABLE settings ADD COLUMN silent_adjustment_paused_until TEXT DEFAULT NULL;
            ALTER TABLE settings ADD COLUMN auto_detect_work_time INTEGER DEFAULT 1;
            ALTER TABLE settings ADD COLUMN use_custom_work_time INTEGER DEFAULT 0;
            ALTER TABLE settings ADD COLUMN custom_work_time_start TEXT DEFAULT '09:00';
            ALTER TABLE settings ADD COLUMN custom_work_time_end TEXT DEFAULT '18:00';
            ALTER TABLE settings ADD COLUMN learned_work_time TEXT DEFAULT NULL;
            ALTER TABLE settings ADD COLUMN capture_mode TEXT DEFAULT 'primary';
            ALTER TABLE settings ADD COLUMN selected_monitor_index INTEGER DEFAULT 0;
            ALTER TABLE settings ADD COLUMN tag_categories TEXT DEFAULT '[]';
            ALTER TABLE settings ADD COLUMN is_ollama INTEGER DEFAULT 0;
            ALTER TABLE settings ADD COLUMN weekly_report_prompt TEXT;
            ALTER TABLE settings ADD COLUMN weekly_report_day INTEGER DEFAULT 0;
            ALTER TABLE settings ADD COLUMN last_weekly_report_path TEXT;
            ALTER TABLE settings ADD COLUMN monthly_report_prompt TEXT;
            ALTER TABLE settings ADD COLUMN last_monthly_report_path TEXT;
            ALTER TABLE settings ADD COLUMN custom_report_prompt TEXT;
            ALTER TABLE settings ADD COLUMN last_custom_report_path TEXT;
            ALTER TABLE settings ADD COLUMN obsidian_vaults TEXT DEFAULT '[]';
            ALTER TABLE settings ADD COLUMN comparison_report_prompt TEXT;
            ALTER TABLE settings ADD COLUMN logseq_graphs TEXT DEFAULT '[]';
            ALTER TABLE settings ADD COLUMN notion_api_key TEXT;
            ALTER TABLE settings ADD COLUMN notion_database_id TEXT;
            ALTER TABLE settings ADD COLUMN github_token TEXT;
            ALTER TABLE settings ADD COLUMN github_repositories TEXT DEFAULT '[]';
            ALTER TABLE settings ADD COLUMN slack_webhook_url TEXT;
            ALTER TABLE settings ADD COLUMN dingtalk_webhook_url TEXT;
            ALTER TABLE settings ADD COLUMN capture_only_mode INTEGER DEFAULT 0;
            ALTER TABLE settings ADD COLUMN custom_headers TEXT DEFAULT '[]';
            ALTER TABLE settings ADD COLUMN quality_filter_enabled INTEGER DEFAULT 1;
            ALTER TABLE settings ADD COLUMN quality_filter_threshold REAL DEFAULT 0.3;
            ALTER TABLE settings ADD COLUMN session_gap_minutes INTEGER DEFAULT 30;
            ALTER TABLE settings ADD COLUMN proxy_enabled INTEGER DEFAULT 0;
            ALTER TABLE settings ADD COLUMN proxy_host TEXT;
            ALTER TABLE settings ADD COLUMN proxy_port INTEGER DEFAULT 8080;
            ALTER TABLE settings ADD COLUMN proxy_username TEXT;
            ALTER TABLE settings ADD COLUMN proxy_password TEXT;
            ALTER TABLE settings ADD COLUMN test_model_name TEXT;
            ALTER TABLE settings ADD COLUMN onboarding_completed INTEGER DEFAULT 0;
            ALTER TABLE settings ADD COLUMN language TEXT DEFAULT 'en';
            ALTER TABLE settings ADD COLUMN preferred_language TEXT DEFAULT 'zh-CN';
            ALTER TABLE settings ADD COLUMN supported_languages TEXT DEFAULT '["zh-CN","en","ja"]';
            ALTER TABLE settings ADD COLUMN auto_backup_enabled INTEGER DEFAULT 0;
            ALTER TABLE settings ADD COLUMN auto_backup_interval TEXT DEFAULT 'daily';
            ALTER TABLE settings ADD COLUMN auto_backup_retention INTEGER DEFAULT 5;
            ALTER TABLE settings ADD COLUMN last_auto_backup_at TEXT;
            ALTER TABLE settings ADD COLUMN auto_detect_vault_by_window INTEGER DEFAULT 0;
            ALTER TABLE settings ADD COLUMN custom_export_template TEXT;

            -- manual_tags table
            CREATE TABLE IF NOT EXISTS manual_tags (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                color TEXT NOT NULL DEFAULT 'blue',
                created_at TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_manual_tags_name ON manual_tags(name);

            -- record_manual_tags table
            CREATE TABLE IF NOT EXISTS record_manual_tags (
                record_id INTEGER NOT NULL,
                tag_id INTEGER NOT NULL,
                PRIMARY KEY (record_id, tag_id),
                FOREIGN KEY (record_id) REFERENCES records(id) ON DELETE CASCADE,
                FOREIGN KEY (tag_id) REFERENCES manual_tags(id) ON DELETE CASCADE
            );

            CREATE INDEX IF NOT EXISTS idx_record_manual_tags_tag_id ON record_manual_tags(tag_id);

            -- FTS5 virtual table
            CREATE VIRTUAL TABLE IF NOT EXISTS records_fts USING fts5(
                content,
                content='records',
                content_rowid='id',
                tokenize='unicode61'
            );

            -- FTS5 triggers
            CREATE TRIGGER IF NOT EXISTS records_ai AFTER INSERT ON records BEGIN
                INSERT INTO records_fts(rowid, content) VALUES (new.id, new.content);
            END;

            CREATE TRIGGER IF NOT EXISTS records_ad AFTER DELETE ON records BEGIN
                INSERT INTO records_fts(records_fts, rowid, content)
                VALUES ('delete', old.id, old.content);
            END;

            CREATE TRIGGER IF NOT EXISTS records_au AFTER UPDATE ON records BEGIN
                INSERT INTO records_fts(records_fts, rowid, content)
                VALUES ('delete', old.id, old.content);
                INSERT INTO records_fts(rowid, content) VALUES (new.id, new.content);
            END;

            -- Learning data persistence tables
            CREATE TABLE IF NOT EXISTS silent_pattern_stats (
                date TEXT NOT NULL,
                hour INTEGER NOT NULL,
                silent_captures INTEGER NOT NULL DEFAULT 0,
                change_captures INTEGER NOT NULL DEFAULT 0,
                PRIMARY KEY (date, hour)
            );

            CREATE TABLE IF NOT EXISTS work_time_activity (
                date TEXT NOT NULL,
                hour INTEGER NOT NULL,
                capture_count INTEGER NOT NULL DEFAULT 1,
                PRIMARY KEY (date, hour)
            );
        "#,
    }]
}

/// Initialize the schema version tracking tables
pub fn init_schema_version_table(conn: &Connection) -> Result<(), String> {
    // Create schema_version table if not exists
    conn.execute(
        "CREATE TABLE IF NOT EXISTS schema_version (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            version INTEGER NOT NULL DEFAULT 0,
            updated_at INTEGER NOT NULL
        )",
        [],
    )
    .map_err(|e| format!("Failed to create schema_version table: {}", e))?;

    // Create schema_migrations history table if not exists
    conn.execute(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            version INTEGER NOT NULL UNIQUE,
            description TEXT NOT NULL,
            applied_at INTEGER NOT NULL
        )",
        [],
    )
    .map_err(|e| format!("Failed to create schema_migrations table: {}", e))?;

    // Insert initial version row if not exists
    conn.execute(
        "INSERT OR IGNORE INTO schema_version (id, version, updated_at) VALUES (1, 0, 0)",
        [],
    )
    .map_err(|e| format!("Failed to insert initial schema version: {}", e))?;

    Ok(())
}

/// Get current database schema version
pub fn get_current_version(conn: &Connection) -> Result<i32, String> {
    conn.query_row(
        "SELECT version FROM schema_version WHERE id = 1",
        [],
        |row| row.get(0),
    )
    .map_err(|e| format!("Failed to get current schema version: {}", e))
}

/// Run all pending migrations
pub fn run_migrations(conn: &Connection) -> Result<(), String> {
    let current_version = get_current_version(conn)?;
    let migrations = get_migrations();

    if current_version >= CURRENT_SCHEMA_VERSION {
        tracing::info!("Database schema is up to date (v{})", current_version);
        return Ok(());
    }

    tracing::info!(
        "Current schema version: {}, running {} pending migration(s)",
        current_version,
        CURRENT_SCHEMA_VERSION - current_version
    );

    for migration in migrations {
        if migration.version <= current_version {
            continue;
        }

        migration.execute(conn)?;
    }

    Ok(())
}

/// Get migration history
pub fn get_migration_history(conn: &Connection) -> Result<Vec<(i32, String, i64)>, String> {
    let mut stmt = conn
        .prepare("SELECT version, description, applied_at FROM schema_migrations ORDER BY version")
        .map_err(|e| format!("Failed to prepare statement: {}", e))?;

    let rows = stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
        .map_err(|e| format!("Failed to query migrations: {}", e))?;

    let mut history = Vec::new();
    for row in rows {
        history.push(row.map_err(|e| format!("Failed to read row: {}", e))?);
    }

    Ok(history)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migration_version_constants() {
        assert!(CURRENT_SCHEMA_VERSION >= 1);
    }

    #[test]
    fn test_migrations_are_ordered() {
        let migrations = get_migrations();
        for (i, m) in migrations.iter().enumerate() {
            assert_eq!(m.version as usize, i + 1);
        }
    }

    #[test]
    fn test_init_schema_version_table() {
        use rusqlite::Connection;
        use tempfile::NamedTempFile;

        let temp_file = NamedTempFile::new().unwrap();
        let conn = Connection::open(temp_file.path()).unwrap();

        init_schema_version_table(&conn).unwrap();

        let version: i32 = conn
            .query_row(
                "SELECT version FROM schema_version WHERE id = 1",
                [],
                |row| row.get(0),
            )
            .unwrap();

        assert_eq!(version, 0);
    }

    #[test]
    fn test_run_migrations() {
        use rusqlite::Connection;
        use tempfile::NamedTempFile;

        let temp_file = NamedTempFile::new().unwrap();
        let conn = Connection::open(temp_file.path()).unwrap();

        // Initialize version table first
        init_schema_version_table(&conn).unwrap();

        // Run migrations
        run_migrations(&conn).unwrap();

        // Verify version was updated
        let version = get_current_version(&conn).unwrap();
        assert_eq!(version, CURRENT_SCHEMA_VERSION);

        // Verify migrations were recorded
        let history = get_migration_history(&conn).unwrap();
        assert!(!history.is_empty());
    }

    #[test]
    fn test_run_migrations_is_idempotent() {
        use rusqlite::Connection;
        use tempfile::NamedTempFile;

        let temp_file = NamedTempFile::new().unwrap();
        let conn = Connection::open(temp_file.path()).unwrap();

        // Initialize version table first
        init_schema_version_table(&conn).unwrap();

        // Run migrations twice
        run_migrations(&conn).unwrap();
        let v1 = get_current_version(&conn).unwrap();

        run_migrations(&conn).unwrap();
        let v2 = get_current_version(&conn).unwrap();

        assert_eq!(v1, v2);
        assert_eq!(v2, CURRENT_SCHEMA_VERSION);

        // Should only have one migration recorded
        let history = get_migration_history(&conn).unwrap();
        assert_eq!(history.len(), 1);
    }
}
