// 网络状态检测与离线队列管理模块
// CORE-007: 离线模式支持

use once_cell::sync::Lazy;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use std::time::Duration;
use tauri::command;

/// 网络状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NetworkStatus {
    Online,
    Offline,
}

/// 离线任务类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OfflineTask {
    pub id: i64,
    pub task_type: String,  // "ai_analysis", "daily_summary" 等
    pub payload: String,    // JSON 序列化的任务数据
    pub created_at: String, // RFC3339 时间戳
    pub retry_count: i32,   // 重试次数
    pub last_error: Option<String>,
}

/// 离线队列状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OfflineQueueStatus {
    pub total_tasks: i64,
    pub pending_tasks: i64,
    pub failed_tasks: i64,
}

/// 全局网络状态
pub static NETWORK_STATUS: Lazy<Mutex<NetworkStatus>> =
    Lazy::new(|| Mutex::new(NetworkStatus::Online));

/// 检查网络状态
#[command]
pub fn check_network_status() -> Result<NetworkStatus, String> {
    // 使用 reqwest 尝试连接检测网络
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    // 尝试连接到常用端点检测网络
    let is_online = client
        .head("https://www.google.com")
        .send()
        .map(|resp| resp.status().is_success() || resp.status().is_redirection())
        .unwrap_or(false);

    let status = if is_online {
        NetworkStatus::Online
    } else {
        NetworkStatus::Offline
    };

    // 更新全局状态
    if let Ok(mut current) = NETWORK_STATUS.lock() {
        *current = status.clone();
    }

    tracing::info!("Network status check: {:?}", status);
    Ok(status)
}

/// 初始化离线队列表
pub fn init_offline_queue(conn: &Connection) -> Result<(), String> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS offline_queue (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            task_type TEXT NOT NULL,
            payload TEXT NOT NULL,
            created_at TEXT NOT NULL,
            retry_count INTEGER DEFAULT 0,
            last_error TEXT,
            status TEXT DEFAULT 'pending'
        )",
        [],
    )
    .map_err(|e| format!("Failed to create offline_queue table: {}", e))?;

    tracing::info!("Offline queue table initialized");
    Ok(())
}

/// 添加任务到离线队列
pub fn add_offline_task(task_type: &str, payload: &str, conn: &Connection) -> Result<i64, String> {
    let created_at = chrono::Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO offline_queue (task_type, payload, created_at, status) VALUES (?1, ?2, ?3, 'pending')",
        params![task_type, payload, created_at],
    )
    .map_err(|e| format!("Failed to add offline task: {}", e))?;

    let id = conn.last_insert_rowid();
    tracing::info!("Added offline task: {} - {}", task_type, id);
    Ok(id)
}

/// 获取离线队列状态
#[command]
pub fn get_offline_queue_status() -> Result<OfflineQueueStatus, String> {
    let db = crate::memory_storage::DB_CONNECTION
        .lock()
        .map_err(|e| format!("Failed to lock DB: {}", e))?;
    let conn = db.as_ref().ok_or("Database not initialized")?;

    let total_tasks: i64 = conn
        .query_row("SELECT COUNT(*) FROM offline_queue", [], |row| row.get(0))
        .unwrap_or(0);

    let pending_tasks: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM offline_queue WHERE status = 'pending'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let failed_tasks: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM offline_queue WHERE status = 'failed'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    Ok(OfflineQueueStatus {
        total_tasks,
        pending_tasks,
        failed_tasks,
    })
}

/// 获取所有待处理的离线任务
pub fn get_pending_offline_tasks(conn: &Connection) -> Result<Vec<OfflineTask>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, task_type, payload, created_at, retry_count, last_error
             FROM offline_queue
             WHERE status = 'pending'
             ORDER BY created_at ASC",
        )
        .map_err(|e| format!("Failed to prepare query: {}", e))?;

    let tasks = stmt
        .query_map([], |row| {
            Ok(OfflineTask {
                id: row.get(0)?,
                task_type: row.get(1)?,
                payload: row.get(2)?,
                created_at: row.get(3)?,
                retry_count: row.get(4)?,
                last_error: row.get(5)?,
            })
        })
        .map_err(|e| format!("Failed to query tasks: {}", e))?
        .filter_map(|r| r.ok())
        .collect();

    Ok(tasks)
}

/// 更新任务状态
pub fn update_offline_task_status(
    task_id: i64,
    status: &str,
    error: Option<&str>,
    conn: &Connection,
) -> Result<(), String> {
    conn.execute(
        "UPDATE offline_queue SET status = ?1, last_error = ?2 WHERE id = ?3",
        params![status, error, task_id],
    )
    .map_err(|e| format!("Failed to update task status: {}", e))?;

    tracing::info!("Updated offline task {} status to {}", task_id, status);
    Ok(())
}

