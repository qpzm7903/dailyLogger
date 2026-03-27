//! Logging setup and crash handling
//!
//! This module handles:
//! - Logging initialization with daily rotation
//! - Crash log file writing
//! - Diagnostic file writing for startup issues

use std::fs::OpenOptions;
use std::io::Write;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

use daily_logger_lib::get_app_data_dir;

/// Build tooltip text for tray icon showing status and today's record count.
#[cfg(feature = "screenshot")]
pub fn build_tray_tooltip() -> String {
    use daily_logger_lib::auto_perception::is_auto_capture_running;
    use daily_logger_lib::memory_storage::get_today_record_count_sync;

    let status = if is_auto_capture_running() {
        "捕获中"
    } else {
        "已暂停"
    };

    let record_count = get_today_record_count_sync().unwrap_or(0);

    format!(
        "DailyLogger\n状态: {}\n今日记录: {} 条",
        status, record_count
    )
}

#[cfg(not(feature = "screenshot"))]
pub fn build_tray_tooltip() -> String {
    use daily_logger_lib::memory_storage::get_today_record_count_sync;

    let record_count = get_today_record_count_sync().unwrap_or(0);
    format!("DailyLogger\n今日记录: {} 条", record_count)
}

/// Get crash log file path. For portable mode, write next to the exe.
/// For installed mode, write to the app data directory.
pub fn get_crash_log_path() -> std::path::PathBuf {
    // First try to write next to the executable (portable mode)
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let crash_log = exe_dir.join("dailylogger-crash.log");
            // Test if we can write to this location
            if OpenOptions::new()
                .create(true)
                .append(true)
                .open(&crash_log)
                .is_ok()
            {
                return crash_log;
            }
        }
    }

    // Fall back to app data directory
    get_app_data_dir().join("crash.log")
}

/// Write crash information to a persistent file.
/// This is called from the panic hook to ensure crash info is captured
/// even when stderr is not visible (Windows GUI mode).
pub fn write_crash_log(panic_info: &std::panic::PanicHookInfo) {
    write_crash_message(&panic_info.to_string());
}

/// Write a crash message to the crash log file.
pub fn write_crash_message(message: &str) {
    // Use Utc instead of Local to avoid timezone lookup issues on Windows
    let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.3f UTC");
    let crash_message = format!(
        "[{}] FATAL ERROR\n{}\nVersion: {}\nPlatform: {}\n\n",
        timestamp,
        message,
        env!("CARGO_PKG_VERSION"),
        std::env::consts::OS
    );

    let crash_log_path = get_crash_log_path();

    // Try to write to crash log file
    match OpenOptions::new()
        .create(true)
        .append(true)
        .open(&crash_log_path)
    {
        Ok(mut file) => {
            let write_result = file.write_all(crash_message.as_bytes());
            let flush_result = file.flush();
            if let Err(e) = write_result {
                eprintln!("Failed to write crash log: {}", e);
            }
            if let Err(e) = flush_result {
                eprintln!("Failed to flush crash log: {}", e);
            }
        }
        Err(e) => {
            eprintln!("Failed to open crash log file {:?}: {}", crash_log_path, e);
        }
    }

    // Also print to stderr (may be invisible on Windows GUI mode)
    eprintln!("{}", crash_message);
}

/// Check if WebView2 is installed on Windows.
/// Returns true if WebView2 is available, false otherwise.
#[cfg(target_os = "windows")]
pub fn is_webview2_installed() -> bool {
    use std::process::Command;

    // Check for WebView2 via registry or by trying to find the DLL
    // Method 1: Check registry for installed WebView2
    if let Ok(output) = Command::new("reg")
        .args([
            "query",
            "HKLM\\SOFTWARE\\WOW6432Node\\Microsoft\\EdgeUpdate\\Clients\\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}",
            "/v",
            "pv",
        ])
        .output()
    {
        if output.status.success() {
            return true;
        }
    }

    // Method 2: Check user-specific registry
    if let Ok(output) = Command::new("reg")
        .args([
            "query",
            "HKCU\\SOFTWARE\\Microsoft\\EdgeUpdate\\Clients\\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}",
            "/v",
            "pv",
        ])
        .output()
    {
        if output.status.success() {
            return true;
        }
    }

    // Method 3: Check for the fixed version DLL path
    let webview2_paths = [
        // Fixed version paths
        std::env::var("WEBVIEW2_BROWSER_EXECUTABLE_FOLDER").unwrap_or_default(),
        // Common installation paths
        format!(
            "{}\\Microsoft\\EdgeWebView\\Application",
            std::env::var("ProgramFiles(x86)")
                .unwrap_or_else(|_| "C:\\Program Files (x86)".to_string())
        ),
        format!(
            "{}\\Microsoft\\EdgeWebView\\Application",
            std::env::var("ProgramFiles").unwrap_or_else(|_| "C:\\Program Files".to_string())
        ),
    ];

    for path in &webview2_paths {
        if !path.is_empty() && std::path::Path::new(path).exists() {
            return true;
        }
    }

    false
}

