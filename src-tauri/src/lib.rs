#[cfg(feature = "screenshot")]
pub mod auto_perception;
pub mod backup;
pub mod crypto;
pub mod export;
pub mod manual_entry;
pub mod memory_storage;
#[cfg(feature = "screenshot")]
pub mod monitor;
pub mod monitor_types;
pub mod network_status;
pub mod offline_queue;
pub mod ollama;
pub mod silent_tracker;
pub mod synthesis;
pub mod window_info;
pub mod work_time;

use once_cell::sync::Lazy;
use std::sync::Mutex;

pub static APP_STATE: Lazy<Mutex<AppState>> = Lazy::new(|| Mutex::new(AppState::default()));

#[derive(Default)]
pub struct AppState {
    pub auto_capture_running: bool,
}

/// Mask an API key for safe logging: show prefix (up to 5 chars) + "..." + "****".
/// Example: "sk-abc123xyz9999" -> "sk-ab...****"
pub fn mask_api_key(key: &str) -> String {
    if key.is_empty() {
        return "****".to_string();
    }

    // Check if it's an encrypted key - don't reveal any part of it
    if crate::crypto::is_encrypted(key) {
        return "[encrypted]".to_string();
    }

    // Show prefix (first 5 chars or less) + "..." + "****"
    let prefix_len = key.len().min(5);
    format!("{}...****", &key[..prefix_len])
}

/// Add two i32 values and return their sum.
pub fn add(a: i32, b: i32) -> i32 {
    a + b
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
    fn mask_api_key_shows_prefix() {
        assert_eq!(mask_api_key("sk-abc123xyz9999"), "sk-ab...****");
    }

    #[test]
    fn mask_api_key_short_key_shows_available_prefix() {
        assert_eq!(mask_api_key("ab"), "ab...****");
        assert_eq!(mask_api_key("abcd"), "abcd...****");
    }

    #[test]
    fn mask_api_key_empty_string() {
        assert_eq!(mask_api_key(""), "****");
    }

    #[test]
    fn mask_api_key_exactly_five_chars() {
        assert_eq!(mask_api_key("12345"), "12345...****");
    }

    #[test]
    fn mask_api_key_encrypted_key() {
        assert_eq!(mask_api_key("ENC:somebase64data"), "[encrypted]");
    }

    #[test]
    fn add_positive_numbers() {
        assert_eq!(add(2, 3), 5);
    }

    #[test]
    fn add_negative_and_positive() {
        assert_eq!(add(-1, 1), 0);
    }

    #[test]
    fn add_zeros() {
        assert_eq!(add(0, 0), 0);
    }
}
