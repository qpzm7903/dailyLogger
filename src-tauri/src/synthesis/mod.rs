use std::path::PathBuf;
use tauri::command;

use crate::memory_storage;

#[command]
pub async fn generate_daily_summary() -> Result<String, String> {
    let settings = memory_storage::get_settings_sync()
        .map_err(|e| format!("Failed to get settings: {}", e))?;

    let obsidian_path = settings
        .obsidian_path
        .clone()
        .ok_or("Obsidian path not configured")?;

    if obsidian_path.is_empty() {
        return Err("Obsidian path is empty".to_string());
    }

    let api_base_url = settings
        .api_base_url
        .clone()
        .ok_or("API Base URL not configured")?;
    let api_key = settings.api_key.clone().ok_or("API Key not configured")?;
    let model_name = settings
        .model_name
        .clone()
        .unwrap_or_else(|| "gpt-4o".to_string());

    if api_key.is_empty() {
        return Err("API Key is empty".to_string());
    }

    let records = memory_storage::get_all_today_records_for_summary()
        .map_err(|e| format!("Failed to get records: {}", e))?;

    if records.is_empty() {
        return Err("No records for today".to_string());
    }

    let records_text: String = records
        .iter()
        .map(|r| {
            let time = chrono::DateTime::parse_from_rfc3339(&r.timestamp)
                .map(|dt| dt.format("%H:%M").to_string())
                .unwrap_or_else(|_| "unknown".to_string());

            let source = if r.source_type == "auto" {
                "🖥️ 自动感知"
            } else {
                "⚡ 闪念"
            };

            format!("- [{}] {}: {}", time, source, r.content)
        })
        .collect::<Vec<_>>()
        .join("\n");

    let prompt = format!(
        r#"你是一个工作日志助手。请根据以下今日工作记录，生成一份结构化的 Markdown 格式日报。

要求：
1. 按时间顺序组织
2. 提取关键工作内容和技术关键词
3. 总结今日工作成果和遇到的问题
4. 输出纯 Markdown 格式，不要有其他说明文字

今日记录：
{}

请生成日报："#,
        records_text
    );

    let client = reqwest::Client::new();

    let request_body = serde_json::json!({
        "model": model_name,
        "messages": [
            {
                "role": "user",
                "content": prompt
            }
        ],
        "max_tokens": 2000
    });

    let response = client
        .post(format!("{}/chat/completions", api_base_url))
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await
        .map_err(|e| format!("API request failed: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("API error ({}): {}", status, body));
    }

    let response_json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    let summary = response_json["choices"][0]["message"]["content"]
        .as_str()
        .ok_or("No content in response")?;

    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let filename = format!("{}.md", today);

    let output_dir = PathBuf::from(&obsidian_path);
    std::fs::create_dir_all(&output_dir)
        .map_err(|e| format!("Failed to create output directory: {}", e))?;

    let output_path = output_dir.join(&filename);
    std::fs::write(&output_path, summary).map_err(|e| format!("Failed to write summary: {}", e))?;

    let path_str = output_path.to_string_lossy().to_string();

    let mut updated_settings = settings.clone();
    updated_settings.last_summary_path = Some(path_str.clone());
    memory_storage::save_settings_sync(&updated_settings)
        .map_err(|e| format!("Failed to update settings: {}", e))?;

    tracing::info!("Daily summary generated: {}", path_str);

    Ok(path_str)
}
