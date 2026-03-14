#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use daily_logger_lib::init_app;
use std::path::PathBuf;
use tauri::Manager;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

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
            daily_logger_lib::manual_entry::get_screenshot,
            daily_logger_lib::manual_entry::read_file,
            daily_logger_lib::manual_entry::get_recent_logs,
            daily_logger_lib::manual_entry::get_logs_for_export,
            daily_logger_lib::manual_entry::get_log_file_path,
            daily_logger_lib::memory_storage::get_today_records,
            daily_logger_lib::memory_storage::get_records_by_date_range,
            daily_logger_lib::memory_storage::get_settings,
            daily_logger_lib::memory_storage::save_settings,
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
                    let running = is_auto_capture_running();
                    let status_indicator = if running { "● " } else { "○ " };
                    let toggle_text = if running {
                        "停止自动捕获"
                    } else {
                        "启动自动捕获"
                    };
                    let capture_text = format!("{}{}", status_indicator, toggle_text);

                    let capture_toggle = MenuItem::with_id(
                        app,
                        "capture_toggle",
                        &capture_text,
                        true,
                        None::<&str>,
                    )?;
                    let show = MenuItem::with_id(app, "show", "显示窗口", true, None::<&str>)?;
                    let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
                    let separator = PredefinedMenuItem::separator(app)?;

                    Menu::with_items(app, &[&capture_toggle, &separator, &show, &quit])
                        .map_err(Into::into)
                }

                #[cfg(not(feature = "screenshot"))]
                fn build_tray_menu(
                    app: &tauri::AppHandle,
                ) -> Result<Menu<tauri::Wry>, Box<dyn std::error::Error>> {
                    let show = MenuItem::with_id(app, "show", "显示窗口", true, None::<&str>)?;
                    let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
                    Menu::with_items(app, &[&show, &quit]).map_err(Into::into)
                }

                let menu = build_tray_menu(app.handle())?;

                let _tray = TrayIconBuilder::new()
                    .menu(&menu)
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
                        "capture_toggle" => {
                            #[cfg(feature = "screenshot")]
                            {
                                use daily_logger_lib::auto_perception::is_auto_capture_running;
                                let running = is_auto_capture_running();
                                if running {
                                    tracing::info!("Stopping auto capture from tray");
                                    if let Err(e) = app.run_on_main_thread(|| {
                                        // Note: This is a simplified version. In production,
                                        // we would use async to call stop_auto_capture properly.
                                        tracing::info!("Auto capture stop requested");
                                    }) {
                                        tracing::error!("Failed to run on main thread: {}", e);
                                    }
                                } else {
                                    tracing::info!("Starting auto capture from tray");
                                    if let Err(e) = app.run_on_main_thread(|| {
                                        tracing::info!("Auto capture start requested");
                                    }) {
                                        tracing::error!("Failed to run on main thread: {}", e);
                                    }
                                }
                            }
                            #[cfg(not(feature = "screenshot"))]
                            {
                                tracing::warn!("Screenshot feature not enabled");
                            }
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
                        }
                    })
                    .build(app)?;
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
