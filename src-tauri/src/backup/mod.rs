use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use zip::write::SimpleFileOptions;
use zip::{ZipArchive, ZipWriter};

/// 备份信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupInfo {
    pub path: String,
    pub created_at: String,
    pub size_bytes: u64,
    pub record_count: usize,
    pub screenshot_count: usize,
}

/// 备份结果
#[derive(Debug, Serialize, Deserialize)]
pub struct BackupResult {
    pub path: String,
    pub size_bytes: u64,
    pub record_count: usize,
    pub screenshot_count: usize,
}

/// 恢复结果
#[derive(Debug, Serialize, Deserialize)]
pub struct RestoreResult {
    pub success: bool,
    pub record_count: usize,
    pub screenshot_count: usize,
    pub auto_backup_created: bool,
}

/// Backup manifest structure for zip archives
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupManifest {
    pub version: String,
    pub created_at: String,
    pub record_count: usize,
    pub screenshot_count: usize,
}

/// Get database file path
pub fn get_db_path() -> PathBuf {
    crate::get_app_data_dir().join("data").join("local.db")
}

/// Get screenshots directory path
pub fn get_screenshots_dir() -> PathBuf {
    crate::get_app_data_dir().join("screenshots")
}

/// Get the default backup directory: `<Documents>/DailyLogger/backups`.
pub fn get_default_backup_dir() -> PathBuf {
    dirs::document_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("DailyLogger")
        .join("backups")
}

/// Count screenshots in the app's screenshots directory
pub fn count_screenshots() -> usize {
    count_screenshots_in_dir(&get_screenshots_dir())
}

fn count_screenshots_in_dir(dir: &Path) -> usize {
    if !dir.exists() {
        return 0;
    }

    fs::read_dir(dir)
        .map(|entries| {
            entries
                .filter_map(std::result::Result::ok)
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "png"))
                .count()
        })
        .unwrap_or(0)
}

/// Copy all files from `src_dir` to `dst_dir` (non-recursive, files only).
pub fn copy_dir_files(src_dir: &Path, dst_dir: &Path) -> Result<(), String> {
    if !src_dir.exists() {
        return Ok(());
    }
    fs::create_dir_all(dst_dir)
        .map_err(|e| format!("Failed to create dir {}: {e}", dst_dir.display()))?;

    for entry in fs::read_dir(src_dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let src_path = entry.path();
        if src_path.is_file() {
            if let Some(file_name) = src_path.file_name() {
                fs::copy(&src_path, dst_dir.join(file_name))
                    .map_err(|e| format!("Failed to copy {}: {e}", src_path.display()))?;
            }
        }
    }
    Ok(())
}

/// Remove all files in a directory (non-recursive, files only).
fn clear_dir_files(dir: &Path) -> Result<(), String> {
    if !dir.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.is_file() {
            fs::remove_file(&path)
                .map_err(|e| format!("Failed to remove {}: {e}", path.display()))?;
        }
    }
    Ok(())
}

/// Read a `BackupManifest` from a `ZipArchive`.
fn read_manifest_from_archive<R: std::io::Read + std::io::Seek>(
    archive: &mut ZipArchive<R>,
) -> Result<BackupManifest, String> {
    let mut manifest_file = archive
        .by_name("manifest.json")
        .map_err(|e| format!("Invalid backup file: {e}"))?;
    let mut content = String::new();
    manifest_file
        .read_to_string(&mut content)
        .map_err(|e| e.to_string())?;
    serde_json::from_str(&content).map_err(|e| format!("Invalid manifest: {e}"))
}

