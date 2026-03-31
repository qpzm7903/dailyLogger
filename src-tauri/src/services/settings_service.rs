//! Settings service - Business logic for settings management
//!
//! This module contains business logic functions for settings operations.
//! Commands should delegate to these service functions rather than implementing logic directly.

use crate::errors::AppResult;
use crate::memory_storage::Settings;

/// Get settings from the database.
///
/// This function retrieves the current application settings.
/// It wraps the sync function from memory_storage for consistency with the service layer pattern.
pub fn get_settings_service() -> AppResult<Settings> {
    crate::memory_storage::get_settings_sync()
}

/// Save settings to the database.
///
/// This function saves the application settings.
/// It wraps the sync function from memory_storage for consistency with the service layer pattern.
pub fn save_settings_service(settings: &Settings) -> AppResult<()> {
    crate::memory_storage::save_settings_sync(settings)
}
