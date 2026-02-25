pub mod auto_perception;
pub mod manual_entry;
pub mod memory_storage;
pub mod synthesis;

use once_cell::sync::Lazy;
use std::sync::Mutex;

pub static APP_STATE: Lazy<Mutex<AppState>> = Lazy::new(|| Mutex::new(AppState::default()));

#[derive(Default)]
pub struct AppState {
    pub auto_capture_running: bool,
}

/// Mask an API key for safe logging: show only the last 4 characters.
pub fn mask_api_key(key: &str) -> String {
    if key.len() <= 4 {
        return "****".to_string();
    }
    format!("****{}", &key[key.len() - 4..])
}

pub fn init_app() -> tauri::Result<()> {
    memory_storage::init_database().map_err(|e| tauri::Error::Anyhow(anyhow::anyhow!("{}", e)))?;
    tracing::info!("DailyLogger initialized successfully");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mask_api_key_hides_prefix() {
        assert_eq!(mask_api_key("sk-abc123xyz9999"), "****9999");
    }

    #[test]
    fn mask_api_key_short_key_fully_masked() {
        assert_eq!(mask_api_key("ab"), "****");
        assert_eq!(mask_api_key("abcd"), "****");
    }

    #[test]
    fn mask_api_key_empty_string() {
        assert_eq!(mask_api_key(""), "****");
    }

    #[test]
    fn mask_api_key_exactly_five_chars() {
        assert_eq!(mask_api_key("12345"), "****2345");
    }
}
