//! Services module - Business logic layer
//!
//! This module contains business logic functions organized by domain.
//! Commands (in `commands/`) are thin wrappers that delegate to these services.
//!
//! Design principles:
//! - Services contain no `#[tauri::command]` attributes
//! - Services use idiomatic Rust error handling
//! - Commands (in `commands/`) are responsible for error mapping to Tauri responses

#[cfg(feature = "screenshot")]
pub mod capture_service;
pub mod model_service;
pub mod report_service;
pub mod session_service;
pub mod settings_service;
pub mod vision_api;

// Re-export business logic from existing modules
pub use crate::memory_storage::{
    delete_record_sync, get_history_records_cursor_sync, get_history_records_sync,
    get_records_by_date_range_sync, get_settings_sync, get_today_records_sync,
    get_today_stats_sync, save_settings_sync, search_records_sync, update_record_user_notes_sync,
};
pub use crate::synthesis::{get_default_summary_prompt, get_supported_languages};

// Service functions from model_service
pub use model_service::get_model_info_service;

// Service functions from session_service
pub use session_service::{
    analyze_session, analyze_session_service, detect_or_create_session, end_current_session,
    get_session_screenshots_service, get_today_sessions_service, get_today_sessions_sync,
    update_session_user_summary_service, Session, SessionAnalysisResponse, SessionStatus,
};

// Service functions from settings_service
pub use settings_service::{get_settings_service, save_settings_service};

// Re-export capture_service functions for use by auto_perception and commands
#[cfg(feature = "screenshot")]
pub use capture_service::{
    evaluate_and_adjust_threshold, get_auto_capture_status_service,
    get_default_analysis_prompt_service, get_filtered_today, get_quality_filter_stats_service,
    get_work_time_status_service, is_auto_capture_running, reanalyze_record_service,
    reanalyze_records_by_date_service, reanalyze_today_records_service, reset_filtered_count,
    reset_quality_filter_counter_service, retry_screenshot_analysis_service,
    should_capture_by_work_time, start_auto_capture_service, stop_auto_capture_service,
    take_screenshot_service, trigger_auto_capture_service, trigger_capture_service,
    CaptureSettings, QualityFilterStats, ReanalyzeResult, ScreenAnalysis, ThresholdAdjustment,
};
