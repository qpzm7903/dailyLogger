//! Session commands - Tauri command entry points for session operations
//!
//! These commands are thin wrappers that delegate to service functions.
//! No business logic is implemented here - only parameter transformation
//! and error mapping.
//!
//! SESSION-001: Session management - detect, create and manage work sessions
//! SESSION-002: Batch analysis of session screenshots via Vision API
//! SESSION-003: User summary editing for sessions

use crate::memory_storage::SessionScreenshot;
use crate::services::session_service::{
    analyze_session_service, get_session_screenshots_service, get_today_sessions_service,
    update_session_user_summary_service, Session,
};

/// Get today's sessions
///
/// This is a thin command wrapper that delegates to the session service.
/// No business logic is implemented here - only error mapping.
#[tauri::command]
pub async fn get_today_sessions() -> Result<Vec<Session>, String> {
    get_today_sessions_service().map_err(|e| e.to_string())
}

/// Analyze a session's screenshots in batch using Vision API
///
/// This is a thin command wrapper that delegates to the session service.
/// The service handles screenshot collection, API calls, retries, and result storage.
#[tauri::command]
pub async fn analyze_session(session_id: i64) -> Result<(), String> {
    analyze_session_service(session_id)
        .await
        .map_err(|e| e.to_string())
}

/// Get screenshots for a session
///
/// This is a thin command wrapper that delegates to the session service.
#[tauri::command]
pub async fn get_session_screenshots(session_id: i64) -> Result<Vec<SessionScreenshot>, String> {
    get_session_screenshots_service(session_id).map_err(|e| e.to_string())
}

/// Update user summary for a session
///
/// This is a thin command wrapper that delegates to the session service.
#[tauri::command]
pub async fn update_session_user_summary(
    session_id: i64,
    user_summary: Option<String>,
) -> Result<(), String> {
    update_session_user_summary_service(session_id, user_summary.as_deref())
        .map_err(|e| e.to_string())
}
