// ─── DATA-003: 手动标签系统 ────────────────────────────────────────────────────
//!
//! Tag management module for manual tagging of records.
//! Supports CRUD operations for tags, tag-record associations, and tag-based filtering.

use crate::memory_storage::{Record, DB_CONNECTION};
use rusqlite::{params, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use tauri::command;

/// 手动标签结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManualTag {
    pub id: i64,
    pub name: String,
    pub color: String,
    pub created_at: String,
    /// 使用计数，用于标签云显示
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage_count: Option<i64>,
}

/// 记录与标签关联信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordTagInfo {
    pub record_id: i64,
    pub tag_ids: Vec<i64>,
}

// ─── AI-004: 工作分类标签相关常量 ──────────────────────────────────────────────

/// Default tag categories for work classification
pub const DEFAULT_TAG_CATEGORIES: &[&str] = &[
    "开发", "会议", "写作", "学习", "研究", "沟通", "规划", "文档", "测试", "设计",
];

/// 预设标签颜色
pub const PRESET_TAG_COLORS: [&str; 8] = [
    "blue", "green", "yellow", "red", "purple", "pink", "cyan", "orange",
];

// ─── Tag Query Functions ────────────────────────────────────────────────────────

/// Get the default tag categories
#[command]
pub fn get_default_tag_categories() -> Vec<String> {
    DEFAULT_TAG_CATEGORIES
        .iter()
        .map(|s| s.to_string())
        .collect()
}

/// Get all unique tags currently used in records
#[command]
pub fn get_all_tags() -> Result<Vec<String>, String> {
    let db_guard = DB_CONNECTION
        .lock()
        .map_err(|e| format!("DB lock error: {}", e))?;
    let conn = db_guard.as_ref().ok_or("Database not initialized")?;

    let mut stmt = conn
        .prepare("SELECT DISTINCT tags FROM records WHERE tags IS NOT NULL AND tags != '[]'")
        .map_err(|e| format!("Failed to prepare query: {}", e))?;

    let tag_strings: Vec<String> = stmt
        .query_map([], |row| row.get(0))
        .map_err(|e| format!("Failed to query tags: {}", e))?
        .filter_map(|r| r.ok())
        .collect();

    // Parse JSON arrays and collect unique tags
    let mut unique_tags = HashSet::new();
    for tag_str in tag_strings {
        if let Ok(tags) = serde_json::from_str::<Vec<String>>(&tag_str) {
            for tag in tags {
                unique_tags.insert(tag);
            }
        }
    }

    let mut result: Vec<String> = unique_tags.into_iter().collect();
    result.sort();
    Ok(result)
}

/// Get records filtered by a specific tag
#[command]
pub fn get_records_by_tag(tag: String) -> Result<Vec<Record>, String> {
    let db_guard = DB_CONNECTION
        .lock()
        .map_err(|e| format!("DB lock error: {}", e))?;
    let conn = db_guard.as_ref().ok_or("Database not initialized")?;

    // Get all records with tags and filter in Rust (SQLite doesn't handle JSON arrays well)
    let mut stmt = conn
        .prepare(
            "SELECT id, timestamp, source_type, content, screenshot_path, monitor_info, tags
             FROM records
             WHERE tags IS NOT NULL
             ORDER BY timestamp DESC",
        )
        .map_err(|e| format!("Failed to prepare query: {}", e))?;

    let records: Vec<Record> = stmt
        .query_map([], |row| {
            Ok(Record {
                id: row.get(0)?,
                timestamp: row.get(1)?,
                source_type: row.get(2)?,
                content: row.get(3)?,
                screenshot_path: row.get(4)?,
                monitor_info: row.get(5)?,
                tags: row.get(6)?,
            })
        })
        .map_err(|e| format!("Failed to query records: {}", e))?
        .filter_map(|r| r.ok())
        .filter(|r| {
            r.tags
                .as_ref()
                .and_then(|t| serde_json::from_str::<Vec<String>>(t).ok())
                .map(|tags| tags.contains(&tag))
                .unwrap_or(false)
        })
        .collect();

    Ok(records)
}

// ─── Manual Tag CRUD Operations ─────────────────────────────────────────────────

