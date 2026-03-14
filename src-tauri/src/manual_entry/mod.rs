use crate::memory_storage;
use tauri::command;

/// Save a quick note from the tray menu (synchronous version for tests)
pub fn add_quick_note_sync(content: &str) -> Result<i64, String> {
    if content.trim().is_empty() {
        return Err("内容不能为空".to_string());
    }

    memory_storage::add_record("manual", content, None).map_err(|e| format!("保存记录失败: {}", e))
}

/// Save a quick note from the tray menu.
/// This is called from the tray quick note window.
#[command]
pub async fn tray_quick_note(content: String) -> Result<(), String> {
    add_quick_note_sync(&content)?;
    tracing::info!(
        "Tray quick note added: {}...",
        &content[..content.len().min(50)]
    );
    Ok(())
}

/// Opens the Obsidian folder in the system file manager.
/// Returns an error message if the path is not configured or invalid.
pub fn open_obsidian_folder_sync() -> Result<(), String> {
    use crate::memory_storage::get_settings_sync;
    use std::path::Path;

    let settings = get_settings_sync().map_err(|e| format!("获取设置失败: {}", e))?;

    let path_str = settings
        .obsidian_path
        .filter(|p| !p.trim().is_empty())
        .ok_or_else(|| "请先在设置中配置 Obsidian 路径".to_string())?;

    let path = Path::new(&path_str);
    if !path.exists() {
        return Err(format!("Obsidian 路径不存在: {}", path_str));
    }

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(&path_str)
            .spawn()
            .map_err(|e| format!("无法打开文件夹: {}", e))?;
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&path_str)
            .spawn()
            .map_err(|e| format!("无法打开文件夹: {}", e))?;
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&path_str)
            .spawn()
            .map_err(|e| format!("无法打开文件夹: {}", e))?;
    }

    tracing::info!("Opened Obsidian folder: {}", path_str);
    Ok(())
}

#[command]
pub async fn open_obsidian_folder() -> Result<(), String> {
    open_obsidian_folder_sync()
}

#[command]
pub async fn add_quick_note(content: String) -> Result<(), String> {
    if content.trim().is_empty() {
        return Err("Content cannot be empty".to_string());
    }

    memory_storage::add_record("manual", &content, None)
        .map_err(|e| format!("Failed to save note: {}", e))?;

    tracing::info!("Quick note added: {}...", &content[..content.len().min(50)]);
    Ok(())
}

#[command]
pub async fn get_screenshot(path: String) -> Result<String, String> {
    let image_data =
        std::fs::read(&path).map_err(|e| format!("Failed to read screenshot: {}", e))?;

    let base64_data =
        base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &image_data);
    Ok(format!("data:image/png;base64,{}", base64_data))
}

#[command]
pub async fn read_file(path: String) -> Result<String, String> {
    std::fs::read_to_string(&path).map_err(|e| format!("Failed to read file: {}", e))
}

#[command]
pub async fn get_recent_logs(lines: Option<usize>) -> Result<String, String> {
    let log_path = dirs::data_dir()
        .ok_or_else(|| "Cannot determine data directory".to_string())?
        .join("DailyLogger")
        .join("logs")
        .join("daily-logger.log");

    if !log_path.exists() {
        return Ok(String::new());
    }

    let content = std::fs::read_to_string(&log_path)
        .map_err(|e| format!("Failed to read log file: {}", e))?;

    let n = lines.unwrap_or(300);
    let recent: Vec<&str> = content
        .lines()
        .rev()
        .take(n)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();

    Ok(recent.join("\n"))
}

/// Get the log file content for export
#[command]
pub async fn get_logs_for_export() -> Result<String, String> {
    let log_path = dirs::data_dir()
        .ok_or_else(|| "Cannot determine data directory".to_string())?
        .join("DailyLogger")
        .join("logs")
        .join("daily-logger.log");

    if !log_path.exists() {
        return Err("Log file does not exist".to_string());
    }

    std::fs::read_to_string(&log_path).map_err(|e| format!("Failed to read log file: {}", e))
}

