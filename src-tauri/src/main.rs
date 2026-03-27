#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use daily_logger_lib::init_app;

mod bootstrap;

use bootstrap::commands::register_commands;
#[cfg(target_os = "windows")]
use bootstrap::logging::{is_webview2_installed, show_webview2_error_message};
use bootstrap::logging::{
    setup_logging, write_crash_log, write_crash_message, write_diagnostic_file,
};
use bootstrap::tray::setup_tray;

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
                Or use the installer version (*-setup.exe) which will automatically\n                install WebView2 for you.";

            tracing::error!("{}", error_msg);
            write_crash_message(error_msg);

            // Show a Windows message box so the user can see the error
            show_webview2_error_message(error_msg);

            std::process::exit(1);
        }
        write_diagnostic_file("WebView2 found");
    }

    write_diagnostic_file("Initializing app...");
    tracing::info!("Starting init_app()");
    if let Err(e) = init_app() {
        write_diagnostic_file(&format!("Failed to initialize app: {}", e));
        tracing::error!("Failed to initialize app: {}", e);
        eprintln!("Fatal: Failed to initialize app: {}", e);
        write_crash_message(&format!("Failed to initialize app: {}", e));
        return;
    }
    write_diagnostic_file("App initialized successfully");
    tracing::info!("init_app() completed successfully");

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

    // Register all commands
    let builder = register_commands(builder);

    let result = builder
        .setup(|app| {
            write_diagnostic_file("Tauri setup started");
            tracing::info!("Application setup complete");

            // Start background network connectivity monitor
            daily_logger_lib::network_status::start_network_monitor(app.handle().clone());
            write_diagnostic_file("Network monitor started");

            // Setup tray icon on desktop platforms
            #[cfg(not(any(target_os = "android", target_os = "ios")))]
            {
                if let Err(e) = setup_tray(app.handle()) {
                    tracing::error!("Failed to setup tray: {}", e);
                    write_diagnostic_file(&format!("Failed to setup tray: {}", e));
                } else {
                    write_diagnostic_file("Tray icon created successfully");
                }
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
