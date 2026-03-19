#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use daily_logger_lib::get_app_data_dir;
use daily_logger_lib::init_app;
use std::fs::OpenOptions;
use std::io::Write;
use tauri::{Emitter, Manager};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

#[cfg(target_os = "windows")]
use std::process::Command;

/// Build tooltip text for tray icon showing status and today's record count.
#[cfg(feature = "screenshot")]
fn build_tray_tooltip() -> String {
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
fn build_tray_tooltip() -> String {
    use daily_logger_lib::memory_storage::get_today_record_count_sync;

    let record_count = get_today_record_count_sync().unwrap_or(0);
    format!("DailyLogger\n今日记录: {} 条", record_count)
}

/// Get crash log file path. For portable mode, write next to the exe.
/// For installed mode, write to the app data directory.
fn get_crash_log_path() -> std::path::PathBuf {
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
fn write_crash_log(panic_info: &std::panic::PanicHookInfo) {
    write_crash_message(&panic_info.to_string());
}

/// Write a crash message to the crash log file.
fn write_crash_message(message: &str) {
    let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
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
            if let Err(e) = file.write_all(crash_message.as_bytes()) {
                eprintln!("Failed to write crash log: {}", e);
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
fn is_webview2_installed() -> bool {
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
fn is_webview2_installed() -> bool {
    true // WebView2 is only required on Windows
}

/// Show a Windows message box with WebView2 error message.
#[cfg(target_os = "windows")]
fn show_webview2_error_message(message: &str) {
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
fn show_webview2_error_message(_message: &str) {
    // No-op on non-Windows platforms
}

fn setup_logging() -> Option<WorkerGuard> {
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

fn write_diagnostic_file(message: &str) {
    // Try to write a diagnostic file to help debug Windows startup issues
    // This should be called early in the startup process
    let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
    let diagnostic_message = format!("[{}] {}\n", timestamp, message);

    // Try multiple locations in order of preference
    let locations: Vec<std::path::PathBuf> = vec![
        // 1. Next to executable (portable mode)
        std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.join("dailylogger-startup.log")))
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
        match OpenOptions::new().create(true).append(true).open(location) {
            Ok(mut file) => {
                if file.write_all(diagnostic_message.as_bytes()).is_ok() {
                    return; // Successfully wrote
                }
            }
            Err(_) => continue,
        }
    }
    // Last resort: try to print to stderr (may be invisible on Windows GUI mode)
    eprintln!("{}", diagnostic_message);
}

fn main() {
    // Write diagnostic file IMMEDIATELY - before anything else
    write_diagnostic_file("Application starting...");

    // Set panic hook FIRST so any panic during initialization is logged to a crash file.
    // With `panic = "abort"` in release, the hook runs before process termination.
    // This ensures crash info is captured even on Windows where stderr is invisible.
    std::panic::set_hook(Box::new(|panic_info| {
        write_diagnostic_file(&format!("PANIC: {}", panic_info));
        write_crash_log(panic_info);
    }));

    write_diagnostic_file("Panic hook installed");

    // _log_guard must live until main() returns; dropping it early stops the
    // background logging thread and discards all subsequent log messages.
    let _log_guard = setup_logging();

    write_diagnostic_file("Logging setup complete");

    // Check WebView2 availability on Windows
    #[cfg(target_os = "windows")]
    {
        write_diagnostic_file("Checking WebView2...");
        if !is_webview2_installed() {
            write_diagnostic_file("WebView2 NOT found - showing error");
            let error_msg = "DailyLogger requires Microsoft Edge WebView2 Runtime to run.\n\n\
                Please download and install WebView2 Runtime from:\n\
                https://developer.microsoft.com/en-us/microsoft-edge/webview2/\n\n\
                Direct download link:\n\
                https://go.microsoft.com/fwlink/p/?LinkId=2124703\n\n\
                Or use the installer version (*-setup.exe) which will automatically\n\
                install WebView2 for you.";

            tracing::error!("{}", error_msg);
            write_crash_message(error_msg);

            // Show a Windows message box so the user can see the error
            show_webview2_error_message(error_msg);

            std::process::exit(1);
        }
        write_diagnostic_file("WebView2 found");
    }

    write_diagnostic_file("Initializing app...");
    if let Err(e) = init_app() {
        write_diagnostic_file(&format!("Failed to initialize app: {}", e));
        tracing::error!("Failed to initialize app: {}", e);
        eprintln!("Fatal: Failed to initialize app: {}", e);
        write_crash_message(&format!("Failed to initialize app: {}", e));
        return;
    }
    write_diagnostic_file("App initialized successfully");

    write_diagnostic_file("Building Tauri application...");

    // Build base builder with cross-platform plugins
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_os::init());

    // Add desktop-only plugins (global shortcuts not supported on mobile)
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    let builder = builder.plugin(tauri_plugin_global_shortcut::Builder::new().build());

    let result = builder
        .invoke_handler(tauri::generate_handler![
            daily_logger_lib::manual_entry::add_quick_note,
            daily_logger_lib::manual_entry::tray_quick_note,
            daily_logger_lib::manual_entry::get_screenshot,
            daily_logger_lib::manual_entry::read_file,
            daily_logger_lib::manual_entry::get_recent_logs,
            daily_logger_lib::manual_entry::get_logs_for_export,
            daily_logger_lib::manual_entry::get_log_file_path,
            daily_logger_lib::manual_entry::open_obsidian_folder,
            daily_logger_lib::memory_storage::get_today_records,
            daily_logger_lib::memory_storage::get_records_by_date_range,
            daily_logger_lib::memory_storage::get_settings,
            daily_logger_lib::memory_storage::save_settings,
            daily_logger_lib::ollama::test_api_connection_with_ollama,
            daily_logger_lib::memory_storage::get_model_info,
            daily_logger_lib::memory_storage::delete_record,
            daily_logger_lib::memory_storage::get_history_records,
            daily_logger_lib::memory_storage::search_records,
            daily_logger_lib::memory_storage::get_default_tag_categories,
            daily_logger_lib::memory_storage::get_all_tags,
            daily_logger_lib::memory_storage::get_records_by_tag,
            // DATA-003: 手动标签系统
            daily_logger_lib::memory_storage::create_manual_tag,
            daily_logger_lib::memory_storage::get_all_manual_tags,
            daily_logger_lib::memory_storage::update_manual_tag,
            daily_logger_lib::memory_storage::delete_manual_tag,
            daily_logger_lib::memory_storage::add_tag_to_record,
            daily_logger_lib::memory_storage::remove_tag_from_record,
            daily_logger_lib::memory_storage::get_tags_for_record,
            daily_logger_lib::memory_storage::get_tags_for_records,
            daily_logger_lib::memory_storage::get_records_by_manual_tags,
            daily_logger_lib::synthesis::generate_daily_summary,
            daily_logger_lib::synthesis::get_default_summary_prompt,
            daily_logger_lib::synthesis::generate_weekly_report,
            daily_logger_lib::synthesis::generate_monthly_report,
            daily_logger_lib::synthesis::generate_custom_report,
            daily_logger_lib::synthesis::compare_reports,
            // DATA-004: 数据导出
            daily_logger_lib::export::export_records,
            daily_logger_lib::export::open_export_dir,
            // DATA-005: 数据备份与恢复
            daily_logger_lib::backup::create_backup,
            daily_logger_lib::backup::get_backup_info,
            daily_logger_lib::backup::list_backups,
            daily_logger_lib::backup::delete_backup,
            daily_logger_lib::backup::restore_backup,
            daily_logger_lib::ollama::get_ollama_models,
            daily_logger_lib::ollama::pull_ollama_model,
            daily_logger_lib::ollama::delete_ollama_model,
            daily_logger_lib::ollama::get_running_models,
            daily_logger_lib::ollama::create_ollama_model,
            daily_logger_lib::ollama::copy_ollama_model,
            daily_logger_lib::ollama::show_ollama_model,
            daily_logger_lib::notion::test_notion_connection,
            daily_logger_lib::github::test_github_connection,
            daily_logger_lib::slack::test_slack_connection,
            daily_logger_lib::network_status::get_network_status,
            daily_logger_lib::network_status::check_network_status,
            // CORE-008: Performance benchmark
            daily_logger_lib::performance::get_platform_info,
            daily_logger_lib::performance::get_memory_usage_mb,
            daily_logger_lib::performance::benchmark_database_query,
            daily_logger_lib::performance::run_performance_benchmark,
            #[cfg(feature = "screenshot")]
            daily_logger_lib::performance::benchmark_screenshot_processing,
            daily_logger_lib::offline_queue::get_offline_queue_status,
            daily_logger_lib::offline_queue::process_offline_queue,
            #[cfg(feature = "screenshot")]
            daily_logger_lib::auto_perception::start_auto_capture,
            #[cfg(feature = "screenshot")]
            daily_logger_lib::auto_perception::stop_auto_capture,
            #[cfg(feature = "screenshot")]
            daily_logger_lib::auto_perception::trigger_capture,
            #[cfg(feature = "screenshot")]
            daily_logger_lib::auto_perception::take_screenshot,
            #[cfg(feature = "screenshot")]
            daily_logger_lib::auto_perception::get_default_analysis_prompt,
            #[cfg(feature = "screenshot")]
            daily_logger_lib::auto_perception::get_auto_capture_status,
            #[cfg(feature = "screenshot")]
            daily_logger_lib::auto_perception::get_work_time_status,
            #[cfg(feature = "screenshot")]
            daily_logger_lib::monitor::get_monitors,
            // Timeline visualization
            daily_logger_lib::timeline::get_timeline_today,
            daily_logger_lib::timeline::get_timeline_for_date,
            daily_logger_lib::timeline::get_timeline_for_range,
            // Plugin system
            daily_logger_lib::plugin::list_discovered_plugins,
            daily_logger_lib::plugin::enable_plugin,
            daily_logger_lib::plugin::disable_plugin,
            daily_logger_lib::plugin::open_plugins_directory,
            // FUTURE-003: Model fine-tuning
            daily_logger_lib::fine_tuning::prepare_training_data,
            daily_logger_lib::fine_tuning::start_fine_tuning,
            daily_logger_lib::fine_tuning::get_default_fine_tuning_config,
            // Team collaboration: User authentication
            daily_logger_lib::auth::register_user,
            daily_logger_lib::auth::login_user,
            daily_logger_lib::auth::get_user_by_id,
            daily_logger_lib::auth::get_all_users,
            daily_logger_lib::auth::delete_user,
            daily_logger_lib::auth::has_any_user,
            daily_logger_lib::auth::get_current_session,
            daily_logger_lib::auth::logout,
        ])
        .setup(|app| {
            write_diagnostic_file("Tauri setup started");
            tracing::info!("Application setup complete");

            // Start background network connectivity monitor
            daily_logger_lib::network_status::start_network_monitor(app.handle().clone());
            write_diagnostic_file("Network monitor started");

            #[cfg(not(any(target_os = "android", target_os = "ios")))]
            {
                use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
                use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder};

                #[cfg(feature = "screenshot")]
                fn build_tray_menu(
                    app: &tauri::AppHandle,
                ) -> Result<Menu<tauri::Wry>, Box<dyn std::error::Error>> {
                    use daily_logger_lib::auto_perception::is_auto_capture_running;
                    use tauri::menu::CheckMenuItem;

                    let running = is_auto_capture_running();

                    let capture_toggle = CheckMenuItem::with_id(
                        app,
                        "capture_toggle",
                        "自动捕获",
                        true,
                        running,
                        None::<&str>,
                    )?;
                    let generate_summary =
                        MenuItem::with_id(app, "generate_summary", "生成日报", true, None::<&str>)?;
                    let quick_note =
                        MenuItem::with_id(app, "quick_note", "快速记录...", true, None::<&str>)?;
                    let open_obsidian = MenuItem::with_id(
                        app,
                        "open_obsidian",
                        "打开 Obsidian 文件夹",
                        true,
                        None::<&str>,
                    )?;
                    let settings =
                        MenuItem::with_id(app, "settings", "设置...", true, None::<&str>)?;
                    let show = MenuItem::with_id(app, "show", "显示窗口", true, None::<&str>)?;
                    let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
                    let separator1 = PredefinedMenuItem::separator(app)?;
                    let separator2 = PredefinedMenuItem::separator(app)?;
                    let _separator3 = PredefinedMenuItem::separator(app)?;

                    // Menu order: 自动捕获 → 生成日报 → 快速记录 → 打开Obsidian → 分隔线 → 设置 → 显示窗口 → 分隔线 → 退出
                    Menu::with_items(
                        app,
                        &[
                            &capture_toggle,
                            &generate_summary,
                            &quick_note,
                            &open_obsidian,
                            &separator1,
                            &settings,
                            &show,
                            &separator2,
                            &quit,
                        ],
                    )
                    .map_err(Into::into)
                }

                #[cfg(not(feature = "screenshot"))]
                fn build_tray_menu(
                    app: &tauri::AppHandle,
                ) -> Result<Menu<tauri::Wry>, Box<dyn std::error::Error>> {
                    let generate_summary =
                        MenuItem::with_id(app, "generate_summary", "生成日报", true, None::<&str>)?;
                    let quick_note =
                        MenuItem::with_id(app, "quick_note", "快速记录...", true, None::<&str>)?;
                    let open_obsidian = MenuItem::with_id(
                        app,
                        "open_obsidian",
                        "打开 Obsidian 文件夹",
                        true,
                        None::<&str>,
                    )?;
                    let settings =
                        MenuItem::with_id(app, "settings", "设置...", true, None::<&str>)?;
                    let show = MenuItem::with_id(app, "show", "显示窗口", true, None::<&str>)?;
                    let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
                    let separator1 = PredefinedMenuItem::separator(app)?;
                    let separator2 = PredefinedMenuItem::separator(app)?;

                    // Menu order: 生成日报 → 快速记录 → 打开Obsidian → 分隔线 → 设置 → 显示窗口 → 分隔线 → 退出
                    Menu::with_items(
                        app,
                        &[
                            &generate_summary,
                            &quick_note,
                            &open_obsidian,
                            &separator1,
                            &settings,
                            &show,
                            &separator2,
                            &quit,
                        ],
                    )
                    .map_err(Into::into)
                }

                let menu = build_tray_menu(app.handle())?;
                let tooltip = build_tray_tooltip();

                let _tray = TrayIconBuilder::new()
                    .menu(&menu)
                    .tooltip(&tooltip)
                    .on_menu_event(|app, event| match event.id.as_ref() {
                        "quit" => {
                            tracing::info!("Quit requested from tray");
                            app.exit(0);
                        }
                        "show" => {
                            if let Some(window) = app.get_webview_window("main") {
                                window.show().ok();
                                window.set_focus().ok();
                            }
                        }
                        "settings" => {
                            tracing::info!("Settings requested from tray");
                            // Show main window and emit event to open settings
                            if let Some(window) = app.get_webview_window("main") {
                                window.show().ok();
                                window.set_focus().ok();
                                // Emit event to frontend to open settings modal
                                let _ = app.emit("tray-open-settings", ());
                            }
                        }
                        "generate_summary" => {
                            tracing::info!("Generate summary requested from tray");
                            let app_handle = app.clone();
                            tauri::async_runtime::spawn(async move {
                                use daily_logger_lib::synthesis::generate_daily_summary;
                                match generate_daily_summary().await {
                                    Ok(path) => {
                                        tracing::info!("Summary generated: {}", path);
                                        let _ = app_handle.emit("summary-generated", path);
                                    }
                                    Err(e) => {
                                        tracing::error!("Failed to generate summary: {}", e);
                                        let _ = app_handle.emit("tray-error", e);
                                    }
                                }
                            });
                        }
                        "capture_toggle" => {
                            #[cfg(feature = "screenshot")]
                            {
                                use daily_logger_lib::auto_perception::{
                                    is_auto_capture_running, start_auto_capture, stop_auto_capture,
                                };
                                let running = is_auto_capture_running();
                                let app_handle = app.clone();
                                let app_handle2 = app.clone();
                                tauri::async_runtime::spawn(async move {
                                    let result = if running {
                                        tracing::info!("Stopping auto capture from tray");
                                        stop_auto_capture().await
                                    } else {
                                        tracing::info!("Starting auto capture from tray");
                                        start_auto_capture(app_handle2).await
                                    };
                                    if let Err(e) = result {
                                        tracing::error!("Failed to toggle auto capture: {}", e);
                                    }
                                    // Emit event to frontend to update UI
                                    let _ = app_handle.emit("tray-menu-update", ());
                                });
                            }
                            #[cfg(not(feature = "screenshot"))]
                            {
                                tracing::warn!("Screenshot feature not enabled");
                            }
                        }
                        "quick_note" => {
                            tracing::info!("Quick note requested from tray");
                            // Show main window and emit event to open quick note modal
                            if let Some(window) = app.get_webview_window("main") {
                                window.show().ok();
                                window.set_focus().ok();
                                // Emit event to frontend to open quick note modal
                                let _ = app.emit("tray-open-quick-note", ());
                            }
                        }
                        "open_obsidian" => {
                            tracing::info!("Open Obsidian folder requested from tray");
                            let app_handle = app.clone();
                            tauri::async_runtime::spawn(async move {
                                use daily_logger_lib::manual_entry::open_obsidian_folder;
                                if let Err(e) = open_obsidian_folder().await {
                                    tracing::error!("Failed to open Obsidian folder: {}", e);
                                    let _ = app_handle.emit("tray-error", e);
                                }
                            });
                        }
                        _ => {}
                    })
                    .on_tray_icon_event(|tray, event| {
                        if let tauri::tray::TrayIconEvent::Click {
                            button: MouseButton::Left,
                            button_state: MouseButtonState::Up,
                            ..
                        } = event
                        {
                            let app = tray.app_handle();
                            if let Some(window) = app.get_webview_window("main") {
                                window.show().ok();
                                window.set_focus().ok();
                            }
                        }
                        // Rebuild menu on right-click to show updated status
                        #[cfg(feature = "screenshot")]
                        if let tauri::tray::TrayIconEvent::Click {
                            button: MouseButton::Right,
                            button_state: MouseButtonState::Up,
                            ..
                        } = event
                        {
                            let app = tray.app_handle();
                            if let Ok(new_menu) = build_tray_menu(app) {
                                if let Err(e) = tray.set_menu(Some(new_menu)) {
                                    tracing::error!("Failed to update tray menu: {}", e);
                                }
                            }
                            // Update tooltip with current status and record count
                            let new_tooltip = build_tray_tooltip();
                            if let Err(e) = tray.set_tooltip(Some(&new_tooltip)) {
                                tracing::error!("Failed to update tray tooltip: {}", e);
                            }
                        }
                    })
                    .build(app)?;
                write_diagnostic_file("Tray icon created successfully");
            }

            write_diagnostic_file("Tauri setup completed - window should be visible");
            Ok(())
        })
        .run(tauri::generate_context!());

    write_diagnostic_file("Tauri run completed");

    if let Err(e) = result {
        write_diagnostic_file(&format!("Tauri application error: {}", e));
        tracing::error!("Tauri application error: {}", e);
        write_crash_message(&format!("Tauri application error: {}", e));
        std::process::exit(1);
    }

    write_diagnostic_file("Application exiting normally");
}