/// Get the log file path for export
#[command]
pub async fn get_log_file_path() -> Result<String, String> {
    let log_path = dirs::data_dir()
        .ok_or_else(|| "Cannot determine data directory".to_string())?
        .join("DailyLogger")
        .join("logs")
        .join("daily-logger.log");

    log_path
        .to_str()
        .map(|s| s.to_string())
        .ok_or_else(|| "Invalid log file path".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

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
        let mut db = crate::memory_storage::DB_CONNECTION.lock().unwrap();
        *db = Some(conn);
    }

    #[test]
    fn test_get_logs_for_export_no_file() {
        // Test that get_logs_for_export returns error when log file doesn't exist
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(get_logs_for_export());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Log file does not exist"));
    }

    #[test]
    fn test_get_log_file_path() {
        // Test that get_log_file_path returns a valid path
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(get_log_file_path());
        assert!(result.is_ok());
        let path = result.unwrap();
        assert!(path.contains("DailyLogger"));
        assert!(path.contains("logs"));
        assert!(path.ends_with("daily-logger.log"));
    }

    #[test]
    fn test_get_recent_logs_empty() {
        // Test that get_recent_logs returns empty string when no log file exists
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(get_recent_logs(None));
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_get_recent_logs_with_lines() {
        // Test that get_recent_logs with specific line count works
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(get_recent_logs(Some(10)));
        assert!(result.is_ok());
    }

    // ── tray_quick_note and add_quick_note_sync tests ──

    #[test]
    fn test_add_quick_note_sync_saves_record() {
        setup_test_db();

        let result = add_quick_note_sync("测试快速记录");
        assert!(result.is_ok(), "add_quick_note_sync should succeed");
        let id = result.unwrap();
        assert!(id > 0, "Record ID should be positive");
    }

    #[test]
    fn test_add_quick_note_sync_rejects_empty_content() {
        let result = add_quick_note_sync("");
        assert!(result.is_err(), "Empty content should be rejected");
        assert!(
            result.unwrap_err().contains("内容不能为空"),
            "Error message should indicate empty content"
        );
    }

    #[test]
    fn test_add_quick_note_sync_rejects_whitespace_only() {
        let result = add_quick_note_sync("   \n\t  ");
        assert!(
            result.is_err(),
            "Whitespace-only content should be rejected"
        );
    }

    #[test]
    fn test_tray_quick_note_saves_record() {
        setup_test_db();

        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(tray_quick_note("托盘快速记录测试".to_string()));
        assert!(result.is_ok(), "tray_quick_note should succeed");
    }

    #[test]
    fn test_tray_quick_note_rejects_empty_content() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(tray_quick_note("".to_string()));
        assert!(result.is_err(), "Empty content should be rejected");
    }

    #[test]
    fn test_add_quick_note_saves_record() {
        setup_test_db();

        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(add_quick_note("快速记录测试".to_string()));
        assert!(result.is_ok(), "add_quick_note should succeed");
    }

    #[test]
    fn test_add_quick_note_rejects_empty_content() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(add_quick_note("".to_string()));
        assert!(result.is_err(), "Empty content should be rejected");
    }

    // ── open_obsidian_folder tests ──

    #[test]
    fn test_open_obsidian_folder_sync_rejects_missing_path() {
        setup_test_db_with_settings();

        // Settings with no obsidian_path configured
        let result = open_obsidian_folder_sync();
        assert!(result.is_err(), "Missing path should be rejected");
        let err = result.unwrap_err();
        assert!(
            err.contains("请先在设置中配置 Obsidian 路径"),
            "Error should mention configuring path, got: {}",
            err
        );
    }

    #[test]
    fn test_open_obsidian_folder_sync_rejects_empty_path() {
        setup_test_db_with_settings();

        // Save settings with empty obsidian_path
        let mut settings = memory_storage::get_settings_sync().unwrap();
        settings.obsidian_path = Some("".to_string());
        memory_storage::save_settings_sync(&settings).unwrap();

        let result = open_obsidian_folder_sync();
        assert!(result.is_err(), "Empty path should be rejected");
        let err = result.unwrap_err();
        assert!(
            err.contains("请先在设置中配置 Obsidian 路径"),
            "Error should mention configuring path, got: {}",
            err
        );
    }

    #[test]
    fn test_open_obsidian_folder_sync_rejects_whitespace_path() {
        setup_test_db_with_settings();

        // Save settings with whitespace-only obsidian_path
        let mut settings = memory_storage::get_settings_sync().unwrap();
        settings.obsidian_path = Some("   \t\n  ".to_string());
        memory_storage::save_settings_sync(&settings).unwrap();

        let result = open_obsidian_folder_sync();
        assert!(result.is_err(), "Whitespace-only path should be rejected");
        let err = result.unwrap_err();
        assert!(
            err.contains("请先在设置中配置 Obsidian 路径"),
            "Error should mention configuring path, got: {}",
            err
        );
    }

    #[test]
    fn test_open_obsidian_folder_sync_rejects_nonexistent_path() {
        setup_test_db_with_settings();

        // Save settings with a path that doesn't exist
        let mut settings = memory_storage::get_settings_sync().unwrap();
        settings.obsidian_path = Some("/nonexistent/path/that/does/not/exist".to_string());
        memory_storage::save_settings_sync(&settings).unwrap();

        let result = open_obsidian_folder_sync();
        assert!(result.is_err(), "Non-existent path should be rejected");
        let err = result.unwrap_err();
        assert!(
            err.contains("Obsidian 路径不存在"),
            "Error should mention path does not exist, got: {}",
            err
        );
    }

    #[test]
    fn test_open_obsidian_folder_sync_opens_valid_path() {
        setup_test_db_with_settings();

        // Use a valid existing directory (temp dir)
        let temp_dir = std::env::temp_dir();
        let path_str = temp_dir.to_string_lossy().to_string();

        let mut settings = memory_storage::get_settings_sync().unwrap();
        settings.obsidian_path = Some(path_str.clone());
        memory_storage::save_settings_sync(&settings).unwrap();

        let result = open_obsidian_folder_sync();
        // In CI environments without a GUI, the spawn() may fail,
        // but we should still verify that the path validation passed.
        // The error, if any, should be about opening the folder, not path validation.
        if result.is_err() {
            let err = result.unwrap_err();
            // Should NOT be about path not configured or path not existing
            assert!(
                !err.contains("请先在设置中配置 Obsidian 路径"),
                "Should not fail due to missing path config, got: {}",
                err
            );
            assert!(
                !err.contains("Obsidian 路径不存在"),
                "Should not fail due to path not existing, got: {}",
                err
            );
        }
    }

    #[test]
    fn test_open_obsidian_folder_async_rejects_missing_path() {
        setup_test_db_with_settings();

        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(open_obsidian_folder());
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("请先在设置中配置 Obsidian 路径"));
    }

    // Helper function to set up test database with settings table
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
                include_manual_records INTEGER DEFAULT 1
            )",
            [],
        )
        .unwrap();
        conn.execute("INSERT OR IGNORE INTO settings (id) VALUES (1)", [])
            .unwrap();
        let mut db = crate::memory_storage::DB_CONNECTION.lock().unwrap();
        *db = Some(conn);
    }
}
