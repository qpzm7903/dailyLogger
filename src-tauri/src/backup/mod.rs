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
    pub rolled_back: bool,
}

/// manifest.json 结构
#[derive(Debug, Serialize, Deserialize)]
struct BackupManifest {
    version: String,
    created_at: String,
    record_count: usize,
    screenshot_count: usize,
}

fn get_app_data_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("DailyLogger")
}

fn get_db_path() -> PathBuf {
    get_app_data_dir().join("data").join("local.db")
}

fn get_screenshots_dir() -> PathBuf {
    get_app_data_dir().join("screenshots")
}

fn get_default_backup_dir() -> PathBuf {
    dirs::document_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("DailyLogger")
        .join("backups")
}

fn get_record_count() -> Result<usize, String> {
    use crate::memory_storage::DB_CONNECTION;

    let guard = DB_CONNECTION.lock().map_err(|e| e.to_string())?;
    let conn = guard.as_ref().ok_or("Database not initialized")?;

    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM records", [], |row| row.get(0))
        .map_err(|e| format!("Failed to count records: {}", e))?;

    Ok(count as usize)
}

fn count_screenshots() -> usize {
    let screenshots_dir = get_screenshots_dir();
    if !screenshots_dir.exists() {
        return 0;
    }

    fs::read_dir(&screenshots_dir)
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .filter(|e| {
                    e.path()
                        .extension()
                        .map(|ext| ext == "png")
                        .unwrap_or(false)
                })
                .count()
        })
        .unwrap_or(0)
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

    // 复制数据库文件
    let db_path = get_db_path();
    if db_path.exists() {
        fs::copy(&db_path, data_dir.join("local.db"))
            .map_err(|e| format!("Failed to copy database: {}", e))?;
    }

    // 复制截图文件
    let screenshots_src = get_screenshots_dir();
    if screenshots_src.exists() {
        for entry in fs::read_dir(&screenshots_src).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let src_path = entry.path();

            if src_path.is_file() {
                let file_name = src_path.file_name().unwrap();
                fs::copy(&src_path, screenshots_dir.join(file_name))
                    .map_err(|e| format!("Failed to copy screenshot: {}", e))?;
            }
        }
    }

    // 获取统计信息
    let record_count = get_record_count().unwrap_or(0);
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

    if !path.exists() {
        return Err("Backup file not found".to_string());
    }

    let file = fs::File::open(&path).map_err(|e| e.to_string())?;
    let mut archive = ZipArchive::new(file).map_err(|e| e.to_string())?;

    // 读取 manifest
    let manifest: BackupManifest = {
        let mut manifest_file = archive
            .by_name("manifest.json")
            .map_err(|e| format!("Invalid backup file: {}", e))?;
        let mut content = String::new();
        manifest_file
            .read_to_string(&mut content)
            .map_err(|e| e.to_string())?;
        serde_json::from_str(&content).map_err(|e| format!("Invalid manifest: {}", e))?
    };

    let metadata = fs::metadata(&path).map_err(|e| e.to_string())?;

    Ok(BackupInfo {
        path: backup_path,
        created_at: manifest.created_at,
        size_bytes: metadata.len(),
        record_count: manifest.record_count,
        screenshot_count: manifest.screenshot_count,
    })
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
    let file = fs::File::open(path).map_err(|e| e.to_string())?;
    let mut archive = ZipArchive::new(file).map_err(|e| e.to_string())?;

    let manifest: BackupManifest = {
        let mut manifest_file = archive
            .by_name("manifest.json")
            .map_err(|e| format!("Invalid backup file: {}", e))?;
        let mut content = String::new();
        manifest_file
            .read_to_string(&mut content)
            .map_err(|e| e.to_string())?;
        serde_json::from_str(&content).map_err(|e| format!("Invalid manifest: {}", e))?
    };

    let metadata = fs::metadata(path).map_err(|e| e.to_string())?;

    Ok(BackupInfo {
        path: path.to_string_lossy().to_string(),
        created_at: manifest.created_at,
        size_bytes: metadata.len(),
        record_count: manifest.record_count,
        screenshot_count: manifest.screenshot_count,
    })
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

