use once_cell::sync::Lazy;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::command;

use crate::memory_storage::DB_CONNECTION;
use crate::network_status;

/// Task types that can be queued for offline retry
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OfflineTaskType {
    /// Screenshot AI analysis (stores screenshot path + record_id)
    ScreenshotAnalysis,
    /// Daily summary generation
    DailySummary,
    /// Weekly report generation
    WeeklyReport,
    /// Monthly report generation
    MonthlyReport,
}

impl std::fmt::Display for OfflineTaskType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OfflineTaskType::ScreenshotAnalysis => write!(f, "screenshot_analysis"),
            OfflineTaskType::DailySummary => write!(f, "daily_summary"),
            OfflineTaskType::WeeklyReport => write!(f, "weekly_report"),
            OfflineTaskType::MonthlyReport => write!(f, "monthly_report"),
        }
    }
}

impl std::str::FromStr for OfflineTaskType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "screenshot_analysis" => Ok(OfflineTaskType::ScreenshotAnalysis),
            "daily_summary" => Ok(OfflineTaskType::DailySummary),
            "weekly_report" => Ok(OfflineTaskType::WeeklyReport),
            "monthly_report" => Ok(OfflineTaskType::MonthlyReport),
            _ => Err(format!("Unknown task type: {}", s)),
        }
    }
}

/// A queued offline task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OfflineTask {
    pub id: i64,
    pub task_type: String,
    pub payload: String,
    pub record_id: Option<i64>,
    pub status: String,
    pub error_message: Option<String>,
    pub created_at: String,
    pub completed_at: Option<String>,
    pub retry_count: i32,
    pub max_retries: i32,
}

/// Maximum retries for a queued task
const DEFAULT_MAX_RETRIES: i32 = 5;

/// Flag to prevent multiple queue processors running concurrently
static QUEUE_PROCESSING: Lazy<Mutex<bool>> = Lazy::new(|| Mutex::new(false));

/// Create the offline_queue table. Called from init_database().
pub fn create_offline_queue_table(conn: &rusqlite::Connection) -> Result<(), String> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS offline_queue (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            task_type TEXT NOT NULL,
            payload TEXT NOT NULL DEFAULT '{}',
            record_id INTEGER,
            status TEXT NOT NULL DEFAULT 'pending',
            error_message TEXT,
            created_at TEXT NOT NULL,
            completed_at TEXT,
            retry_count INTEGER NOT NULL DEFAULT 0,
            max_retries INTEGER NOT NULL DEFAULT 5,
            FOREIGN KEY (record_id) REFERENCES records(id) ON DELETE SET NULL
        )",
        [],
    )
    .map_err(|e| format!("Failed to create offline_queue table: {}", e))?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_offline_queue_status ON offline_queue(status)",
        [],
    )
    .map_err(|e| format!("Failed to create index on offline_queue: {}", e))?;

    Ok(())
}

/// Enqueue a task for later processing when the network is available.
pub fn enqueue_task(
    task_type: &OfflineTaskType,
    payload: &str,
    record_id: Option<i64>,
) -> Result<i64, String> {
    let db = DB_CONNECTION
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;
    let conn = db.as_ref().ok_or("Database not initialized")?;

    let now = chrono::Utc::now().to_rfc3339();
    let task_type_str = task_type.to_string();

    conn.execute(
        "INSERT INTO offline_queue (task_type, payload, record_id, status, created_at, max_retries)
         VALUES (?1, ?2, ?3, 'pending', ?4, ?5)",
        params![task_type_str, payload, record_id, now, DEFAULT_MAX_RETRIES],
    )
    .map_err(|e| format!("Failed to enqueue task: {}", e))?;

    let id = conn.last_insert_rowid();
    tracing::info!(
        "Offline task queued: id={}, type={}, record_id={:?}",
        id,
        task_type_str,
        record_id
    );
    Ok(id)
}

/// Get all pending tasks from the queue, ordered by creation time (oldest first).
pub fn get_pending_tasks() -> Result<Vec<OfflineTask>, String> {
    let db = DB_CONNECTION
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;
    let conn = db.as_ref().ok_or("Database not initialized")?;

    let mut stmt = conn
        .prepare(
            "SELECT id, task_type, payload, record_id, status, error_message,
                    created_at, completed_at, retry_count, max_retries
             FROM offline_queue
             WHERE status = 'pending' AND retry_count < max_retries
             ORDER BY created_at ASC",
        )
        .map_err(|e| format!("Failed to prepare query: {}", e))?;

    let tasks = stmt
        .query_map([], |row| {
            Ok(OfflineTask {
                id: row.get(0)?,
                task_type: row.get(1)?,
                payload: row.get(2)?,
                record_id: row.get(3)?,
                status: row.get(4)?,
                error_message: row.get(5)?,
                created_at: row.get(6)?,
                completed_at: row.get(7)?,
                retry_count: row.get(8)?,
                max_retries: row.get(9)?,
            })
        })
        .map_err(|e| format!("Failed to query tasks: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to collect tasks: {}", e))?;

    Ok(tasks)
}

