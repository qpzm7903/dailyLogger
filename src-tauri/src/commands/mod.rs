//! Commands module - Tauri command entry points
//!
//! This module acts as a facade that re-exports commands from domain modules.
//! This provides a clear boundary between the command layer and the service layer.
//!
//! Design principles:
//! - Commands serve as thin IPC adapter layer
//! - Business logic resides in the `services` module
//! - This module re-exports commands for a clean public API

// Re-export all commands from their respective domain modules
// This allows bootstrap/commands.rs to import from a single location

// Manual entry commands
pub use crate::manual_entry::{
    add_quick_note, get_log_file_path, get_logs_for_export, get_recent_logs, get_screenshot,
    list_report_files, log_frontend_error, open_obsidian_folder, read_file, tray_quick_note,
};

// Memory storage commands (records, settings, tags)
pub use crate::memory_storage::{
    add_tag_to_record,
    create_manual_tag,
    delete_manual_tag,
    delete_record,
    get_all_manual_tags,
    get_all_tags,
    // Tags
    get_default_tag_categories,
    get_history_records,
    get_history_records_cursor,
    get_model_info,
    get_records_by_date_range,
    get_records_by_manual_tags,
    get_records_by_tag,
    get_settings,
    get_statistics,
    get_tags_for_record,
    get_tags_for_records,
    // Records
    get_today_records,
    get_today_stats,
    remove_tag_from_record,
    save_settings,
    search_records,
    update_manual_tag,
    update_record_user_notes,
};

// Session commands
pub use crate::session_manager::{
    analyze_session, get_session_screenshots, get_today_sessions, update_session_user_summary,
};

// Synthesis/Report commands
pub use crate::synthesis::{
    compare_reports, generate_custom_report, generate_daily_summary, generate_monthly_report,
    generate_multilingual_daily_summary, generate_weekly_report, get_default_summary_prompt,
    get_supported_languages,
};

// Export commands
pub use crate::export::{export_records, open_export_dir};

// Backup commands
pub use crate::backup::{
    create_backup, delete_backup, get_backup_info, list_backups, restore_backup,
};

// Ollama commands
pub use crate::ollama::{
    copy_ollama_model, create_ollama_model, delete_ollama_model, get_ollama_models,
    get_running_models, pull_ollama_model, show_ollama_model, test_api_connection_with_ollama,
};

// Notification commands
pub use crate::dingtalk::test_dingtalk_connection;
pub use crate::notion::test_notion_connection;
pub use crate::slack::test_slack_connection;

// Network commands
pub use crate::network_status::{check_network_status, get_network_status};

// Performance commands
pub use crate::performance::{
    benchmark_database_query, get_memory_usage_mb, get_platform_info, run_performance_benchmark,
};

#[cfg(feature = "screenshot")]
pub use crate::performance::benchmark_screenshot_processing;

// Offline queue commands
pub use crate::offline_queue::{
    get_offline_queue_status, get_pending_offline_tasks, process_offline_queue,
};

// Screenshot/Auto perception commands
#[cfg(feature = "screenshot")]
pub use crate::auto_perception::{
    get_auto_capture_status, get_default_analysis_prompt, get_quality_filter_stats,
    get_work_time_status, reanalyze_record, reanalyze_records_by_date, reanalyze_today_records,
    reset_quality_filter_counter, start_auto_capture, stop_auto_capture, take_screenshot,
    trigger_capture,
};

// Monitor commands
#[cfg(feature = "screenshot")]
pub use crate::monitor::get_monitors;

// Timeline commands
pub use crate::timeline::{get_timeline_for_date, get_timeline_for_range, get_timeline_today};

// Auto backup scheduler commands
pub use crate::auto_backup_scheduler::trigger_auto_backup;
