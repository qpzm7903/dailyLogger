//! Settings commands - Tauri command entry points for settings operations
//!
//! These commands are thin wrappers that delegate to service functions.
//! No business logic is implemented here - only parameter transformation
//! and error mapping.

use std::sync::Arc;

use crate::memory_storage::Settings;
use crate::services::settings_service::{get_settings_service, save_settings_service};

/// Get application settings
///
/// This is a thin command wrapper that delegates to the settings service.
/// Unwraps the `Arc<Settings>` at the IPC boundary (Tauri serialization
/// requires ownership of the value).
#[tauri::command]
pub async fn get_settings() -> Result<Settings, String> {
    get_settings_service()
        .map(|arc: Arc<Settings>| Arc::try_unwrap(arc).unwrap_or_else(|arc| (*arc).clone()))
        .map_err(|e| e.to_string())
}

/// Save application settings
///
/// This is a thin command wrapper that delegates to the settings service.
/// No business logic is implemented here - only error mapping.
#[tauri::command]
pub async fn save_settings(settings: Settings) -> Result<(), String> {
    save_settings_service(&settings).map_err(|e| e.to_string())
}