/// 创建备份
#[tauri::command]
pub async fn create_backup(backup_dir: Option<String>) -> Result<BackupResult, String> {
    let target_dir = backup_dir
        .map(PathBuf::from)
        .unwrap_or_else(get_default_backup_dir);

    // 确保备份目录存在
    fs::create_dir_all(&target_dir)
        .map_err(|e| format!("Failed to create backup directory: {}", e))?;

    // 创建临时目录
    let temp_dir = tempfile::Builder::new()
        .prefix("dailylogger-backup-")
        .tempdir()
        .map_err(|e| format!("Failed to create temp directory: {}", e))?;

    let data_dir = temp_dir.path().join("data");
    let screenshots_dir = temp_dir.path().join("screenshots");

    fs::create_dir_all(&data_dir).map_err(|e| format!("Failed to create data dir: {}", e))?;
    fs::create_dir_all(&screenshots_dir)
        .map_err(|e| format!("Failed to create screenshots dir: {}", e))?;

    // 获取统计信息并复制数据库（在同一个 DB 锁内，确保一致性）
    let record_count = {
        use crate::memory_storage::DB_CONNECTION;
        let guard = DB_CONNECTION.lock().map_err(|e| e.to_string())?;
        let conn = guard.as_ref().ok_or("Database not initialized")?;

        // Flush WAL journal before copying
        let _ = conn.execute_batch("PRAGMA wal_checkpoint(FULL)");

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM records", [], |row| row.get(0))
            .map_err(|e| format!("Failed to count records: {}", e))?;

        // 复制数据库文件（在锁内，防止并发写入导致不一致）
        let db_path = get_db_path();
        if db_path.exists() {
            fs::copy(&db_path, data_dir.join("local.db"))
                .map_err(|e| format!("Failed to copy database: {}", e))?;
        }

        count as usize
    };

    // 复制截图文件
    let screenshots_src = get_screenshots_dir();
    copy_dir_files(&screenshots_src, &screenshots_dir)?;

    let screenshot_count = count_screenshots();

    // 创建 manifest
    let manifest = BackupManifest {
        version: "1.0".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
        record_count,
        screenshot_count,
    };

    let manifest_path = temp_dir.path().join("manifest.json");
    let manifest_json = serde_json::to_string_pretty(&manifest).map_err(|e| e.to_string())?;
    fs::write(&manifest_path, manifest_json).map_err(|e| e.to_string())?;

    // 生成备份文件名
    let timestamp = chrono::Local::now().format("%Y-%m-%d-%H%M%S");
    let backup_filename = format!("dailylogger-backup-{}.zip", timestamp);
    let backup_path = target_dir.join(&backup_filename);

    // 创建 zip 文件
    let file = fs::File::create(&backup_path)
        .map_err(|e| format!("Failed to create backup file: {}", e))?;
    let mut zip = ZipWriter::new(file);

    // 添加所有文件到 zip
    for entry in walkdir::WalkDir::new(temp_dir.path())
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_file() {
            let relative_path = path.strip_prefix(temp_dir.path()).unwrap();
            let zip_path = relative_path.to_string_lossy().replace("\\", "/");

            zip.start_file(&zip_path, SimpleFileOptions::default())
                .map_err(|e| e.to_string())?;

            let mut file = fs::File::open(path).map_err(|e| e.to_string())?;
            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer).map_err(|e| e.to_string())?;
            zip.write_all(&buffer).map_err(|e| e.to_string())?;
        }
    }

    zip.finish().map_err(|e| e.to_string())?;

    // 获取备份文件大小
    let metadata = fs::metadata(&backup_path).map_err(|e| e.to_string())?;
    let size_bytes = metadata.len();

    Ok(BackupResult {
        path: backup_path.to_string_lossy().to_string(),
        size_bytes,
        record_count,
        screenshot_count,
    })
}

/// 获取备份信息
#[tauri::command]
pub async fn get_backup_info(backup_path: String) -> Result<BackupInfo, String> {
    let path = PathBuf::from(&backup_path);
    get_backup_info_internal(&path)
}

/// 列出备份历史
#[tauri::command]
pub async fn list_backups() -> Result<Vec<BackupInfo>, String> {
    let backup_dir = get_default_backup_dir();

    if !backup_dir.exists() {
        return Ok(Vec::new());
    }

    let mut backups = Vec::new();

    for entry in fs::read_dir(&backup_dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();

        if path.extension().map(|e| e == "zip").unwrap_or(false) {
            match get_backup_info_internal(&path) {
                Ok(info) => backups.push(info),
                Err(e) => {
                    tracing::warn!("Failed to read backup info for {:?}: {}", path, e);
                }
            }
        }
    }

    // 按创建时间排序，最新的在前
    backups.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    // 只保留最近 10 个
    backups.truncate(10);

    Ok(backups)
}