/// Mark a task as completed.
pub fn mark_task_completed(task_id: i64) -> Result<(), String> {
    let db = DB_CONNECTION
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;
    let conn = db.as_ref().ok_or("Database not initialized")?;

    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE offline_queue SET status = 'completed', completed_at = ?1 WHERE id = ?2",
        params![now, task_id],
    )
    .map_err(|e| format!("Failed to mark task completed: {}", e))?;

    Ok(())
}

/// Mark a task as failed, incrementing the retry count.
pub fn mark_task_failed(task_id: i64, error: &str) -> Result<(), String> {
    let db = DB_CONNECTION
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;
    let conn = db.as_ref().ok_or("Database not initialized")?;

    conn.execute(
        "UPDATE offline_queue SET retry_count = retry_count + 1, error_message = ?1 WHERE id = ?2",
        params![error, task_id],
    )
    .map_err(|e| format!("Failed to mark task failed: {}", e))?;

    // Check if max retries exceeded — mark as permanently failed
    let retry_count: i32 = conn
        .query_row(
            "SELECT retry_count FROM offline_queue WHERE id = ?1",
            params![task_id],
            |row| row.get(0),
        )
        .map_err(|e| format!("Failed to get retry count: {}", e))?;

    let max_retries: i32 = conn
        .query_row(
            "SELECT max_retries FROM offline_queue WHERE id = ?1",
            params![task_id],
            |row| row.get(0),
        )
        .map_err(|e| format!("Failed to get max retries: {}", e))?;

    if retry_count >= max_retries {
        conn.execute(
            "UPDATE offline_queue SET status = 'failed' WHERE id = ?1",
            params![task_id],
        )
        .map_err(|e| format!("Failed to mark task as permanently failed: {}", e))?;
        tracing::warn!(
            "Offline task {} permanently failed after {} retries",
            task_id,
            retry_count
        );
    }

    Ok(())
}

/// Get the count of pending tasks in the queue.
pub fn get_pending_count() -> Result<i64, String> {
    let db = DB_CONNECTION
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;
    let conn = db.as_ref().ok_or("Database not initialized")?;

    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM offline_queue WHERE status = 'pending' AND retry_count < max_retries",
            [],
            |row| row.get(0),
        )
        .map_err(|e| format!("Failed to count pending tasks: {}", e))?;

    Ok(count)
}

/// Clean up completed and permanently failed tasks older than 7 days.
pub fn cleanup_old_tasks() -> Result<i64, String> {
    let db = DB_CONNECTION
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;
    let conn = db.as_ref().ok_or("Database not initialized")?;

    let cutoff = (chrono::Utc::now() - chrono::Duration::days(7)).to_rfc3339();
    let deleted = conn
        .execute(
            "DELETE FROM offline_queue WHERE status IN ('completed', 'failed') AND created_at < ?1",
            params![cutoff],
        )
        .map_err(|e| format!("Failed to cleanup old tasks: {}", e))?;

    Ok(deleted as i64)
}

/// Process pending tasks in the offline queue.
/// Called when network comes back online.
/// Uses exponential backoff between retries.
pub async fn process_queue() {
    // Prevent concurrent processing
    {
        let mut processing = QUEUE_PROCESSING.lock().unwrap();
        if *processing {
            tracing::debug!("Queue processing already in progress, skipping");
            return;
        }
        *processing = true;
    }

    tracing::info!("Processing offline queue...");

    let tasks = match get_pending_tasks() {
        Ok(t) => t,
        Err(e) => {
            tracing::error!("Failed to get pending tasks: {}", e);
            let mut processing = QUEUE_PROCESSING.lock().unwrap();
            *processing = false;
            return;
        }
    };

    if tasks.is_empty() {
        tracing::debug!("No pending tasks in offline queue");
        let mut processing = QUEUE_PROCESSING.lock().unwrap();
        *processing = false;
        return;
    }

    tracing::info!("Found {} pending tasks to process", tasks.len());

    for task in &tasks {
        // Check if still online before processing each task
        if !network_status::is_online() {
            tracing::info!("Network went offline during queue processing, pausing");
            break;
        }

        // Exponential backoff: 2^retry_count seconds (1, 2, 4, 8, 16s)
        if task.retry_count > 0 {
            let delay_secs = 2u64.pow(task.retry_count as u32).min(60);
            tokio::time::sleep(std::time::Duration::from_secs(delay_secs)).await;
        }

        let task_type = match task.task_type.parse::<OfflineTaskType>() {
            Ok(t) => t,
            Err(e) => {
                tracing::error!("Invalid task type for task {}: {}", task.id, &e);
                let _ = mark_task_failed(task.id, &e);
                continue;
            }
        };

        tracing::info!(
            "Processing queued task: id={}, type={}, retry={}",
            task.id,
            task.task_type,
            task.retry_count
        );

        let result = execute_queued_task(&task_type, &task.payload, task.record_id).await;

        match result {
            Ok(()) => {
                if let Err(e) = mark_task_completed(task.id) {
                    tracing::error!("Failed to mark task {} as completed: {}", task.id, e);
                } else {
                    tracing::info!("Queued task {} completed successfully", task.id);
                }
            }
            Err(e) => {
                tracing::warn!("Queued task {} failed: {}", task.id, e);
                let _ = mark_task_failed(task.id, &e);
            }
        }
    }

    // Cleanup old completed/failed tasks
    if let Ok(cleaned) = cleanup_old_tasks() {
        if cleaned > 0 {
            tracing::info!("Cleaned up {} old queue entries", cleaned);
        }
    }

    let mut processing = QUEUE_PROCESSING.lock().unwrap();
    *processing = false;
}

