//! Model commands - Tauri command entry points for model operations
//!
//! These commands are thin wrappers that delegate to service functions.
//! All business logic resides in the services module.

use crate::services::model_service::get_model_info_service;

/// Get model information including context window
///
/// This is a thin command wrapper that delegates to the model service.
/// No business logic is implemented here - only parameter transformation
/// and error mapping.
#[tauri::command]
pub async fn get_model_info(
    api_base_url: String,
    api_key: String,
    model_name: String,
) -> Result<crate::memory_storage::ModelInfo, String> {
    get_model_info_service(api_base_url, api_key, model_name).await
}
