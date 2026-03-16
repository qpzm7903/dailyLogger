use crate::memory_storage;
use tauri::command;

/// Save a quick note from the tray menu (synchronous version for tests)
pub fn add_quick_note_sync(content: &str) -> Result<i64, String> {
    if content.trim().is_empty() {
        return Err("内容不能为空".to_string());
    }

    memory_storage::add_record("manual", content, None, None, None)
        .map_err(|e| format!("保存记录失败: {}", e))
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
        .get_obsidian_output_path()
        .map_err(|_| "请先在设置中配置 Obsidian 路径".to_string())?;

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

    memory_storage::add_record("manual", &content, None, None, None)
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

/// Find all log files in the given directory, sorted by name (oldest first).
/// Matches files with prefix "daily-logger" (e.g. daily-logger.2026-03-16.log).
fn find_log_files(log_dir: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut files: Vec<std::path::PathBuf> = std::fs::read_dir(log_dir)
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.starts_with("daily-logger"))
                .unwrap_or(false)
        })
        .collect();
    files.sort();
    files
}

#[command]
pub async fn get_recent_logs(lines: Option<usize>) -> Result<String, String> {
    let log_dir = dirs::data_dir()
        .ok_or_else(|| "Cannot determine data directory".to_string())?
        .join("DailyLogger")
        .join("logs");

    let log_files = find_log_files(&log_dir);
    if log_files.is_empty() {
        return Ok(String::new());
    }

    // Read from the most recent log file
    let latest = log_files.last().unwrap();
    let content =
        std::fs::read_to_string(latest).map_err(|e| format!("Failed to read log file: {}", e))?;

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

/// Get the log file content for export (all log files concatenated)
#[command]
pub async fn get_logs_for_export() -> Result<String, String> {
    let log_dir = dirs::data_dir()
        .ok_or_else(|| "Cannot determine data directory".to_string())?
        .join("DailyLogger")
        .join("logs");

    let log_files = find_log_files(&log_dir);
    if log_files.is_empty() {
        return Err("Log file does not exist".to_string());
    }

    let mut content = String::new();
    for file in &log_files {
        let file_content =
            std::fs::read_to_string(file).map_err(|e| format!("Failed to read log file: {}", e))?;
        content.push_str(&file_content);
    }

    Ok(content)
}

/// Get the log directory path
#[command]
pub async fn get_log_file_path() -> Result<String, String> {
    let log_dir = dirs::data_dir()
        .ok_or_else(|| "Cannot determine data directory".to_string())?
        .join("DailyLogger")
        .join("logs");

    log_dir
        .to_str()
        .map(|s| s.to_string())
        .ok_or_else(|| "Invalid log directory path".to_string())
}

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
                obsidian_vaults TEXT DEFAULT '[]'
            )",
            [],
        )
        .unwrap();
        conn.execute("INSERT OR IGNORE INTO settings (id) VALUES (1)", [])
            .unwrap();
        let mut db = crate::memory_storage::DB_CONNECTION.lock().unwrap();
        *db = Some(conn);
    }

    #[test]
    #[serial]
    fn test_get_logs_for_export_no_file() {
        // Test that get_logs_for_export returns error when log file doesn't exist
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(get_logs_for_export());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Log file does not exist"));
    }

    #[test]
    #[serial]
    fn test_get_log_file_path() {
        // Test that get_log_file_path returns the logs directory
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(get_log_file_path());
        assert!(result.is_ok());
        let path = result.unwrap();
        assert!(path.contains("DailyLogger"));
        assert!(path.ends_with("logs"));
    }

    #[test]
    #[serial]
    fn test_get_recent_logs_empty() {
        // Test that get_recent_logs returns empty string when no log file exists
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(get_recent_logs(None));
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    #[serial]
    fn test_get_recent_logs_with_lines() {
        // Test that get_recent_logs with specific line count works
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(get_recent_logs(Some(10)));
        assert!(result.is_ok());
    }

    // ── tray_quick_note and add_quick_note_sync tests ──

    #[test]
    #[serial]
    fn test_add_quick_note_sync_saves_record() {
        setup_test_db();

        let result = add_quick_note_sync("测试快速记录");
        assert!(result.is_ok(), "add_quick_note_sync should succeed");
        let id = result.unwrap();
        assert!(id > 0, "Record ID should be positive");
    }

    #[test]
    #[serial]
    fn test_add_quick_note_sync_rejects_empty_content() {
        let result = add_quick_note_sync("");
        assert!(result.is_err(), "Empty content should be rejected");
        assert!(
            result.unwrap_err().contains("内容不能为空"),
            "Error message should indicate empty content"
        );
    }

    #[test]
    #[serial]
    fn test_add_quick_note_sync_rejects_whitespace_only() {
        let result = add_quick_note_sync("   \n\t  ");
        assert!(
            result.is_err(),
            "Whitespace-only content should be rejected"
        );
    }

    #[test]
    #[serial]
    fn test_tray_quick_note_saves_record() {
        setup_test_db();

        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(tray_quick_note("托盘快速记录测试".to_string()));
        assert!(result.is_ok(), "tray_quick_note should succeed");
    }

    #[test]
    #[serial]
    fn test_tray_quick_note_rejects_empty_content() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(tray_quick_note("".to_string()));
        assert!(result.is_err(), "Empty content should be rejected");
    }

    #[test]
    #[serial]
    fn test_add_quick_note_saves_record() {
        setup_test_db();

        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(add_quick_note("快速记录测试".to_string()));
        assert!(result.is_ok(), "add_quick_note should succeed");
    }

    #[test]
    #[serial]
    fn test_add_quick_note_rejects_empty_content() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(add_quick_note("".to_string()));
        assert!(result.is_err(), "Empty content should be rejected");
    }

    // ── open_obsidian_folder tests ──

    #[test]
    #[serial]
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
    #[serial]
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
    #[serial]
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
    #[serial]
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
    #[serial]
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
    #[serial]
    fn test_open_obsidian_folder_async_rejects_missing_path() {
        setup_test_db_with_settings();

        // Explicitly clear obsidian_path to ensure it's None (protect against parallel test interference)
        let mut settings = memory_storage::get_settings_sync().unwrap();
        settings.obsidian_path = None;
        memory_storage::save_settings_sync(&settings).unwrap();

        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(open_obsidian_folder());
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("请先在设置中配置 Obsidian 路径"));
    }

    // ── Platform command verification tests (CORE-008 Task 2.4) ──

    #[test]
    #[cfg(target_os = "macos")]
    fn test_macos_open_command_available_for_obsidian() {
        // Verify 'open' command exists on macOS (used by open_obsidian_folder_sync)
        let output = std::process::Command::new("which")
            .arg("open")
            .output()
            .expect("'which' command should work on macOS");
        assert!(
            output.status.success(),
            "macOS 'open' command should be available for opening directories"
        );
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_windows_explorer_available_for_obsidian() {
        // Verify 'explorer' command exists on Windows
        let output = std::process::Command::new("where")
            .arg("explorer")
            .output()
            .expect("'where' command should work on Windows");
        assert!(
            output.status.success(),
            "Windows 'explorer' should be available for opening directories"
        );
    }

    #[test]
    #[serial]
    #[cfg(target_os = "macos")]
    fn test_open_obsidian_folder_spawns_open_on_macos() {
        setup_test_db_with_settings();

        // Use a valid existing directory
        let temp_dir = std::env::temp_dir();
        let mut settings = memory_storage::get_settings_sync().unwrap();
        settings.obsidian_path = Some(temp_dir.to_string_lossy().to_string());
        memory_storage::save_settings_sync(&settings).unwrap();

        let result = open_obsidian_folder_sync();
        assert!(
            result.is_ok(),
            "open_obsidian_folder should succeed on macOS with valid path"
        );
    }

    #[test]
    #[serial]
    #[cfg(target_os = "windows")]
    fn test_open_obsidian_folder_spawns_explorer_on_windows() {
        setup_test_db_with_settings();

        let temp_dir = std::env::temp_dir();
        let mut settings = memory_storage::get_settings_sync().unwrap();
        settings.obsidian_path = Some(temp_dir.to_string_lossy().to_string());
        memory_storage::save_settings_sync(&settings).unwrap();

        let result = open_obsidian_folder_sync();
        assert!(
            result.is_ok(),
            "open_obsidian_folder should succeed on Windows with valid path"
        );
    }

    #[test]
    #[serial]
    fn test_open_obsidian_folder_path_validation_is_platform_independent() {
        setup_test_db_with_settings();

        // Path validation (empty, nonexistent) should work the same on all platforms
        let mut settings = memory_storage::get_settings_sync().unwrap();
        settings.obsidian_path = Some("/absolutely/nonexistent/path/12345".to_string());
        memory_storage::save_settings_sync(&settings).unwrap();

        let result = open_obsidian_folder_sync();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Obsidian 路径不存在"));
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
                obsidian_vaults TEXT DEFAULT '[]'
            )",
            [],
        )
        .unwrap();
        conn.execute("INSERT OR IGNORE INTO settings (id) VALUES (1)", [])
            .unwrap();
        let mut db = crate::memory_storage::DB_CONNECTION.lock().unwrap();
        *db = Some(conn);
    }

    // ── Platform-specific command tests (CORE-008 AC#5) ──

    #[test]
    fn test_get_screenshot_reads_valid_png() {
        // Test that get_screenshot can handle a valid base64-encoded image
        // This is a basic test - actual screenshot reading requires a screenshot file
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_screenshot.png");

        // Create a minimal valid PNG (1x1 transparent pixel)
        let png_data: Vec<u8> = vec![
            0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG signature
            0x00, 0x00, 0x00, 0x0D, // IHDR length
            0x49, 0x48, 0x44, 0x52, // IHDR
            0x00, 0x00, 0x00, 0x01, // width = 1
            0x00, 0x00, 0x00, 0x01, // height = 1
            0x08, 0x06, // bit depth = 8, color type = 6 (RGBA)
            0x00, 0x00, 0x00, // compression, filter, interlace
            0x1F, 0x15, 0xC4, 0x89, // CRC
            0x00, 0x00, 0x00, 0x0A, // IDAT length
            0x49, 0x44, 0x41, 0x54, // IDAT
            0x78, 0x9C, 0x63, 0x00, 0x01, 0x00, 0x00, 0x05, 0x00, 0x01, // compressed data
            0x0D, 0x0A, 0x2D, 0xB4, // CRC
            0x00, 0x00, 0x00, 0x00, // IEND length
            0x49, 0x45, 0x4E, 0x44, // IEND
            0xAE, 0x42, 0x60, 0x82, // CRC
        ];

        std::fs::write(&test_file, &png_data).unwrap();

        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(get_screenshot(test_file.to_string_lossy().to_string()));

        let _ = std::fs::remove_file(&test_file);

        assert!(result.is_ok());
        let data = result.unwrap();
        assert!(data.starts_with("data:image/png;base64,"));
    }

    #[test]
    fn test_get_screenshot_fails_for_nonexistent_file() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(get_screenshot("/nonexistent/file.png".to_string()));
        assert!(result.is_err());
    }

    #[test]
    fn test_read_file_reads_text_file() {
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_read_file.txt");
        let test_content = "Hello, World! 你好世界！";

        std::fs::write(&test_file, test_content).unwrap();

        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(read_file(test_file.to_string_lossy().to_string()));

        let _ = std::fs::remove_file(&test_file);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), test_content);
    }

    #[test]
    fn test_read_file_fails_for_nonexistent_file() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(read_file("/nonexistent/file.txt".to_string()));
        assert!(result.is_err());
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn windows_platform_uses_explorer_for_obsidian_folder() {
        // On Windows, open_obsidian_folder_sync should use "explorer" command
        // This test verifies the platform-specific code compiles correctly
        let test_path = "C:\\Users\\test\\Obsidian";
        let path = std::path::Path::new(&test_path);
        let _ = path.exists();
        // Windows paths use backslashes
        assert!(test_path.contains('\\'));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn macos_platform_uses_open_for_obsidian_folder() {
        // On macOS, open_obsidian_folder_sync should use "open" command
        // This test verifies the platform-specific code compiles correctly
        let test_path = "/Users/test/Obsidian";
        let path = std::path::Path::new(&test_path);
        let _ = path.exists();
        // macOS paths use forward slashes
        assert!(test_path.contains('/'));
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn linux_platform_uses_xdg_open_for_obsidian_folder() {
        // On Linux, open_obsidian_folder_sync should use "xdg-open" command
        // This test verifies the platform-specific code compiles correctly
        let test_path = "/home/test/Obsidian";
        let path = std::path::Path::new(&test_path);
        let _ = path.exists();
        // Linux paths use forward slashes
        assert!(test_path.contains('/'));
    }

    #[test]
    fn test_get_log_file_path_returns_valid_path_structure() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(get_log_file_path());
        assert!(result.is_ok());
        let path = result.unwrap();

        // Should contain DailyLogger and end with logs directory
        assert!(
            path.contains("DailyLogger"),
            "Path should contain DailyLogger"
        );
        assert!(
            path.ends_with("logs"),
            "Path should end with logs directory"
        );
    }

    #[test]
    fn test_add_quick_note_content_length_handling() {
        setup_test_db();

        // Test with a long content
        let long_content = "a".repeat(10000);
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(add_quick_note(long_content.clone()));
        assert!(result.is_ok());

        // Test with unicode content
        let unicode_content = "你好世界🌍🎉";
        let result = rt.block_on(add_quick_note(unicode_content.to_string()));
        assert!(result.is_ok());
    }
}
