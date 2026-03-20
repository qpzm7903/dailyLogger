pub mod auth;
#[cfg(feature = "screenshot")]
pub mod auto_perception;
pub mod backup;
pub mod crypto;
pub mod export;
pub mod fine_tuning;
pub mod github;
#[cfg(feature = "screenshot")]
pub mod hardware;
pub mod manual_entry;
pub mod memory_storage;
#[cfg(feature = "screenshot")]
pub mod monitor;
pub mod monitor_types;
pub mod network_status;
pub mod notion;
pub mod offline_queue;
pub mod ollama;
pub mod performance;
pub mod plugin;
pub mod silent_tracker;
pub mod slack;
pub mod synthesis;
pub mod team;
pub mod timeline;
pub mod window_info;
pub mod work_time;

use once_cell::sync::Lazy;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;

pub static APP_STATE: Lazy<Mutex<AppState>> = Lazy::new(|| Mutex::new(AppState::default()));

/// Returns the application data directory: `<system_data_dir>/DailyLogger`.
/// Used by all modules that need access to the app's persistent data.
pub fn get_app_data_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("DailyLogger")
}

/// Write a diagnostic message to a startup log file for debugging Windows portable issues.
/// Tries multiple locations in order of preference:
/// 1. Next to executable (portable mode)
/// 2. App data directory
/// 3. User home directory as fallback
/// 4. Temp directory as last resort
pub fn write_diagnostic_file(message: &str) {
    // Use Utc instead of Local to avoid timezone lookup issues on Windows
    let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.3f UTC");
    let diagnostic_message = format!("[{}] {}\n", timestamp, message);

    // Get executable path once to avoid repeated calls
    let exe_path = std::env::current_exe().ok();
    let exe_dir = exe_path.as_ref().and_then(|p| p.parent());

    // Try multiple locations in order of preference
    let locations: Vec<PathBuf> = vec![
        // 1. Next to executable (portable mode)
        exe_dir
            .map(|d| d.join("dailylogger-startup.log"))
            .unwrap_or_default(),
        // 2. App data directory
        get_app_data_dir().join("startup.log"),
        // 3. User home directory as fallback
        dirs::home_dir()
            .map(|h| h.join("dailylogger-startup.log"))
            .unwrap_or_default(),
        // 4. Temp directory as last resort
        std::env::temp_dir().join("dailylogger-startup.log"),
    ];

    let mut last_error = String::new();
    for location in &locations {
        if location.as_os_str().is_empty() {
            continue;
        }
        match OpenOptions::new().create(true).append(true).open(location) {
            Ok(mut file) => {
                match file.write_all(diagnostic_message.as_bytes()) {
                    Ok(()) => {
                        // Flush to ensure data is written to disk
                        if file.flush().is_ok() {
                            return; // Successfully wrote
                        }
                    }
                    Err(e) => {
                        last_error = format!("Write failed to {:?}: {}", location, e);
                    }
                }
            }
            Err(e) => {
                last_error = format!("Open failed for {:?}: {}", location, e);
            }
        }
    }
    // Last resort: try to print to stderr (may be invisible on Windows GUI mode)
    eprintln!("{}", diagnostic_message);
    if !last_error.is_empty() {
        eprintln!("Diagnostic write error: {}", last_error);
    }
}

#[derive(Default)]
pub struct AppState {
    pub auto_capture_running: bool,
}

/// Mask an API key for safe logging: show prefix (up to 5 chars) + "..." + "****".
/// Example: "sk-abc123xyz9999" -> "sk-ab...****"
pub fn mask_api_key(key: &str) -> String {
    if key.is_empty() {
        return "****".to_string();
    }

    // Check if it's an encrypted key - don't reveal any part of it
    if crate::crypto::is_encrypted(key) {
        return "[encrypted]".to_string();
    }

    // Show prefix (first 5 chars or less) + "..." + "****"
    let prefix_len = key.len().min(5);
    format!("{}...****", &key[..prefix_len])
}

pub fn init_app() -> tauri::Result<()> {
    write_diagnostic_file("init_app: Starting database initialization");
    tracing::info!("init_app: Starting database initialization");

    // Log the app data directory for debugging
    let app_data_dir = get_app_data_dir();
    write_diagnostic_file(&format!("init_app: App data directory: {:?}", app_data_dir));
    tracing::info!("init_app: App data directory: {:?}", app_data_dir);

    write_diagnostic_file("init_app: Calling init_database()");
    memory_storage::init_database().map_err(|e| {
        write_diagnostic_file(&format!("init_app: init_database FAILED: {}", e));
        tracing::error!("init_app: Database initialization failed: {}", e);
        tauri::Error::Anyhow(anyhow::anyhow!("{}", e))
    })?;

    write_diagnostic_file("init_app: Database initialized successfully");
    tracing::info!("init_app: Database initialized successfully");

    // Load persisted learning data (DEBT-005)
    write_diagnostic_file("init_app: Loading silent pattern stats");
    if let Err(e) = silent_tracker::load_silent_pattern_stats() {
        write_diagnostic_file(&format!(
            "init_app: Failed to load silent pattern stats: {}",
            e
        ));
        tracing::warn!("Failed to load silent pattern stats: {}", e);
    }
    write_diagnostic_file("init_app: Silent pattern stats loaded");
    tracing::info!("init_app: Silent pattern stats loaded");

    write_diagnostic_file("init_app: Loading work time activity");
    if let Err(e) = work_time::load_work_time_activity() {
        write_diagnostic_file(&format!(
            "init_app: Failed to load work time activity: {}",
            e
        ));
        tracing::warn!("Failed to load work time activity: {}", e);
    }
    write_diagnostic_file("init_app: Work time activity loaded");
    tracing::info!("init_app: Work time activity loaded");

    write_diagnostic_file("init_app: All initialization complete");
    tracing::info!("DailyLogger initialized successfully");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mask_api_key_shows_prefix() {
        assert_eq!(mask_api_key("sk-abc123xyz9999"), "sk-ab...****");
    }

    #[test]
    fn mask_api_key_short_key_shows_available_prefix() {
        assert_eq!(mask_api_key("ab"), "ab...****");
        assert_eq!(mask_api_key("abcd"), "abcd...****");
    }

    #[test]
    fn mask_api_key_empty_string() {
        assert_eq!(mask_api_key(""), "****");
    }

    #[test]
    fn mask_api_key_exactly_five_chars() {
        assert_eq!(mask_api_key("12345"), "12345...****");
    }

    #[test]
    fn mask_api_key_encrypted_key() {
        assert_eq!(mask_api_key("ENC:somebase64data"), "[encrypted]");
    }

    #[test]
    fn get_app_data_dir_returns_dailylogger_subdir() {
        let dir = get_app_data_dir();
        assert!(dir.ends_with("DailyLogger"));
    }
}
