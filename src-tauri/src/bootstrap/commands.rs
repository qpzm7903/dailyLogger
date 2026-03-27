//! Command registration
//!
//! This module contains all Tauri command handlers organized by feature area.

use tauri::Wry;

/// Register all Tauri commands with the builder
pub fn register_commands(builder: tauri::Builder<Wry>) -> tauri::Builder<Wry> {
    builder.invoke_handler(tauri::generate_handler![
        // Manual entry commands
        daily_logger_lib::manual_entry::add_quick_note,
        daily_logger_lib::manual_entry::tray_quick_note,
        daily_logger_lib::manual_entry::get_screenshot,
        daily_logger_lib::manual_entry::read_file,
        daily_logger_lib::manual_entry::get_recent_logs,
        daily_logger_lib::manual_entry::get_logs_for_export,
        daily_logger_lib::manual_entry::get_log_file_path,
        daily_logger_lib::manual_entry::log_frontend_error,
        daily_logger_lib::manual_entry::open_obsidian_folder,
        daily_logger_lib::manual_entry::list_report_files,
        // Memory storage commands
        daily_logger_lib::memory_storage::get_today_records,
        daily_logger_lib::memory_storage::get_today_stats,
        daily_logger_lib::memory_storage::get_records_by_date_range,
        daily_logger_lib::commands::settings_commands::get_settings,
        daily_logger_lib::commands::settings_commands::save_settings,
        daily_logger_lib::ollama::test_api_connection_with_ollama,
        daily_logger_lib::commands::model_commands::get_model_info,
        daily_logger_lib::memory_storage::delete_record,
        daily_logger_lib::memory_storage::get_history_records,
        daily_logger_lib::memory_storage::get_history_records_cursor,
        daily_logger_lib::memory_storage::search_records,
        daily_logger_lib::memory_storage::get_default_tag_categories,
        daily_logger_lib::memory_storage::get_all_tags,
        daily_logger_lib::memory_storage::get_records_by_tag,
        // DATA-003: Manual tag system
        daily_logger_lib::memory_storage::create_manual_tag,
        daily_logger_lib::memory_storage::get_all_manual_tags,
        daily_logger_lib::memory_storage::update_manual_tag,
        daily_logger_lib::memory_storage::delete_manual_tag,
        daily_logger_lib::memory_storage::add_tag_to_record,
        daily_logger_lib::memory_storage::remove_tag_from_record,
        daily_logger_lib::memory_storage::get_tags_for_record,
        daily_logger_lib::memory_storage::get_tags_for_records,
        daily_logger_lib::memory_storage::get_records_by_manual_tags,
        // FEAT-005: User notes for screenshot records
        daily_logger_lib::memory_storage::update_record_user_notes,
        // SESSION-001: Session management
        daily_logger_lib::commands::session_commands::get_today_sessions,
        // SESSION-002: Session batch analysis
        daily_logger_lib::commands::session_commands::analyze_session,
        daily_logger_lib::commands::session_commands::get_session_screenshots,
        // SESSION-003: Session summary editing
        daily_logger_lib::commands::session_commands::update_session_user_summary,
        // DATA-008: Statistics panel
        daily_logger_lib::memory_storage::get_statistics,
        // Synthesis commands
        daily_logger_lib::synthesis::generate_daily_summary,
        daily_logger_lib::synthesis::generate_multilingual_daily_summary,
        daily_logger_lib::synthesis::get_supported_languages,
        daily_logger_lib::synthesis::get_default_summary_prompt,
        daily_logger_lib::synthesis::generate_weekly_report,
        daily_logger_lib::synthesis::generate_monthly_report,
        daily_logger_lib::synthesis::generate_custom_report,
        daily_logger_lib::synthesis::compare_reports,
        // DATA-004: Data export
        daily_logger_lib::export::export_records,
        daily_logger_lib::export::open_export_dir,
        // DATA-005: Data backup and restore
        daily_logger_lib::backup::create_backup,
        daily_logger_lib::backup::get_backup_info,
        daily_logger_lib::backup::list_backups,
        daily_logger_lib::backup::delete_backup,
        daily_logger_lib::backup::restore_backup,
        // Ollama commands
        daily_logger_lib::ollama::get_ollama_models,
        daily_logger_lib::ollama::pull_ollama_model,
        daily_logger_lib::ollama::delete_ollama_model,
        daily_logger_lib::ollama::get_running_models,
        daily_logger_lib::ollama::create_ollama_model,
        daily_logger_lib::ollama::copy_ollama_model,
        daily_logger_lib::ollama::show_ollama_model,
        // Notification commands
        daily_logger_lib::notion::test_notion_connection,
        daily_logger_lib::slack::test_slack_connection,
        daily_logger_lib::dingtalk::test_dingtalk_connection,
        // Network status commands
        daily_logger_lib::network_status::get_network_status,
        daily_logger_lib::network_status::check_network_status,
        // CORE-008: Performance benchmark
        daily_logger_lib::performance::get_platform_info,
        daily_logger_lib::performance::get_memory_usage_mb,
        daily_logger_lib::performance::benchmark_database_query,
        daily_logger_lib::performance::run_performance_benchmark,
        #[cfg(feature = "screenshot")]
        daily_logger_lib::performance::benchmark_screenshot_processing,
        // Offline queue commands
        daily_logger_lib::offline_queue::get_offline_queue_status,
        daily_logger_lib::offline_queue::get_pending_offline_tasks,
        daily_logger_lib::offline_queue::process_offline_queue,
        // Auto perception commands (screenshot feature)
        #[cfg(feature = "screenshot")]
        daily_logger_lib::auto_perception::start_auto_capture,
        #[cfg(feature = "screenshot")]
        daily_logger_lib::auto_perception::stop_auto_capture,
        #[cfg(feature = "screenshot")]
        daily_logger_lib::auto_perception::trigger_capture,
        #[cfg(feature = "screenshot")]
        daily_logger_lib::auto_perception::take_screenshot,
        #[cfg(feature = "screenshot")]
        daily_logger_lib::auto_perception::reanalyze_record,
        #[cfg(feature = "screenshot")]
        daily_logger_lib::auto_perception::reanalyze_today_records,
        #[cfg(feature = "screenshot")]
        daily_logger_lib::auto_perception::reanalyze_records_by_date,
        #[cfg(feature = "screenshot")]
        daily_logger_lib::auto_perception::get_default_analysis_prompt,
        #[cfg(feature = "screenshot")]
        daily_logger_lib::auto_perception::get_auto_capture_status,
        #[cfg(feature = "screenshot")]
        daily_logger_lib::auto_perception::get_work_time_status,
        // Monitor commands
        #[cfg(feature = "screenshot")]
        daily_logger_lib::monitor::get_monitors,
        // EXP-002: Quality filter stats
        #[cfg(feature = "screenshot")]
        daily_logger_lib::auto_perception::get_quality_filter_stats,
        #[cfg(feature = "screenshot")]
        daily_logger_lib::auto_perception::reset_quality_filter_counter,
        // Timeline commands
        daily_logger_lib::timeline::get_timeline_today,
        daily_logger_lib::timeline::get_timeline_for_date,
        daily_logger_lib::timeline::get_timeline_for_range,
        // STAB-002: Auto backup scheduler
        daily_logger_lib::auto_backup_scheduler::trigger_auto_backup,
    ])
}
