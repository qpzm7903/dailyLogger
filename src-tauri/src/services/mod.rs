//! Services module - Business logic layer
//!
//! This module re-exports business logic functions from the existing modules.
//! Over time, functions will be migrated here from their current locations to create
//! a clear separation between command handling and business logic.
//!
//! Design principles:
//! - Services contain no `#[tauri::command]` attributes
//! - Services use idiomatic Rust error handling
//! - Commands (in `commands/`) are responsible for error mapping to Tauri responses

// Re-export business logic from existing modules
pub use crate::memory_storage::{
    delete_record_sync, get_history_records_cursor_sync, get_history_records_sync,
    get_records_by_date_range_sync, get_settings_sync, get_today_records_sync,
    get_today_stats_sync, save_settings_sync, search_records_sync, update_record_user_notes_sync,
};
pub use crate::session_manager::get_today_sessions_sync;
pub use crate::synthesis::{get_default_summary_prompt, get_supported_languages};