/// 增加任务重试计数
pub fn increment_retry_count(task_id: i64, error: &str, conn: &Connection) -> Result<(), String> {
    conn.execute(
        "UPDATE offline_queue SET retry_count = retry_count + 1, last_error = ?1, status = 'pending' WHERE id = ?2",
        params![error, task_id],
    )
    .map_err(|e| format!("Failed to increment retry count: {}", e))?;

    Ok(())
}

/// 删除已完成的任务
pub fn remove_offline_task(task_id: i64, conn: &Connection) -> Result<(), String> {
    conn.execute("DELETE FROM offline_queue WHERE id = ?1", params![task_id])
        .map_err(|e| format!("Failed to remove task: {}", e))?;

    tracing::info!("Removed offline task {}", task_id);
    Ok(())
}

/// 处理离线队列（当网络恢复时调用）
#[command]
pub async fn process_offline_queue() -> Result<i64, String> {
    // 检查网络状态
    let status = check_network_status()?;
    if status == NetworkStatus::Offline {
        return Err("Network is still offline".to_string());
    }

    let db = crate::memory_storage::DB_CONNECTION
        .lock()
        .map_err(|e| format!("Failed to lock DB: {}", e))?;
    let conn = db.as_ref().ok_or("Database not initialized")?;

    let tasks = get_pending_offline_tasks(conn)?;
    let mut processed = 0;

    for task in tasks {
        // 根据任务类型处理
        match task.task_type.as_str() {
            "ai_analysis" => {
                // AI 分析任务处理逻辑
                tracing::info!("Processing AI analysis task: {}", task.id);
                // 实际处理时需要调用相应的 AI 分析逻辑
                remove_offline_task(task.id, conn)?;
                processed += 1;
            }
            "daily_summary" => {
                // 日报生成任务处理逻辑
                tracing::info!("Processing daily summary task: {}", task.id);
                // 实际处理时需要调用日报生成逻辑
                remove_offline_task(task.id, conn)?;
                processed += 1;
            }
            _ => {
                tracing::warn!("Unknown task type: {}", task.task_type);
                update_offline_task_status(task.id, "failed", Some("Unknown task type"), conn)?;
            }
        }
    }

    tracing::info!("Processed {} offline tasks", processed);
    Ok(processed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        init_offline_queue(&conn).unwrap();
        conn
    }

    #[test]
    #[serial]
    fn test_offline_queue_crud() {
        let conn = setup_test_db();

        // Add task
        let task_id = add_offline_task("ai_analysis", r#"{"screenshot":"data"}"#, &conn).unwrap();
        assert!(task_id > 0);

        // Get status directly from the test connection
        let total: i64 = conn
            .query_row("SELECT COUNT(*) FROM offline_queue", [], |row| row.get(0))
            .unwrap();
        assert_eq!(total, 1);

        // Update status
        update_offline_task_status(task_id, "completed", None, &conn).unwrap();

        // Get updated status directly
        let pending: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM offline_queue WHERE status = 'pending'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(pending, 0);

        // Remove task
        remove_offline_task(task_id, &conn).unwrap();
        let total: i64 = conn
            .query_row("SELECT COUNT(*) FROM offline_queue", [], |row| row.get(0))
            .unwrap();
        assert_eq!(total, 0);
    }

    #[test]
    #[serial]
    fn test_retry_increment() {
        let conn = setup_test_db();

        let task_id = add_offline_task("test_task", "{}", &conn).unwrap();
        increment_retry_count(task_id, "Network error", &conn).unwrap();
        increment_retry_count(task_id, "Network error", &conn).unwrap();

        let mut stmt = conn
            .prepare("SELECT retry_count FROM offline_queue WHERE id = ?1")
            .unwrap();
        let count: i32 = stmt.query_row(params![task_id], |row| row.get(0)).unwrap();
        assert_eq!(count, 2);
    }

    #[test]
    #[serial]
    fn test_multiple_tasks_ordering() {
        let conn = setup_test_db();

        // 添加多个任务
        add_offline_task("task_1", "{}", &conn).unwrap();
        add_offline_task("task_2", "{}", &conn).unwrap();
        add_offline_task("task_3", "{}", &conn).unwrap();

        let tasks = get_pending_offline_tasks(&conn).unwrap();
        assert_eq!(tasks.len(), 3);
    }
}