/// 创建手动标签
#[command]
pub fn create_manual_tag(name: String, color: String) -> Result<ManualTag, String> {
    // 验证标签名长度
    if name.is_empty() || name.len() > 20 {
        return Err("标签名长度必须在 1-20 个字符之间".to_string());
    }

    // 验证颜色是否有效
    if !PRESET_TAG_COLORS.contains(&color.as_str()) {
        return Err(format!(
            "无效的颜色 '{}', 可选颜色: {}",
            color,
            PRESET_TAG_COLORS.join(", ")
        ));
    }

    let db_guard = DB_CONNECTION
        .lock()
        .map_err(|e| format!("DB lock error: {}", e))?;
    let conn = db_guard.as_ref().ok_or("Database not initialized")?;

    let created_at = chrono::Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO manual_tags (name, color, created_at) VALUES (?1, ?2, ?3)",
        params![name, color, created_at],
    )
    .map_err(|e| {
        if e.to_string().contains("UNIQUE constraint failed") {
            format!("标签 '{}' 已存在", name)
        } else {
            format!("Failed to create tag: {}", e)
        }
    })?;

    let id = conn.last_insert_rowid();

    Ok(ManualTag {
        id,
        name,
        color,
        created_at,
        usage_count: Some(0),
    })
}

/// 获取所有手动标签（含使用计数）
#[command]
pub fn get_all_manual_tags() -> Result<Vec<ManualTag>, String> {
    let db_guard = DB_CONNECTION
        .lock()
        .map_err(|e| format!("DB lock error: {}", e))?;
    let conn = db_guard.as_ref().ok_or("Database not initialized")?;

    let mut stmt = conn
        .prepare(
            "SELECT t.id, t.name, t.color, t.created_at, COUNT(rmt.record_id) as usage_count
             FROM manual_tags t
             LEFT JOIN record_manual_tags rmt ON t.id = rmt.tag_id
             GROUP BY t.id, t.name, t.color, t.created_at
             ORDER BY usage_count DESC, t.name ASC",
        )
        .map_err(|e| format!("Failed to prepare query: {}", e))?;

    let tags = stmt
        .query_map([], |row| {
            Ok(ManualTag {
                id: row.get(0)?,
                name: row.get(1)?,
                color: row.get(2)?,
                created_at: row.get(3)?,
                usage_count: Some(row.get(4)?),
            })
        })
        .map_err(|e| format!("Failed to query tags: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to collect tags: {}", e))?;

    Ok(tags)
}

/// 更新手动标签
#[command]
pub fn update_manual_tag(id: i64, name: String, color: String) -> Result<(), String> {
    // 验证标签名长度
    if name.is_empty() || name.len() > 20 {
        return Err("标签名长度必须在 1-20 个字符之间".to_string());
    }

    // 验证颜色是否有效
    if !PRESET_TAG_COLORS.contains(&color.as_str()) {
        return Err(format!(
            "无效的颜色 '{}', 可选颜色: {}",
            color,
            PRESET_TAG_COLORS.join(", ")
        ));
    }

    let db_guard = DB_CONNECTION
        .lock()
        .map_err(|e| format!("DB lock error: {}", e))?;
    let conn = db_guard.as_ref().ok_or("Database not initialized")?;

    let rows_affected = conn
        .execute(
            "UPDATE manual_tags SET name = ?1, color = ?2 WHERE id = ?3",
            params![name, color, id],
        )
        .map_err(|e| {
            if e.to_string().contains("UNIQUE constraint failed") {
                format!("标签 '{}' 已存在", name)
            } else {
                format!("Failed to update tag: {}", e)
            }
        })?;

    if rows_affected == 0 {
        return Err(format!("标签 ID {} 不存在", id));
    }

    Ok(())
}

/// 删除手动标签（级联删除关联）
#[command]
pub fn delete_manual_tag(id: i64) -> Result<(), String> {
    let db_guard = DB_CONNECTION
        .lock()
        .map_err(|e| format!("DB lock error: {}", e))?;
    let conn = db_guard.as_ref().ok_or("Database not initialized")?;

    // 先删除关联记录
    conn.execute(
        "DELETE FROM record_manual_tags WHERE tag_id = ?1",
        params![id],
    )
    .map_err(|e| format!("Failed to delete tag associations: {}", e))?;

    // 再删除标签
    let rows_affected = conn
        .execute("DELETE FROM manual_tags WHERE id = ?1", params![id])
        .map_err(|e| format!("Failed to delete tag: {}", e))?;

    if rows_affected == 0 {
        return Err(format!("标签 ID {} 不存在", id));
    }

    Ok(())
}

