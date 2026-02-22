#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use daily_logger_lib::init_app;
use tauri::Manager;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use std::path::PathBuf;

fn setup_logging() {
    let log_dir = get_app_data_dir().join("logs");
    std::fs::create_dir_all(&log_dir).ok();
    
    let file_appender = RollingFileAppender::new(
        Rotation::DAILY,
        log_dir,
        "daily-logger.log"
    );
    
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    
    tracing_subscriber::registry()
        .with(EnvFilter::new("info"))
        .with(fmt::layer().with_writer(non_blocking))
        .with(fmt::layer().with_writer(std::io::stdout))
        .init();
}

fn get_app_data_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("DailyLogger")
}

fn main() {
    setup_logging();
    
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
            daily_logger_lib::auto_perception::start_auto_capture,
            daily_logger_lib::auto_perception::stop_auto_capture,
            daily_logger_lib::manual_entry::add_quick_note,
            daily_logger_lib::memory_storage::get_today_records,
            daily_logger_lib::memory_storage::get_settings,
            daily_logger_lib::memory_storage::save_settings,
            daily_logger_lib::synthesis::generate_daily_summary,
        ])
        .setup(|app| {
            tracing::info!("Application setup complete");
            
            #[cfg(desktop)]
            {
                use tauri::tray::{TrayIconBuilder, MouseButton, MouseButtonState};
                use tauri::menu::{Menu, MenuItem};
                
                let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
                let show = MenuItem::with_id(app, "show", "显示窗口", true, None::<&str>)?;
                let menu = Menu::with_items(app, &[&show, &quit])?;
                
                let _tray = TrayIconBuilder::new()
                    .menu(&menu)
                    .on_menu_event(|app, event| {
                        match event.id.as_ref() {
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
                            _ => {}
                        }
                    })
                    .on_tray_icon_event(|tray, event| {
                        if let tauri::tray::TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = event {
                            let app = tray.app_handle();
                            if let Some(window) = app.get_webview_window("main") {
                                window.show().ok();
                                window.set_focus().ok();
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