#[cfg(not(target_os = "windows"))]
#[allow(dead_code)]
pub fn is_webview2_installed() -> bool {
    true // WebView2 is only required on Windows
}

/// Show a Windows message box with WebView2 error message.
#[cfg(target_os = "windows")]
pub fn show_webview2_error_message(message: &str) {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use windows::Win32::Foundation::HWND;
    use windows::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_ICONERROR};

    let title_wide: Vec<u16> = OsStr::new("DailyLogger - Missing Dependency")
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    let message_wide: Vec<u16> = OsStr::new(message)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    unsafe {
        let _ = MessageBoxW(
            HWND(std::ptr::null_mut()),
            windows::core::PCWSTR(message_wide.as_ptr()),
            windows::core::PCWSTR(title_wide.as_ptr()),
            MB_ICONERROR,
        );
    }
}

#[cfg(not(target_os = "windows"))]
#[allow(dead_code)]
pub fn show_webview2_error_message(_message: &str) {
    // No-op on non-Windows platforms
}

/// Setup logging with daily rotation and file appender
pub fn setup_logging() -> Option<WorkerGuard> {
    let log_dir = get_app_data_dir().join("logs");
    if let Err(e) = std::fs::create_dir_all(&log_dir) {
        eprintln!("Warning: cannot create log directory {:?}: {}", log_dir, e);
    }

    // Daily rotation: creates files like daily-logger.2026-03-16.log
    // Keeps at most 7 days of logs, older files are automatically deleted.
    match RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .filename_prefix("daily-logger")
        .filename_suffix("log")
        .max_log_files(7)
        .build(&log_dir)
    {
        Ok(file_appender) => {
            let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

            tracing_subscriber::registry()
                .with(EnvFilter::new("info"))
                .with(fmt::layer().with_writer(non_blocking))
                .with(fmt::layer().with_writer(std::io::stdout))
                .init();

            Some(guard)
        }
        Err(e) => {
            // Fall back to stdout-only logging if file appender fails
            eprintln!(
                "Warning: cannot create log file appender in {:?}: {}. Logging to stdout only.",
                log_dir, e
            );

            tracing_subscriber::registry()
                .with(EnvFilter::new("info"))
                .with(fmt::layer().with_writer(std::io::stdout))
                .init();

            None
        }
    }
}

/// Write diagnostic information to a file for troubleshooting startup issues
pub fn write_diagnostic_file(message: &str) {
    // Use Utc instead of Local to avoid timezone lookup issues on Windows
    let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.3f UTC");
    let diagnostic_message = format!("[{}] {}\n", timestamp, message);

    // Always try temp directory first as it's most reliable
    let temp_path = std::env::temp_dir().join("dailylogger-startup.log");
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&temp_path)
    {
        let _ = file.write_all(diagnostic_message.as_bytes());
        let _ = file.flush();
    }

    // Get executable path once to avoid repeated calls
    let exe_path = std::env::current_exe().ok();
    let exe_dir = exe_path.as_ref().and_then(|p| p.parent());

    // Try multiple locations in order of preference
    let locations: Vec<std::path::PathBuf> = vec![
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
    ];

    for location in &locations {
        if location.as_os_str().is_empty() {
            continue;
        }
        if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(location) {
            let _ = file.write_all(diagnostic_message.as_bytes());
            let _ = file.flush();
        }
    }
    // Last resort: try to print to stderr (may be invisible on Windows GUI mode)
    eprintln!("{}", diagnostic_message);
}