// ─── Tag-Record Association Operations ──────────────────────────────────────────

/// 为记录添加标签
#[command]
pub fn add_tag_to_record(record_id: i64, tag_id: i64) -> Result<(), String> {
    let db_guard = DB_CONNECTION
        .lock()
        .map_err(|e| format!("DB lock error: {}", e))?;
    let conn = db_guard.as_ref().ok_or("Database not initialized")?;

    // 检查记录是否存在
    let record_exists: bool = conn
        .query_row(
            "SELECT 1 FROM records WHERE id = ?1",
            params![record_id],
            |_row| Ok(true),
        )
        .optional()
        .map_err(|e| format!("Failed to check record: {}", e))?
        .unwrap_or(false);

    if !record_exists {
        return Err(format!("记录 ID {} 不存在", record_id));
    }

    // 检查标签是否存在
    let tag_exists: bool = conn
        .query_row(
            "SELECT 1 FROM manual_tags WHERE id = ?1",
            params![tag_id],
            |_row| Ok(true),
        )
        .optional()
        .map_err(|e| format!("Failed to check tag: {}", e))?
        .unwrap_or(false);

    if !tag_exists {
        return Err(format!("标签 ID {} 不存在", tag_id));
    }

    // 检查是否已关联
    let already_linked: bool = conn
        .query_row(
            "SELECT 1 FROM record_manual_tags WHERE record_id = ?1 AND tag_id = ?2",
            params![record_id, tag_id],
            |_row| Ok(true),
        )
        .optional()
        .map_err(|e| format!("Failed to check link: {}", e))?
        .unwrap_or(false);

    if already_linked {
        return Ok(()); // 已关联，无需重复添加
    }

    // 检查该记录的标签数量是否已达上限
    let tag_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM record_manual_tags WHERE record_id = ?1",
            params![record_id],
            |row| row.get(0),
        )
        .map_err(|e| format!("Failed to count tags: {}", e))?;

    if tag_count >= 10 {
        return Err("每条记录最多只能添加 10 个标签".to_string());
    }

    conn.execute(
        "INSERT INTO record_manual_tags (record_id, tag_id) VALUES (?1, ?2)",
        params![record_id, tag_id],
    )
    .map_err(|e| format!("Failed to add tag to record: {}", e))?;

    Ok(())
}

/// 从记录移除标签
#[command]
pub fn remove_tag_from_record(record_id: i64, tag_id: i64) -> Result<(), String> {
    let db_guard = DB_CONNECTION
        .lock()
        .map_err(|e| format!("DB lock error: {}", e))?;
    let conn = db_guard.as_ref().ok_or("Database not initialized")?;

    let rows_affected = conn
        .execute(
            "DELETE FROM record_manual_tags WHERE record_id = ?1 AND tag_id = ?2",
            params![record_id, tag_id],
        )
        .map_err(|e| format!("Failed to remove tag from record: {}", e))?;

    if rows_affected == 0 {
        return Err("记录与标签未关联".to_string());
    }

    Ok(())
}

/// 获取记录的所有手动标签
#[command]
pub fn get_tags_for_record(record_id: i64) -> Result<Vec<ManualTag>, String> {
    let db_guard = DB_CONNECTION
        .lock()
        .map_err(|e| format!("DB lock error: {}", e))?;
    let conn = db_guard.as_ref().ok_or("Database not initialized")?;

    let mut stmt = conn
        .prepare(
            "SELECT t.id, t.name, t.color, t.created_at
             FROM manual_tags t
             INNER JOIN record_manual_tags rmt ON t.id = rmt.tag_id
             WHERE rmt.record_id = ?1
             ORDER BY t.name",
        )
        .map_err(|e| format!("Failed to prepare query: {}", e))?;

    let tags = stmt
        .query_map(params![record_id], |row| {
            Ok(ManualTag {
                id: row.get(0)?,
                name: row.get(1)?,
                color: row.get(2)?,
                created_at: row.get(3)?,
                usage_count: None,
            })
        })
        .map_err(|e| format!("Failed to query tags: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to collect tags: {}", e))?;

    Ok(tags)
}

