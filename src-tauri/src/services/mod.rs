//! Services module - Business logic layer
//!
//! This module contains business logic functions organized by domain.
//! Commands (in `commands/`) are thin wrappers that delegate to these services.
//!
//! Design principles:
//! - Services contain no `#[tauri::command]` attributes
//! - Services use idiomatic Rust error handling
//! - Commands (in `commands/`) are responsible for error mapping to Tauri responses

pub mod model_service;

// Re-export business logic from existing modules
pub use crate::memory_storage::{
    delete_record_sync, get_history_records_cursor_sync, get_history_records_sync,
    get_records_by_date_range_sync, get_settings_sync, get_today_records_sync,
    get_today_stats_sync, save_settings_sync, search_records_sync, update_record_user_notes_sync,
};
pub use crate::session_manager::get_today_sessions_sync;
pub use crate::synthesis::{get_default_summary_prompt, get_supported_languages};

// Service functions from model_service
pub use model_service::get_model_info_service;
