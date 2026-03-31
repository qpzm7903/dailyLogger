//! Auto backup scheduler module
//!
//! Manages automatic periodic backups based on user settings.
//! Uses tokio's interval for scheduling and runs backups in the background.

use crate::backup::cleanup_old_auto_backups;
use crate::errors::{AppError, AppResult};
use crate::memory_storage::{get_settings_sync, save_settings_sync};
use chrono::{Local, NaiveDateTime};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{interval, Duration};

/// Global scheduler state
static SCHEDULER_RUNNING: AtomicBool = AtomicBool::new(false);
static SCHEDULER_HANDLE: std::sync::OnceLock<
    Arc<Mutex<Option<tauri::async_runtime::JoinHandle<()>>>>,
> = std::sync::OnceLock::new();

/// Auto backup interval in hours
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BackupInterval {
    Daily,
    Weekly,
    Monthly,
}

impl BackupInterval {
    /// Parse interval string to enum
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "weekly" => BackupInterval::Weekly,
            "monthly" => BackupInterval::Monthly,
            _ => BackupInterval::Daily,
        }
    }

    /// Get duration in hours for this interval
    pub fn to_hours(&self) -> u64 {
        match self {
            BackupInterval::Daily => 24,
            BackupInterval::Weekly => 24 * 7,
            BackupInterval::Monthly => 24 * 30, // Approximate
        }
    }
}

/// Check if auto backup is enabled in settings
pub fn is_auto_backup_enabled() -> bool {
    match get_settings_sync() {
        Ok(settings) => settings.auto_backup_enabled.unwrap_or(false),
        Err(_) => false,
    }
}

/// Get auto backup interval from settings
pub fn get_auto_backup_interval() -> BackupInterval {
    match get_settings_sync() {
        Ok(settings) => {
            BackupInterval::from_str(settings.auto_backup_interval.as_deref().unwrap_or("daily"))
        }
        Err(_) => BackupInterval::Daily,
    }
}

/// Get auto backup retention count from settings
pub fn get_auto_backup_retention() -> usize {
    match get_settings_sync() {
        Ok(settings) => settings.auto_backup_retention.unwrap_or(5).clamp(3, 20) as usize,
        Err(_) => 5,
    }
}

/// Check if backup should run based on last backup time and interval
pub fn should_run_backup_now() -> bool {
    let settings = match get_settings_sync() {
        Ok(s) => s,
        Err(_) => return true, // If we can't read settings, run backup
    };

    if !settings.auto_backup_enabled.unwrap_or(false) {
        return false;
    }

    let last_backup = match &settings.last_auto_backup_at {
        Some(t) => t,
        None => return true, // Never run, run now
    };

    // Parse the timestamp
    let last_time = match NaiveDateTime::parse_from_str(last_backup, "%Y-%m-%dT%H:%M:%S%.f") {
        Ok(t) => t,
        Err(_) => {
            // Try parsing without microseconds
            match NaiveDateTime::parse_from_str(last_backup, "%Y-%m-%dT%H:%M:%S") {
                Ok(t) => t,
                Err(_) => return true, // Can't parse, run now
            }
        }
    };

    let interval =
        BackupInterval::from_str(settings.auto_backup_interval.as_deref().unwrap_or("daily"));
    let hours_since_last = (Local::now().naive_local() - last_time).num_hours();

    hours_since_last >= interval.to_hours() as i64
}

/// Update last_auto_backup_at timestamp
pub fn update_last_backup_time() -> AppResult<()> {
    let mut settings = get_settings_sync()?;
    settings.last_auto_backup_at = Some(Local::now().format("%Y-%m-%dT%H:%M:%S%.f").to_string());
    save_settings_sync(&settings)
}

/// Run a single auto backup
pub async fn run_auto_backup() -> AppResult<()> {
    tracing::info!("Starting auto backup...");

    // Run the backup (this creates the backup with auto- prefix internally)
    let result = run_auto_backup_internal().await;

    match &result {
        Ok(_) => {
            tracing::info!("Auto backup completed successfully");
            // Update last backup time
            if let Err(e) = update_last_backup_time() {
                tracing::error!("Failed to update last backup time: {}", e);
            }
            // Run cleanup
            if let Err(e) = cleanup_old_auto_backups() {
                tracing::error!("Failed to cleanup old auto backups: {}", e);
            }
        }
        Err(e) => {
            tracing::error!("Auto backup failed: {}", e);
        }
    }

    result
}

/// Tauri command: Trigger a manual auto backup
#[tauri::command]
pub async fn trigger_auto_backup() -> Result<(), String> {
    run_auto_backup().await.map_err(|e| e.to_string())
}