/// 批量获取多条记录的手动标签 (PERF-001: 替代 N+1 查询)
#[command]
pub fn get_tags_for_records(record_ids: Vec<i64>) -> Result<HashMap<i64, Vec<ManualTag>>, String> {
    if record_ids.is_empty() {
        return Ok(HashMap::new());
    }

    let db_guard = DB_CONNECTION
        .lock()
        .map_err(|e| format!("DB lock error: {}", e))?;
    let conn = db_guard.as_ref().ok_or("Database not initialized")?;

    let placeholders: Vec<String> = record_ids.iter().map(|_| "?".to_string()).collect();
    let placeholders_str = placeholders.join(",");

    let sql = format!(
        "SELECT rmt.record_id, t.id, t.name, t.color, t.created_at
         FROM manual_tags t
         INNER JOIN record_manual_tags rmt ON t.id = rmt.tag_id
         WHERE rmt.record_id IN ({})
         ORDER BY rmt.record_id, t.name",
        placeholders_str
    );

    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| format!("Failed to prepare query: {}", e))?;

    let params: Vec<Box<dyn rusqlite::types::ToSql>> = record_ids
        .iter()
        .map(|id| Box::new(*id) as Box<dyn rusqlite::types::ToSql>)
        .collect();
    let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();

    let rows = stmt
        .query_map(param_refs.as_slice(), |row| {
            Ok((
                row.get::<_, i64>(0)?,
                ManualTag {
                    id: row.get(1)?,
                    name: row.get(2)?,
                    color: row.get(3)?,
                    created_at: row.get(4)?,
                    usage_count: None,
                },
            ))
        })
        .map_err(|e| format!("Failed to query tags: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to collect tags: {}", e))?;

    let mut result: HashMap<i64, Vec<ManualTag>> = HashMap::new();
    for (record_id, tag) in rows {
        result.entry(record_id).or_default().push(tag);
    }

    Ok(result)
}

/// 按多个标签筛选记录（交集 AND 逻辑）
#[command]
pub fn get_records_by_manual_tags(
    tag_ids: Vec<i64>,
    page: i64,
    page_size: i64,
) -> Result<Vec<Record>, String> {
    if tag_ids.is_empty() {
        return Ok(Vec::new());
    }

    let db_guard = DB_CONNECTION
        .lock()
        .map_err(|e| format!("DB lock error: {}", e))?;
    let conn = db_guard.as_ref().ok_or("Database not initialized")?;

    // 构建参数占位符
    let placeholders: Vec<String> = tag_ids.iter().map(|_| "?".to_string()).collect();
    let placeholders_str = placeholders.join(",");

    // 交集筛选：找出同时包含所有指定标签的记录
    let sql = format!(
        "SELECT r.id, r.timestamp, r.source_type, r.content, r.screenshot_path, r.monitor_info, r.tags
         FROM records r
         WHERE r.id IN (
             SELECT record_id FROM record_manual_tags
             WHERE tag_id IN ({})
             GROUP BY record_id
             HAVING COUNT(DISTINCT tag_id) = ?
         )
         ORDER BY r.timestamp DESC
         LIMIT ? OFFSET ?",
        placeholders_str
    );

    let offset = page * page_size;
    let tag_count = tag_ids.len() as i64;

    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| format!("Failed to prepare query: {}", e))?;

    // 构建参数列表
    let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
    for id in &tag_ids {
        params_vec.push(Box::new(*id));
    }
    params_vec.push(Box::new(tag_count));
    params_vec.push(Box::new(page_size));
    params_vec.push(Box::new(offset));

    let params_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|p| p.as_ref()).collect();

    let records = stmt
        .query_map(params_refs.as_slice(), |row| {
            Ok(Record {
                id: row.get(0)?,
                timestamp: row.get(1)?,
                source_type: row.get(2)?,
                content: row.get(3)?,
                screenshot_path: row.get(4)?,
                monitor_info: row.get(5)?,
                tags: row.get(6)?,
            })
        })
        .map_err(|e| format!("Failed to query records: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to collect records: {}", e))?;

    Ok(records)
}