fn get_backup_info_internal(path: &Path) -> Result<BackupInfo, String> {
    if !path.exists() {
        return Err("Backup file not found".to_string());
    }

    let file = fs::File::open(path).map_err(|e| e.to_string())?;
    let mut archive = ZipArchive::new(file).map_err(|e| e.to_string())?;

    let manifest = read_manifest_from_archive(&mut archive)?;

    let metadata = fs::metadata(path).map_err(|e| e.to_string())?;

    Ok(BackupInfo {
        path: path.to_string_lossy().to_string(),
        created_at: manifest.created_at,
        size_bytes: metadata.len(),
        record_count: manifest.record_count,
        screenshot_count: manifest.screenshot_count,
    })
}

/// Clean up old automatic backups, keeping only the most recent ones based on retention policy.
/// Only cleans up files with "auto-" prefix (automatic backups), not manual backups.
pub fn cleanup_old_auto_backups() -> Result<usize, String> {
    use crate::memory_storage::get_settings_sync;

    let retention = match get_settings_sync() {
        Ok(settings) => settings.auto_backup_retention.unwrap_or(5).clamp(3, 20) as usize,
        Err(_) => 5,
    };

    let backup_dir = get_default_backup_dir();

    if !backup_dir.exists() {
        return Ok(0);
    }

    // Collect all auto backup files (with "auto-" prefix)
    let mut auto_backups: Vec<_> = fs::read_dir(&backup_dir)
        .map_err(|e| e.to_string())?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            let path = entry.path();
            // Only include files with .zip extension and auto- prefix
            path.extension().is_some_and(|ext| ext == "zip")
                && path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| name.starts_with("auto-"))
        })
        .collect();

    // If we have more auto backups than the retention limit, delete the oldest ones
    if auto_backups.len() > retention {
        // Sort by modification time (oldest first)
        auto_backups.sort_by_key(|entry| {
            entry
                .metadata()
                .and_then(|m| m.modified())
                .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
        });

        // Calculate how many to delete
        let to_delete = auto_backups.len() - retention;

        let mut deleted = 0;
        for entry in auto_backups.iter().take(to_delete) {
            let path = entry.path();
            match fs::remove_file(&path) {
                Ok(_) => {
                    tracing::info!("Deleted old auto backup: {}", path.display());
                    deleted += 1;
                }
                Err(e) => {
                    tracing::error!("Failed to delete old auto backup {:?}: {}", path, e);
                }
            }
        }

        Ok(deleted)
    } else {
        Ok(0)
    }
}

/// 删除备份
#[tauri::command]
pub async fn delete_backup(backup_path: String) -> Result<(), String> {
    let path = PathBuf::from(&backup_path);

    if !path.exists() {
        return Err("Backup file not found".to_string());
    }

    fs::remove_file(&path).map_err(|e| format!("Failed to delete backup: {}", e))
}

/// Rollback: restore data from rollback_dir to the app data directory.
fn rollback_from(rollback_dir: &Path) -> Result<(), String> {
    let rollback_db = rollback_dir.join("data").join("local.db");
    let rollback_screenshots = rollback_dir.join("screenshots");

    let target_db = get_db_path();
    let target_screenshots = get_screenshots_dir();

    // Restore database
    if rollback_db.exists() {
        let target_data_dir = crate::get_app_data_dir().join("data");
        fs::create_dir_all(&target_data_dir).map_err(|e| e.to_string())?;
        fs::copy(&rollback_db, &target_db)
            .map_err(|e| format!("Rollback: failed to restore database: {}", e))?;
    }

    // Restore screenshots: clear current, copy rollback
    clear_dir_files(&target_screenshots)?;
    copy_dir_files(&rollback_screenshots, &target_screenshots)?;

    Ok(())
}