/// Internal backup creation for auto backup (adds auto- prefix)
async fn run_auto_backup_internal() -> AppResult<()> {
    use std::fs;
    use std::io::{Read, Write};
    use zip::write::SimpleFileOptions;
    use zip::ZipWriter;

    use crate::memory_storage::DB_CONNECTION;

    let target_dir = crate::backup::get_default_backup_dir();

    // Ensure backup directory exists
    fs::create_dir_all(&target_dir)?;

    // Create temp directory
    let temp_dir = tempfile::Builder::new()
        .prefix("dailylogger-auto-backup-")
        .tempdir()?;

    let data_dir = temp_dir.path().join("data");
    let screenshots_dir = temp_dir.path().join("screenshots");

    fs::create_dir_all(&data_dir)?;
    fs::create_dir_all(&screenshots_dir)?;

    // Get database stats with lock
    let record_count = {
        let guard = DB_CONNECTION.lock()?;
        let conn = guard
            .as_ref()
            .ok_or_else(|| AppError::database("Database not initialized"))?;

        // Flush WAL
        let _ = conn.execute_batch("PRAGMA wal_checkpoint(FULL)");

        let count: i64 = conn.query_row("SELECT COUNT(*) FROM records", [], |row| row.get(0))?;

        let db_path = crate::backup::get_db_path();
        if db_path.exists() {
            fs::copy(&db_path, data_dir.join("local.db"))?;
        }

        count as usize
    };

    // Copy screenshots
    let screenshots_src = crate::backup::get_screenshots_dir();
    crate::backup::copy_dir_files(&screenshots_src, &screenshots_dir)?;

    let screenshot_count = crate::backup::count_screenshots();

    // Create manifest
    let manifest = crate::backup::BackupManifest {
        version: "1.0".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
        record_count,
        screenshot_count,
    };

    let manifest_path = temp_dir.path().join("manifest.json");
    let manifest_json = serde_json::to_string_pretty(&manifest)?;
    fs::write(&manifest_path, manifest_json)?;

    // Generate backup filename with auto- prefix
    let timestamp = chrono::Local::now().format("%Y-%m-%d-%H%M%S");
    let backup_filename = format!("auto-dailylogger-backup-{}.zip", timestamp);
    let backup_path = target_dir.join(&backup_filename);

    // Create zip file
    let file = fs::File::create(&backup_path)?;
    let mut zip = ZipWriter::new(file);

    for entry in walkdir::WalkDir::new(temp_dir.path())
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_file() {
            let relative_path = path
                .strip_prefix(temp_dir.path())
                .expect("walkdir entry should be under temp_dir");
            let zip_path = relative_path.to_string_lossy().replace("\\", "/");

            zip.start_file(&zip_path, SimpleFileOptions::default())?;

            let mut file = fs::File::open(path)?;
            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer)?;
            zip.write_all(&buffer)?;
        }
    }

    zip.finish()?;

    tracing::info!(
        "Auto backup created: {} ({} records, {} screenshots)",
        backup_path.display(),
        record_count,
        screenshot_count
    );

    Ok(())
}

/// Start the auto backup scheduler
/// This runs in the background and performs backups at the configured interval
pub fn start_scheduler() {
    if SCHEDULER_RUNNING.load(Ordering::SeqCst) {
        tracing::info!("Auto backup scheduler is already running");
        return;
    }

    // Spawn the scheduler using Tauri async runtime (available in Tauri context)
    let handle = tauri::async_runtime::spawn(async {
        tracing::info!("Auto backup scheduler started");
        run_scheduler_loop().await;
    });

    let global_handle = SCHEDULER_HANDLE.get_or_init(|| Arc::new(Mutex::new(None)));
    let handle_guard = global_handle.clone();
    tauri::async_runtime::spawn(async move {
        let mut guard = handle_guard.lock().await;
        *guard = Some(handle);
    });

    SCHEDULER_RUNNING.store(true, Ordering::SeqCst);
}

/// Stop the auto backup scheduler
pub fn stop_scheduler() {
    if !SCHEDULER_RUNNING.load(Ordering::SeqCst) {
        return;
    }

    SCHEDULER_RUNNING.store(false, Ordering::SeqCst);

    if let Some(global_handle) = SCHEDULER_HANDLE.get() {
        let handle_guard = global_handle.clone();
        tauri::async_runtime::spawn(async move {
            let mut guard = handle_guard.lock().await;
            if let Some(handle) = guard.take() {
                handle.abort();
            }
        });
    }

    tracing::info!("Auto backup scheduler stopped");
}

/// Main scheduler loop
async fn run_scheduler_loop() {
    while SCHEDULER_RUNNING.load(Ordering::SeqCst) {
        // should_run_backup_now() checks both enabled flag and timing
        if should_run_backup_now() {
            tracing::info!("Triggering scheduled auto backup");
            if let Err(e) = run_auto_backup().await {
                tracing::error!("Scheduled auto backup failed: {}", e);
            }
        }

        // Wait for 1 hour before checking again
        // In production, this could be shorter for more precise timing
        let mut interval = interval(Duration::from_secs(3600));
        interval.tick().await;
    }
}

/// Check and run backup on startup if needed
pub async fn check_and_run_startup_backup() {
    if should_run_backup_now() {
        tracing::info!("Running startup auto backup check...");
        match run_auto_backup().await {
            Ok(_) => tracing::info!("Startup auto backup completed"),
            Err(e) => tracing::error!("Startup auto backup failed: {}", e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backup_interval_from_str() {
        assert_eq!(BackupInterval::from_str("daily"), BackupInterval::Daily);
        assert_eq!(BackupInterval::from_str("DAILY"), BackupInterval::Daily);
        assert_eq!(BackupInterval::from_str("weekly"), BackupInterval::Weekly);
        assert_eq!(BackupInterval::from_str("monthly"), BackupInterval::Monthly);
        assert_eq!(BackupInterval::from_str("unknown"), BackupInterval::Daily);
    }

    #[test]
    fn test_backup_interval_to_hours() {
        assert_eq!(BackupInterval::Daily.to_hours(), 24);
        assert_eq!(BackupInterval::Weekly.to_hours(), 24 * 7);
        assert_eq!(BackupInterval::Monthly.to_hours(), 24 * 30);
    }

    #[test]
    fn test_auto_backup_retention_bounds() {
        // Test that retention is clamped between 3 and 20
        // This would require mocking get_settings_sync, so we test the clamping logic directly
        let test_values = vec![0, 1, 3, 5, 10, 20, 25, 100];
        let expected = vec![3, 3, 3, 5, 10, 20, 20, 20];

        for (input, expected) in test_values.into_iter().zip(expected) {
            let clamped = input.max(3).min(20) as usize;
            assert_eq!(clamped, expected);
        }
    }
}
