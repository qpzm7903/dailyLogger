#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use daily_logger_lib::init_app;
use std::path::PathBuf;
use tauri::{Emitter, Manager};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

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

fn setup_logging() -> WorkerGuard {
    let log_dir = get_app_data_dir().join("logs");
    std::fs::create_dir_all(&log_dir).ok();

    // Rotation::NEVER keeps filename as "daily-logger.log" (no date suffix),
    // which matches what get_recent_logs() reads.
    let file_appender = RollingFileAppender::new(Rotation::NEVER, log_dir, "daily-logger.log");

    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::registry()
        .with(EnvFilter::new("info"))
        .with(fmt::layer().with_writer(non_blocking))
        .with(fmt::layer().with_writer(std::io::stdout))
        .init();

    // Return guard so it stays alive until main() exits.
    // If dropped early, the background logging thread terminates and all log
    // messages are silently discarded.
    guard
}

fn get_app_data_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("DailyLogger")
}

fn main() {
    // _log_guard must live until main() returns; dropping it early stops the
    // background logging thread and discards all subsequent log messages.
    let _log_guard = setup_logging();

    if let Err(e) = init_app() {
        tracing::error!("Failed to initialize app: {}", e);
    }

    std::panic::set_hook(Box::new(|panic_info| {
        tracing::error!("Application panic: {}", panic_info);
    }));

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
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
            daily_logger_lib::memory_storage::test_api_connection,
            daily_logger_lib::memory_storage::get_model_info,
            daily_logger_lib::memory_storage::delete_record,
            daily_logger_lib::memory_storage::get_history_records,
            daily_logger_lib::memory_storage::search_records,
            daily_logger_lib::synthesis::generate_daily_summary,
            daily_logger_lib::synthesis::get_default_summary_prompt,
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
        ])
        .setup(|app| {
            tracing::info!("Application setup complete");

            #[cfg(desktop)]
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
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