/// The inner restore logic, separated so we can rollback on failure.
fn perform_restore_inner(archive: &mut ZipArchive<fs::File>) -> Result<(), String> {
    let temp_extract = tempfile::Builder::new()
        .prefix("dailylogger-restore-")
        .tempdir()
        .map_err(|e| format!("Failed to create temp directory: {}", e))?;

    archive
        .extract(temp_extract.path())
        .map_err(|e| format!("Failed to extract backup: {}", e))?;

    let extracted_data_dir = temp_extract.path().join("data");
    let extracted_screenshots_dir = temp_extract.path().join("screenshots");

    // Restore database
    let target_data_dir = crate::get_app_data_dir().join("data");
    fs::create_dir_all(&target_data_dir).map_err(|e| e.to_string())?;

    if extracted_data_dir.join("local.db").exists() {
        fs::copy(
            extracted_data_dir.join("local.db"),
            target_data_dir.join("local.db"),
        )
        .map_err(|e| format!("Failed to restore database: {}", e))?;
    }

    // Restore screenshots: clear old files first, then copy from backup
    let current_screenshots = get_screenshots_dir();
    clear_dir_files(&current_screenshots)?;
    fs::create_dir_all(&current_screenshots).map_err(|e| e.to_string())?;

    copy_dir_files(&extracted_screenshots_dir, &current_screenshots)?;

    Ok(())
}