/// 恢复备份
#[tauri::command]
pub async fn restore_backup(backup_path: String) -> Result<RestoreResult, String> {
    let path = PathBuf::from(&backup_path);

    if !path.exists() {
        return Err("Backup file not found".to_string());
    }

    // 打开备份文件
    let file = fs::File::open(&path).map_err(|e| e.to_string())?;
    let mut archive = ZipArchive::new(file).map_err(|e| e.to_string())?;

    // 读取 manifest 获取统计信息
    let manifest: BackupManifest = {
        let mut manifest_file = archive
            .by_name("manifest.json")
            .map_err(|e| format!("Invalid backup file: {}", e))?;
        let mut content = String::new();
        manifest_file
            .read_to_string(&mut content)
            .map_err(|e| e.to_string())?;
        serde_json::from_str(&content).map_err(|e| format!("Invalid manifest: {}", e))?
    };

    // 创建临时备份目录（用于恢复失败时回滚）
    let rollback_dir = get_app_data_dir().join("temp-rollback");
    let current_db = get_db_path();
    let current_screenshots = get_screenshots_dir();

    let mut rolled_back = false;

    // 如果当前数据存在，先备份
    if current_db.exists() || current_screenshots.exists() {
        let rollback_db = rollback_dir.join("data");
        let rollback_screenshots = rollback_dir.join("screenshots");

        fs::create_dir_all(&rollback_db).map_err(|e| e.to_string())?;
        fs::create_dir_all(&rollback_screenshots).map_err(|e| e.to_string())?;

        if current_db.exists() {
            fs::copy(&current_db, rollback_db.join("local.db"))
                .map_err(|e| format!("Failed to backup current database: {}", e))?;
        }

        if current_screenshots.exists() {
            for entry in fs::read_dir(&current_screenshots).map_err(|e| e.to_string())? {
                let entry = entry.map_err(|e| e.to_string())?;
                let src_path = entry.path();
                if src_path.is_file() {
                    let file_name = src_path.file_name().unwrap();
                    fs::copy(&src_path, rollback_screenshots.join(file_name))
                        .map_err(|e| format!("Failed to backup screenshots: {}", e))?;
                }
            }
        }

        rolled_back = true;
    }

    // 解压备份文件
    let temp_extract = tempfile::Builder::new()
        .prefix("dailylogger-restore-")
        .tempdir()
        .map_err(|e| format!("Failed to create temp directory: {}", e))?;

    archive
        .extract(temp_extract.path())
        .map_err(|e| format!("Failed to extract backup: {}", e))?;

    // 恢复数据
    let extracted_data_dir = temp_extract.path().join("data");
    let extracted_screenshots_dir = temp_extract.path().join("screenshots");

    // 恢复数据库
    let target_data_dir = get_app_data_dir().join("data");
    fs::create_dir_all(&target_data_dir).map_err(|e| e.to_string())?;

    if extracted_data_dir.join("local.db").exists() {
        fs::copy(
            extracted_data_dir.join("local.db"),
            target_data_dir.join("local.db"),
        )
        .map_err(|e| format!("Failed to restore database: {}", e))?;
    }

    // 恢复截图
    fs::create_dir_all(&current_screenshots).map_err(|e| e.to_string())?;

    if extracted_screenshots_dir.exists() {
        for entry in fs::read_dir(&extracted_screenshots_dir).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let src_path = entry.path();
            if src_path.is_file() {
                let file_name = src_path.file_name().unwrap();
                fs::copy(&src_path, current_screenshots.join(file_name))
                    .map_err(|e| format!("Failed to restore screenshot: {}", e))?;
            }
        }
    }

    // 清理临时提取目录
    drop(temp_extract);

    // 清理回滚目录（成功恢复后）
    if rolled_back {
        let _ = fs::remove_dir_all(&rollback_dir);
    }

    Ok(RestoreResult {
        success: true,
        record_count: manifest.record_count,
        screenshot_count: manifest.screenshot_count,
        rolled_back,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_app_data_dir() {
        let dir = get_app_data_dir();
        assert!(dir.to_string_lossy().contains("DailyLogger"));
    }

    #[test]
    fn test_get_default_backup_dir() {
        let dir = get_default_backup_dir();
        assert!(dir.to_string_lossy().contains("DailyLogger"));
        assert!(dir.to_string_lossy().contains("backups"));
    }
}