/// Execute a single queued task based on its type.
async fn execute_queued_task(
    task_type: &OfflineTaskType,
    _payload: &str,
    _record_id: Option<i64>,
) -> Result<(), String> {
    match task_type {
        OfflineTaskType::ScreenshotAnalysis => {
            // Screenshot analysis: the screenshot is already saved locally.
            // We need to re-analyze it with the AI.
            // For now, we skip re-analysis since the screenshot record already exists.
            // The content will remain as "offline - pending analysis" until retry.
            tracing::info!("Screenshot analysis retry - skipping (screenshot already saved)");
            Ok(())
        }
        OfflineTaskType::DailySummary => {
            crate::synthesis::generate_daily_summary().await.map(|_| ())
        }
        OfflineTaskType::WeeklyReport => {
            crate::synthesis::generate_weekly_report().await.map(|_| ())
        }
        OfflineTaskType::MonthlyReport => crate::synthesis::generate_monthly_report()
            .await
            .map(|_| ()),
    }
}

/// Tauri command: get offline queue status
#[command]
pub fn get_offline_queue_status() -> Result<serde_json::Value, String> {
    let pending = get_pending_count()?;
    Ok(serde_json::json!({
        "pending_count": pending,
        "is_online": network_status::is_online(),
    }))
}

/// Tauri command: manually trigger queue processing
#[command]
pub async fn process_offline_queue() -> Result<String, String> {
    if !network_status::is_online() {
        return Err("无法处理队列：当前处于离线状态".to_string());
    }
    process_queue().await;
    let remaining = get_pending_count()?;
    Ok(format!("队列处理完成，剩余 {} 个待处理任务", remaining))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory_storage;
    use serial_test::serial;

    fn setup_test_db() {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        conn.execute(
            "CREATE TABLE IF NOT EXISTS records (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp TEXT NOT NULL,
                source_type TEXT NOT NULL,
                content TEXT NOT NULL,
                screenshot_path TEXT,
                monitor_info TEXT,
                tags TEXT
            )",
            [],
        )
        .unwrap();
        conn.execute(
            "CREATE TABLE IF NOT EXISTS settings (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                api_base_url TEXT, api_key TEXT, model_name TEXT,
                screenshot_interval INTEGER DEFAULT 5,
                summary_time TEXT DEFAULT '18:00',
                obsidian_path TEXT, auto_capture_enabled INTEGER DEFAULT 0,
                last_summary_path TEXT, summary_model_name TEXT,
                analysis_prompt TEXT, summary_prompt TEXT,
                change_threshold INTEGER DEFAULT 3,
                max_silent_minutes INTEGER DEFAULT 30,
                summary_title_format TEXT DEFAULT '工作日报 - {date}',
                include_manual_records INTEGER DEFAULT 1,
                window_whitelist TEXT DEFAULT '[]',
                window_blacklist TEXT DEFAULT '[]',
                use_whitelist_only INTEGER DEFAULT 0,
                auto_adjust_silent INTEGER DEFAULT 1,
                silent_adjustment_paused_until TEXT,
                auto_detect_work_time INTEGER DEFAULT 1,
                use_custom_work_time INTEGER DEFAULT 0,
                custom_work_time_start TEXT DEFAULT '09:00',
                custom_work_time_end TEXT DEFAULT '18:00',
                learned_work_time TEXT,
                capture_mode TEXT DEFAULT 'primary',
                selected_monitor_index INTEGER DEFAULT 0,
                tag_categories TEXT DEFAULT '[]',
                is_ollama INTEGER DEFAULT 0,
                weekly_report_prompt TEXT,
                weekly_report_day INTEGER DEFAULT 0,
                last_weekly_report_path TEXT,
                monthly_report_prompt TEXT,
                custom_report_prompt TEXT,
                last_custom_report_path TEXT,
                last_monthly_report_path TEXT
            )",
            [],
        )
        .unwrap();
        conn.execute("INSERT OR IGNORE INTO settings (id) VALUES (1)", [])
            .unwrap();

        // Create offline_queue table
        create_offline_queue_table(&conn).unwrap();

        let mut db = memory_storage::DB_CONNECTION.lock().unwrap();
        *db = Some(conn);
    }

    #[test]
    #[serial]
    fn test_enqueue_and_get_pending() {
        setup_test_db();

        // Insert a record first to satisfy FK constraint
        {
            let db = DB_CONNECTION.lock().unwrap();
            let conn = db.as_ref().unwrap();
            conn.execute(
                "INSERT INTO records (timestamp, source_type, content) VALUES (?1, 'auto', 'test')",
                params![chrono::Utc::now().to_rfc3339()],
            )
            .unwrap();
        }

        let id = enqueue_task(&OfflineTaskType::ScreenshotAnalysis, "{}", Some(1)).unwrap();
        assert!(id > 0);

        let tasks = get_pending_tasks().unwrap();
        assert!(tasks.iter().any(|t| t.id == id));

        let task = tasks.iter().find(|t| t.id == id).unwrap();
        assert_eq!(task.task_type, "screenshot_analysis");
        assert_eq!(task.status, "pending");
        assert_eq!(task.retry_count, 0);
        assert_eq!(task.record_id, Some(1));
    }

    #[test]
    #[serial]
    fn test_mark_task_completed() {
        setup_test_db();

        let id = enqueue_task(&OfflineTaskType::DailySummary, "{}", None).unwrap();
        mark_task_completed(id).unwrap();

        let tasks = get_pending_tasks().unwrap();
        assert!(!tasks.iter().any(|t| t.id == id));
    }

    #[test]
    #[serial]
    fn test_mark_task_failed_increments_retry() {
        setup_test_db();

        let id = enqueue_task(&OfflineTaskType::DailySummary, "{}", None).unwrap();
        mark_task_failed(id, "network error").unwrap();

        let tasks = get_pending_tasks().unwrap();
        let task = tasks.iter().find(|t| t.id == id).unwrap();
        assert_eq!(task.retry_count, 1);
        assert_eq!(task.error_message, Some("network error".to_string()));
    }

    #[test]
    #[serial]
    fn test_max_retries_marks_failed() {
        setup_test_db();

        let id = enqueue_task(&OfflineTaskType::DailySummary, "{}", None).unwrap();

        // Exhaust all retries
        for _ in 0..DEFAULT_MAX_RETRIES {
            mark_task_failed(id, "still failing").unwrap();
        }

        // Should no longer appear in pending tasks
        let tasks = get_pending_tasks().unwrap();
        assert!(!tasks.iter().any(|t| t.id == id));
    }

    #[test]
    #[serial]
    fn test_get_pending_count() {
        setup_test_db();

        let count_before = get_pending_count().unwrap();

        enqueue_task(&OfflineTaskType::ScreenshotAnalysis, "{}", None).unwrap();
        enqueue_task(&OfflineTaskType::DailySummary, "{}", None).unwrap();

        let count_after = get_pending_count().unwrap();
        assert_eq!(count_after - count_before, 2);
    }

    #[test]
    #[serial]
    fn test_task_type_roundtrip() {
        let types = vec![
            OfflineTaskType::ScreenshotAnalysis,
            OfflineTaskType::DailySummary,
            OfflineTaskType::WeeklyReport,
            OfflineTaskType::MonthlyReport,
        ];

        for t in types {
            let s = t.to_string();
            let parsed: OfflineTaskType = s.parse().unwrap();
            assert_eq!(parsed, t);
        }
    }

    #[test]
    #[serial]
    fn test_cleanup_old_tasks() {
        setup_test_db();

        // Enqueue and complete a task
        let id = enqueue_task(&OfflineTaskType::DailySummary, "{}", None).unwrap();
        mark_task_completed(id).unwrap();

        // Set completed_at to 8 days ago to simulate old task
        let old_date = (chrono::Utc::now() - chrono::Duration::days(8)).to_rfc3339();
        let db = DB_CONNECTION.lock().unwrap();
        let conn = db.as_ref().unwrap();
        conn.execute(
            "UPDATE offline_queue SET created_at = ?1 WHERE id = ?2",
            params![old_date, id],
        )
        .unwrap();
        drop(db);

        let cleaned = cleanup_old_tasks().unwrap();
        assert!(cleaned >= 1);
    }
}