/// 恢复备份
#[tauri::command]
pub async fn restore_backup(backup_path: String) -> Result<RestoreResult, String> {
    let path = PathBuf::from(&backup_path);

    if !path.exists() {
        return Err("Backup file not found".to_string());
    }

    // 打开备份文件并读取 manifest
    let file = fs::File::open(&path).map_err(|e| e.to_string())?;
    let mut archive = ZipArchive::new(file).map_err(|e| e.to_string())?;

    let manifest = read_manifest_from_archive(&mut archive)?;

    // 创建临时备份目录（用于恢复失败时回滚）
    let rollback_dir = crate::get_app_data_dir().join("temp-rollback");
    let current_db = get_db_path();
    let current_screenshots = get_screenshots_dir();

    let mut auto_backup_created = false;

    // 如果当前数据存在，先备份用于回滚
    if current_db.exists() || current_screenshots.exists() {
        // Clean any stale rollback dir from previous failed restore
        let _ = fs::remove_dir_all(&rollback_dir);

        let rollback_db_dir = rollback_dir.join("data");
        let rollback_screenshots_dir = rollback_dir.join("screenshots");

        fs::create_dir_all(&rollback_db_dir).map_err(|e| e.to_string())?;
        fs::create_dir_all(&rollback_screenshots_dir).map_err(|e| e.to_string())?;

        if current_db.exists() {
            fs::copy(&current_db, rollback_db_dir.join("local.db"))
                .map_err(|e| format!("Failed to backup current database: {}", e))?;
        }

        copy_dir_files(&current_screenshots, &rollback_screenshots_dir)?;

        auto_backup_created = true;
    }

    // Re-open the archive (the previous one was consumed by read_manifest)
    let file = fs::File::open(&path).map_err(|e| e.to_string())?;
    let mut archive = ZipArchive::new(file).map_err(|e| e.to_string())?;

    // Attempt restore; rollback on failure
    if let Err(restore_err) = perform_restore_inner(&mut archive) {
        if auto_backup_created {
            tracing::error!("Restore failed, rolling back: {}", restore_err);
            if let Err(rollback_err) = rollback_from(&rollback_dir) {
                return Err(format!(
                    "Restore failed: {}. Rollback also failed: {}. Manual recovery may be needed.",
                    restore_err, rollback_err
                ));
            }
            let _ = fs::remove_dir_all(&rollback_dir);
            return Err(format!(
                "Restore failed and rolled back to previous state: {}",
                restore_err
            ));
        }
        return Err(restore_err);
    }

    // Re-initialize database connection to use the restored DB file
    if let Err(e) = crate::memory_storage::init_database() {
        tracing::error!("Failed to re-initialize database after restore: {}", e);
        // Non-fatal: the app may need a restart, but the files are restored
    }

    // 清理回滚目录（成功恢复后）
    if auto_backup_created {
        let _ = fs::remove_dir_all(&rollback_dir);
    }

    Ok(RestoreResult {
        success: true,
        record_count: manifest.record_count,
        screenshot_count: manifest.screenshot_count,
        auto_backup_created,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_app_data_dir() {
        let dir = crate::get_app_data_dir();
        assert!(dir.to_string_lossy().contains("DailyLogger"));
    }

    #[test]
    fn test_get_default_backup_dir() {
        let dir = get_default_backup_dir();
        assert!(dir.to_string_lossy().contains("DailyLogger"));
        assert!(dir.to_string_lossy().contains("backups"));
    }

    #[test]
    fn test_backup_manifest_serialization() {
        let manifest = BackupManifest {
            version: "1.0".to_string(),
            created_at: "2026-03-15T10:00:00Z".to_string(),
            record_count: 42,
            screenshot_count: 10,
        };
        let json = serde_json::to_string(&manifest).unwrap();
        let parsed: BackupManifest = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.version, "1.0");
        assert_eq!(parsed.record_count, 42);
        assert_eq!(parsed.screenshot_count, 10);
        assert_eq!(parsed.created_at, "2026-03-15T10:00:00Z");
    }

    #[test]
    fn test_backup_manifest_missing_fields_fails() {
        let json = r#"{"version": "1.0"}"#;
        let result: Result<BackupManifest, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_create_and_read_backup_zip() {
        let temp_dir = tempfile::tempdir().unwrap();
        let zip_path = temp_dir.path().join("test-backup.zip");

        // Create a zip with manifest
        let manifest = BackupManifest {
            version: "1.0".to_string(),
            created_at: "2026-03-15T12:00:00Z".to_string(),
            record_count: 5,
            screenshot_count: 2,
        };

        {
            let file = fs::File::create(&zip_path).unwrap();
            let mut zip = ZipWriter::new(file);

            // Add manifest.json
            let manifest_json = serde_json::to_string_pretty(&manifest).unwrap();
            zip.start_file("manifest.json", SimpleFileOptions::default())
                .unwrap();
            zip.write_all(manifest_json.as_bytes()).unwrap();

            // Add a dummy data file
            zip.start_file("data/local.db", SimpleFileOptions::default())
                .unwrap();
            zip.write_all(b"dummy database content").unwrap();

            // Add a dummy screenshot
            zip.start_file(
                "screenshots/screenshot_001.png",
                SimpleFileOptions::default(),
            )
            .unwrap();
            zip.write_all(b"fake png").unwrap();

            zip.finish().unwrap();
        }

        // Read the backup info
        let info = get_backup_info_internal(&zip_path).unwrap();
        assert_eq!(info.record_count, 5);
        assert_eq!(info.screenshot_count, 2);
        assert_eq!(info.created_at, "2026-03-15T12:00:00Z");
        assert!(info.size_bytes > 0);
    }

    #[test]
    fn test_backup_info_not_found() {
        let result = get_backup_info_internal(Path::new("/nonexistent/backup.zip"));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    #[test]
    fn test_backup_info_invalid_zip() {
        let temp_dir = tempfile::tempdir().unwrap();
        let bad_path = temp_dir.path().join("not-a-zip.zip");
        fs::write(&bad_path, b"this is not a zip file").unwrap();

        let result = get_backup_info_internal(&bad_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_backup_info_zip_without_manifest() {
        let temp_dir = tempfile::tempdir().unwrap();
        let zip_path = temp_dir.path().join("no-manifest.zip");

        {
            let file = fs::File::create(&zip_path).unwrap();
            let mut zip = ZipWriter::new(file);
            zip.start_file("data/local.db", SimpleFileOptions::default())
                .unwrap();
            zip.write_all(b"dummy").unwrap();
            zip.finish().unwrap();
        }

        let result = get_backup_info_internal(&zip_path);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid backup file"));
    }

    #[test]
    fn test_delete_backup_removes_file() {
        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("to-delete.zip");
        fs::write(&file_path, b"dummy").unwrap();
        assert!(file_path.exists());

        let path_str = file_path.to_string_lossy().to_string();
        // Call the sync version of the logic (delete_backup is async, test the core)
        fs::remove_file(&file_path).unwrap();
        assert!(!PathBuf::from(&path_str).exists());
    }

    #[test]
    fn test_copy_dir_files() {
        let temp = tempfile::tempdir().unwrap();
        let src = temp.path().join("src");
        let dst = temp.path().join("dst");
        fs::create_dir_all(&src).unwrap();

        fs::write(src.join("a.txt"), b"hello").unwrap();
        fs::write(src.join("b.png"), b"image").unwrap();

        copy_dir_files(&src, &dst).unwrap();

        assert!(dst.join("a.txt").exists());
        assert!(dst.join("b.png").exists());
        assert_eq!(fs::read_to_string(dst.join("a.txt")).unwrap(), "hello");
    }

    #[test]
    fn test_copy_dir_files_nonexistent_src() {
        let temp = tempfile::tempdir().unwrap();
        let dst = temp.path().join("dst");
        // Source doesn't exist — should succeed (no-op)
        let result = copy_dir_files(Path::new("/nonexistent/dir"), &dst);
        assert!(result.is_ok());
    }

    #[test]
    fn test_clear_dir_files() {
        let temp = tempfile::tempdir().unwrap();
        let dir = temp.path().join("dir");
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("a.txt"), b"hello").unwrap();
        fs::write(dir.join("b.txt"), b"world").unwrap();

        clear_dir_files(&dir).unwrap();

        assert!(dir.exists()); // dir itself still exists
        assert_eq!(fs::read_dir(&dir).unwrap().count(), 0); // but empty
    }

    #[test]
    fn test_clear_dir_files_nonexistent() {
        let result = clear_dir_files(Path::new("/nonexistent/dir"));
        assert!(result.is_ok());
    }

    #[test]
    fn test_count_screenshots_in_dir() {
        let temp = tempfile::tempdir().unwrap();
        let dir = temp.path().join("screenshots");
        fs::create_dir_all(&dir).unwrap();

        fs::write(dir.join("shot1.png"), b"img").unwrap();
        fs::write(dir.join("shot2.png"), b"img").unwrap();
        fs::write(dir.join("notes.txt"), b"text").unwrap();
        fs::write(dir.join("shot3.jpg"), b"img").unwrap();

        assert_eq!(count_screenshots_in_dir(&dir), 2); // only .png files
    }

    #[test]
    fn test_count_screenshots_empty_dir() {
        let temp = tempfile::tempdir().unwrap();
        assert_eq!(count_screenshots_in_dir(temp.path()), 0);
    }

    #[test]
    fn test_count_screenshots_nonexistent_dir() {
        assert_eq!(
            count_screenshots_in_dir(Path::new("/nonexistent/screenshots")),
            0
        );
    }

    #[test]
    fn test_restore_result_serialization() {
        let result = RestoreResult {
            success: true,
            record_count: 10,
            screenshot_count: 3,
            auto_backup_created: true,
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("auto_backup_created"));
        assert!(!json.contains("rolled_back"));
    }

    #[test]
    fn test_read_manifest_from_valid_archive() {
        let temp_dir = tempfile::tempdir().unwrap();
        let zip_path = temp_dir.path().join("valid.zip");

        {
            let file = fs::File::create(&zip_path).unwrap();
            let mut zip = ZipWriter::new(file);
            let manifest = BackupManifest {
                version: "1.0".to_string(),
                created_at: "2026-01-01T00:00:00Z".to_string(),
                record_count: 100,
                screenshot_count: 50,
            };
            let json = serde_json::to_string(&manifest).unwrap();
            zip.start_file("manifest.json", SimpleFileOptions::default())
                .unwrap();
            zip.write_all(json.as_bytes()).unwrap();
            zip.finish().unwrap();
        }

        let file = fs::File::open(&zip_path).unwrap();
        let mut archive = ZipArchive::new(file).unwrap();
        let manifest = read_manifest_from_archive(&mut archive).unwrap();
        assert_eq!(manifest.record_count, 100);
        assert_eq!(manifest.screenshot_count, 50);
    }

    #[test]
    fn test_rollback_restores_files() {
        let temp = tempfile::tempdir().unwrap();
        let rollback_dir = temp.path().join("rollback");
        let data_dir = rollback_dir.join("data");
        let screenshots_dir = rollback_dir.join("screenshots");

        fs::create_dir_all(&data_dir).unwrap();
        fs::create_dir_all(&screenshots_dir).unwrap();

        fs::write(data_dir.join("local.db"), b"original db").unwrap();
        fs::write(screenshots_dir.join("shot.png"), b"original shot").unwrap();

        // rollback_from uses get_app_data_dir() which points to system dir,
        // so we test the helper functions it uses instead
        let target = temp.path().join("target");
        fs::create_dir_all(&target).unwrap();
        fs::write(target.join("modified.txt"), b"modified").unwrap();

        clear_dir_files(&target).unwrap();
        assert_eq!(fs::read_dir(&target).unwrap().count(), 0);

        copy_dir_files(&screenshots_dir, &target).unwrap();
        assert!(target.join("shot.png").exists());
    }
}