// ─── Tests ───────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory_storage::add_record;
    use rusqlite::Connection;
    use serial_test::serial;

    /// Initializes an in-memory database for tag testing
    fn setup_test_db() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute(
            "CREATE TABLE records (
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
            "CREATE TABLE manual_tags (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                color TEXT NOT NULL DEFAULT 'blue',
                created_at TEXT NOT NULL
            )",
            [],
        )
        .unwrap();
        conn.execute(
            "CREATE TABLE record_manual_tags (
                record_id INTEGER NOT NULL,
                tag_id INTEGER NOT NULL,
                PRIMARY KEY (record_id, tag_id),
                FOREIGN KEY (record_id) REFERENCES records(id) ON DELETE CASCADE,
                FOREIGN KEY (tag_id) REFERENCES manual_tags(id) ON DELETE CASCADE
            )",
            [],
        )
        .unwrap();
        let mut db = DB_CONNECTION.lock().unwrap();
        *db = Some(conn);
    }

    #[test]
    #[serial]
    fn get_default_tag_categories_returns_expected_tags() {
        let tags = get_default_tag_categories();
        assert!(!tags.is_empty(), "Should return non-empty list");
        assert!(tags.contains(&"开发".to_string()), "Should contain '开发'");
        assert!(tags.contains(&"会议".to_string()), "Should contain '会议'");
        assert!(tags.contains(&"写作".to_string()), "Should contain '写作'");
    }

    #[test]
    #[serial]
    fn get_default_tag_categories_has_expected_count() {
        let tags = get_default_tag_categories();
        assert_eq!(tags.len(), 10, "Should have 10 default tag categories");
    }

    #[test]
    #[serial]
    fn get_all_tags_returns_empty_when_no_records() {
        setup_test_db();
        let tags = get_all_tags().unwrap();
        assert!(tags.is_empty(), "Should return empty when no records exist");
    }

    #[test]
    #[serial]
    fn get_all_tags_returns_unique_tags_from_records() {
        setup_test_db();

        // Add records with tags
        let _ = add_record(
            "auto",
            r#"{"current_focus":"test1"}"#,
            None,
            None,
            Some(r#"["开发","测试"]"#),
        );
        let _ = add_record(
            "auto",
            r#"{"current_focus":"test2"}"#,
            None,
            None,
            Some(r#"["会议","开发"]"#),
        );
        let _ = add_record("manual", "plain text", None, None, None);

        let tags = get_all_tags().unwrap();
        assert_eq!(tags.len(), 3, "Should have 3 unique tags");
        assert!(tags.contains(&"开发".to_string()));
        assert!(tags.contains(&"测试".to_string()));
        assert!(tags.contains(&"会议".to_string()));
    }

    #[test]
    #[serial]
    fn get_records_by_tag_returns_matching_records() {
        setup_test_db();

        // Add records with different tags
        let _ = add_record(
            "auto",
            r#"{"current_focus":"dev work"}"#,
            None,
            None,
            Some(r#"["开发"]"#),
        );
        let _ = add_record(
            "auto",
            r#"{"current_focus":"meeting"}"#,
            None,
            None,
            Some(r#"["会议"]"#),
        );
        let _ = add_record(
            "auto",
            r#"{"current_focus":"dev and test"}"#,
            None,
            None,
            Some(r#"["开发","测试"]"#),
        );

        let records = get_records_by_tag("开发".to_string()).unwrap();
        assert_eq!(records.len(), 2, "Should find 2 records with '开发' tag");

        let meeting_records = get_records_by_tag("会议".to_string()).unwrap();
        assert_eq!(
            meeting_records.len(),
            1,
            "Should find 1 record with '会议' tag"
        );
    }

    #[test]
    #[serial]
    fn get_records_by_tag_returns_empty_for_nonexistent_tag() {
        setup_test_db();

        let _ = add_record(
            "auto",
            r#"{"current_focus":"test"}"#,
            None,
            None,
            Some(r#"["开发"]"#),
        );
        let records = get_records_by_tag("不存在".to_string()).unwrap();
        assert!(
            records.is_empty(),
            "Should return empty for nonexistent tag"
        );
    }

    #[test]
    #[serial]
    fn get_records_by_tag_ignores_records_without_tags() {
        setup_test_db();

        // Add records without tags
        let _ = add_record("auto", r#"{"current_focus":"no tags"}"#, None, None, None);
        let _ = add_record("manual", "plain text", None, None, None);

        let records = get_records_by_tag("开发".to_string()).unwrap();
        assert!(
            records.is_empty(),
            "Should return empty when no records have tags"
        );
    }

    // ── Tests for DATA-003: Manual Tag System ──

    #[test]
    #[serial]
    fn create_manual_tag_creates_tag_with_default_color() {
        setup_test_db();

        let tag = create_manual_tag("工作".to_string(), "blue".to_string()).unwrap();
        assert_eq!(tag.name, "工作");
        assert_eq!(tag.color, "blue");
        assert!(tag.id > 0);
    }

    #[test]
    #[serial]
    fn create_manual_tag_rejects_empty_name() {
        setup_test_db();

        let result = create_manual_tag("".to_string(), "blue".to_string());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("1-20"));
    }

    #[test]
    #[serial]
    fn create_manual_tag_rejects_long_name() {
        setup_test_db();

        let long_name = "这是一个非常长的标签名称超过了二十个字符的限制";
        let result = create_manual_tag(long_name.to_string(), "blue".to_string());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("1-20"));
    }

    #[test]
    #[serial]
    fn create_manual_tag_rejects_invalid_color() {
        setup_test_db();

        let result = create_manual_tag("工作".to_string(), "invalid".to_string());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("无效的颜色"));
    }

    #[test]
    #[serial]
    fn create_manual_tag_rejects_duplicate_name() {
        setup_test_db();

        let _ = create_manual_tag("工作".to_string(), "blue".to_string()).unwrap();
        let result = create_manual_tag("工作".to_string(), "green".to_string());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("已存在"));
    }

    #[test]
    #[serial]
    fn get_all_manual_tags_returns_tags_with_usage_count() {
        setup_test_db();

        // Create tags
        let tag1 = create_manual_tag("工作".to_string(), "blue".to_string()).unwrap();
        let _ = create_manual_tag("学习".to_string(), "green".to_string()).unwrap();

        // Add a record and associate with tag1
        let record_id = add_record("manual", "test content", None, None, None).unwrap();
        add_tag_to_record(record_id, tag1.id).unwrap();

        let tags = get_all_manual_tags().unwrap();
        assert_eq!(tags.len(), 2);

        // Tags should be sorted by usage count (工作 has 1, 学习 has 0)
        assert_eq!(tags[0].name, "工作");
        assert_eq!(tags[0].usage_count, Some(1));
        assert_eq!(tags[1].name, "学习");
        assert_eq!(tags[1].usage_count, Some(0));
    }

    #[test]
    #[serial]
    fn update_manual_tag_changes_name_and_color() {
        setup_test_db();

        let tag = create_manual_tag("工作".to_string(), "blue".to_string()).unwrap();
        update_manual_tag(tag.id, "任务".to_string(), "red".to_string()).unwrap();

        let tags = get_all_manual_tags().unwrap();
        assert_eq!(tags[0].name, "任务");
        assert_eq!(tags[0].color, "red");
    }

    #[test]
    #[serial]
    fn delete_manual_tag_removes_tag_and_associations() {
        setup_test_db();

        let tag = create_manual_tag("工作".to_string(), "blue".to_string()).unwrap();
        let record_id = add_record("manual", "test", None, None, None).unwrap();
        add_tag_to_record(record_id, tag.id).unwrap();

        delete_manual_tag(tag.id).unwrap();

        let tags = get_all_manual_tags().unwrap();
        assert!(tags.is_empty());

        // Record should still exist
        let tags_for_record = get_tags_for_record(record_id).unwrap();
        assert!(tags_for_record.is_empty());
    }

    #[test]
    #[serial]
    fn add_tag_to_record_associates_tag() {
        setup_test_db();

        let tag = create_manual_tag("工作".to_string(), "blue".to_string()).unwrap();
        let record_id = add_record("manual", "test content", None, None, None).unwrap();

        add_tag_to_record(record_id, tag.id).unwrap();

        let tags = get_tags_for_record(record_id).unwrap();
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].name, "工作");
    }

    #[test]
    #[serial]
    fn add_tag_to_record_rejects_nonexistent_record() {
        setup_test_db();

        let tag = create_manual_tag("工作".to_string(), "blue".to_string()).unwrap();
        let result = add_tag_to_record(999, tag.id);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("不存在"));
    }

    #[test]
    #[serial]
    fn add_tag_to_record_rejects_nonexistent_tag() {
        setup_test_db();

        let record_id = add_record("manual", "test", None, None, None).unwrap();
        let result = add_tag_to_record(record_id, 999);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("不存在"));
    }

    #[test]
    #[serial]
    fn add_tag_to_record_limits_tags_to_10() {
        setup_test_db();

        let record_id = add_record("manual", "test", None, None, None).unwrap();

        // Create and add 10 tags
        for i in 0..10 {
            let tag = create_manual_tag(format!("标签{}", i), "blue".to_string()).unwrap();
            add_tag_to_record(record_id, tag.id).unwrap();
        }

        // Try to add 11th tag
        let tag11 = create_manual_tag("标签11".to_string(), "red".to_string()).unwrap();
        let result = add_tag_to_record(record_id, tag11.id);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("10"));
    }

    #[test]
    #[serial]
    fn remove_tag_from_record_removes_association() {
        setup_test_db();

        let tag = create_manual_tag("工作".to_string(), "blue".to_string()).unwrap();
        let record_id = add_record("manual", "test", None, None, None).unwrap();
        add_tag_to_record(record_id, tag.id).unwrap();

        remove_tag_from_record(record_id, tag.id).unwrap();

        let tags = get_tags_for_record(record_id).unwrap();
        assert!(tags.is_empty());
    }

    #[test]
    #[serial]
    fn get_records_by_manual_tags_returns_intersection() {
        setup_test_db();

        // Create tags
        let tag1 = create_manual_tag("工作".to_string(), "blue".to_string()).unwrap();
        let tag2 = create_manual_tag("重要".to_string(), "red".to_string()).unwrap();

        // Add records
        let record1 = add_record("manual", "record 1", None, None, None).unwrap();
        let record2 = add_record("manual", "record 2", None, None, None).unwrap();
        let record3 = add_record("manual", "record 3", None, None, None).unwrap();

        // record1: tag1 only
        add_tag_to_record(record1, tag1.id).unwrap();
        // record2: tag2 only
        add_tag_to_record(record2, tag2.id).unwrap();
        // record3: both tags
        add_tag_to_record(record3, tag1.id).unwrap();
        add_tag_to_record(record3, tag2.id).unwrap();

        // Filter by both tags (AND logic)
        let records = get_records_by_manual_tags(vec![tag1.id, tag2.id], 0, 50).unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].id, record3);

        // Filter by single tag
        let records_tag1 = get_records_by_manual_tags(vec![tag1.id], 0, 50).unwrap();
        assert_eq!(records_tag1.len(), 2);
    }

    #[test]
    #[serial]
    fn get_records_by_manual_tags_supports_pagination() {
        setup_test_db();

        let tag = create_manual_tag("工作".to_string(), "blue".to_string()).unwrap();

        // Add 15 records with the tag
        for i in 0..15 {
            let record_id =
                add_record("manual", &format!("record {}", i), None, None, None).unwrap();
            add_tag_to_record(record_id, tag.id).unwrap();
        }

        // Page 0: first 10 records
        let page0 = get_records_by_manual_tags(vec![tag.id], 0, 10).unwrap();
        assert_eq!(page0.len(), 10);

        // Page 1: remaining 5 records
        let page1 = get_records_by_manual_tags(vec![tag.id], 1, 10).unwrap();
        assert_eq!(page1.len(), 5);
    }

    #[test]
    #[serial]
    fn get_tags_for_records_returns_empty_for_empty_input() {
        setup_test_db();

        let result = get_tags_for_records(vec![]).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    #[serial]
    fn get_tags_for_records_returns_tags_grouped_by_record() {
        setup_test_db();

        // Create tags
        let tag1 = create_manual_tag("工作".to_string(), "blue".to_string()).unwrap();
        let tag2 = create_manual_tag("重要".to_string(), "red".to_string()).unwrap();

        // Add records
        let record1 = add_record("manual", "record 1", None, None, None).unwrap();
        let record2 = add_record("manual", "record 2", None, None, None).unwrap();

        // record1 has tag1, record2 has both tags
        add_tag_to_record(record1, tag1.id).unwrap();
        add_tag_to_record(record2, tag1.id).unwrap();
        add_tag_to_record(record2, tag2.id).unwrap();

        let result = get_tags_for_records(vec![record1, record2]).unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result[&record1].len(), 1);
        assert_eq!(result[&record2].len(), 2);
    }

    #[test]
    #[serial]
    fn get_tags_for_records_skips_records_without_tags() {
        setup_test_db();

        let tag = create_manual_tag("工作".to_string(), "blue".to_string()).unwrap();

        // Add two records, only one with a tag
        let record1 = add_record("manual", "record 1", None, None, None).unwrap();
        let record2 = add_record("manual", "record 2", None, None, None).unwrap();

        add_tag_to_record(record1, tag.id).unwrap();

        let result = get_tags_for_records(vec![record1, record2]).unwrap();

        // Only record1 should be in result
        assert_eq!(result.len(), 1);
        assert!(result.contains_key(&record1));
        assert!(!result.contains_key(&record2));
    }
}
