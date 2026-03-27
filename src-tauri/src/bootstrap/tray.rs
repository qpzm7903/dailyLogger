//! Tray icon setup and event handling
//!
//! This module handles:
//! - Building the tray menu
//! - Tray icon event handling
//! - Menu events (quit, show, settings, generate summary, etc.)

use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder},
    AppHandle, Emitter, Manager,
};

#[cfg(feature = "screenshot")]
use daily_logger_lib::commands::capture_commands::{start_auto_capture, stop_auto_capture};
#[cfg(feature = "screenshot")]
use daily_logger_lib::services::capture_service::is_auto_capture_running;

use crate::bootstrap::logging::build_tray_tooltip;

/// Build the tray menu with all menu items
#[cfg(feature = "screenshot")]
pub fn build_tray_menu(app: &AppHandle) -> Result<Menu<tauri::Wry>, Box<dyn std::error::Error>> {
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
    let quick_note = MenuItem::with_id(app, "quick_note", "快速记录...", true, None::<&str>)?;
    let open_obsidian = MenuItem::with_id(
        app,
        "open_obsidian",
        "打开 Obsidian 文件夹",
        true,
        None::<&str>,
    )?;
    let settings = MenuItem::with_id(app, "settings", "设置...", true, None::<&str>)?;
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
pub fn build_tray_menu(app: &AppHandle) -> Result<Menu<tauri::Wry>, Box<dyn std::error::Error>> {
    let generate_summary =
        MenuItem::with_id(app, "generate_summary", "生成日报", true, None::<&str>)?;
    let quick_note = MenuItem::with_id(app, "quick_note", "快速记录...", true, None::<&str>)?;
    let open_obsidian = MenuItem::with_id(
        app,
        "open_obsidian",
        "打开 Obsidian 文件夹",
        true,
        None::<&str>,
    )?;
    let settings = MenuItem::with_id(app, "settings", "设置...", true, None::<&str>)?;
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

/// Setup the tray icon with menu and event handlers
pub fn setup_tray(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let menu = build_tray_menu(app)?;
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

    Ok(())
}
