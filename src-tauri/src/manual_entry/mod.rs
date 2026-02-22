use tauri::command;
use crate::memory_storage;

#[command]
pub async fn add_quick_note(content: String) -> Result<(), String> {
    if content.trim().is_empty() {
        return Err("Content cannot be empty".to_string());
    }
    
    memory_storage::add_record("manual", &content)
        .map_err(|e| format!("Failed to save note: {}", e))?;
    
    tracing::info!("Quick note added: {}...", &content[..content.len().min(50)]);
    Ok(())
}
