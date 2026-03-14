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

/// Get the log file content for export
#[command]
pub async fn get_logs_for_export() -> Result<String, String> {
    let log_path = dirs::data_dir()
        .ok_or_else(|| "Cannot determine data directory".to_string())?
        .join("DailyLogger")
        .join("logs")
        .join("daily-logger.log");

    if !log_path.exists() {
        return Err("Log file does not exist".to_string());
    }

    std::fs::read_to_string(&log_path).map_err(|e| format!("Failed to read log file: {}", e))
}

/// Get the log file path for export
#[command]
pub async fn get_log_file_path() -> Result<String, String> {
    let log_path = dirs::data_dir()
        .ok_or_else(|| "Cannot determine data directory".to_string())?
        .join("DailyLogger")
        .join("logs")
        .join("daily-logger.log");

    log_path
        .to_str()
        .map(|s| s.to_string())
        .ok_or_else(|| "Invalid log file path".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_logs_for_export_no_file() {
        // Test that get_logs_for_export returns error when log file doesn't exist
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(get_logs_for_export());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Log file does not exist"));
    }

    #[test]
    fn test_get_log_file_path() {
        // Test that get_log_file_path returns a valid path
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(get_log_file_path());
        assert!(result.is_ok());
        let path = result.unwrap();
        assert!(path.contains("DailyLogger"));
        assert!(path.contains("logs"));
        assert!(path.ends_with("daily-logger.log"));
    }

    #[test]
    fn test_get_recent_logs_empty() {
        // Test that get_recent_logs returns empty string when no log file exists
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(get_recent_logs(None));
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_get_recent_logs_with_lines() {
        // Test that get_recent_logs with specific line count works
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(get_recent_logs(Some(10)));
        assert!(result.is_ok());
    }
}
