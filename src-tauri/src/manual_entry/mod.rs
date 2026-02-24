use crate::memory_storage;
use tauri::command;

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
    let image_data =
        std::fs::read(&path).map_err(|e| format!("Failed to read screenshot: {}", e))?;

    let base64_data =
        base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &image_data);
    Ok(format!("data:image/png;base64,{}", base64_data))
}

#[command]
pub async fn read_file(path: String) -> Result<String, String> {
    std::fs::read_to_string(&path).map_err(|e| format!("Failed to read file: {}", e))
}

#[command]
pub async fn get_recent_logs(lines: Option<usize>) -> Result<String, String> {
    let log_path = dirs::data_dir()
        .ok_or_else(|| "Cannot determine data directory".to_string())?
        .join("DailyLogger")
        .join("logs")
        .join("daily-logger.log");

    if !log_path.exists() {
        return Ok(String::new());
    }

    let content = std::fs::read_to_string(&log_path)
        .map_err(|e| format!("Failed to read log file: {}", e))?;

    let n = lines.unwrap_or(300);
    let recent: Vec<&str> = content
        .lines()
        .rev()
        .take(n)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();

    Ok(recent.join("\n"))
}
