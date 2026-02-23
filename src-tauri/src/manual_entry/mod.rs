use tauri::command;
use crate::memory_storage;

#[command]
pub async fn add_quick_note(content: String) -> Result<(), String> {
    if content.trim().is_empty() {
        return Err("Content cannot be empty".to_string());
    }
    
    memory_storage::add_record("manual", &content, None)
        .map_err(|e| format!("Failed to save note: {}", e))?;
    
    tracing::info!("Quick note added: {}...", &content[..content.len().min(50)]);
    Ok(())
}

#[command]
pub async fn get_screenshot(path: String) -> Result<String, String> {
    let image_data = std::fs::read(&path)
        .map_err(|e| format!("Failed to read screenshot: {}", e))?;
    
    let base64_data = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &image_data);
    Ok(format!("data:image/png;base64,{}", base64_data))
}

#[command]
pub async fn read_file(path: String) -> Result<String, String> {
    std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read file: {}", e))
}
