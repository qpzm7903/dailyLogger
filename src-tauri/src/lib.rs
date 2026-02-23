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

pub fn init_app() -> tauri::Result<()> {
    memory_storage::init_database().map_err(|e| tauri::Error::Anyhow(anyhow::anyhow!("{}", e)))?;
    tracing::info!("DailyLogger initialized successfully");
    Ok(())
}
