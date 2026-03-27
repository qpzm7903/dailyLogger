//! Settings service - Business logic for settings management
//!
//! This module contains business logic functions for settings operations.
//! Commands should delegate to these service functions rather than implementing logic directly.

use crate::memory_storage::Settings;

/// Get settings from the database.
///
/// This function retrieves the current application settings.
/// It wraps the sync function from memory_storage for consistency with the service layer pattern.
///
/// # Returns
/// * `Ok(Settings)` with current settings on success
/// * `Err(String)` with error message on failure
pub fn get_settings_service() -> Result<Settings, String> {
    crate::memory_storage::get_settings_sync()
}

/// Save settings to the database.
///
/// This function saves the application settings.
/// It wraps the sync function from memory_storage for consistency with the service layer pattern.
///
/// # Arguments
/// * `settings` - The settings to save
///
/// # Returns
/// * `Ok(())` on success
/// * `Err(String)` with error message on failure
pub fn save_settings_service(settings: &Settings) -> Result<(), String> {
    crate::memory_storage::save_settings_sync(settings)
}
