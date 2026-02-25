use std::path::PathBuf;
use tauri::command;

use crate::memory_storage;

const DEFAULT_SUMMARY_PROMPT: &str = r#"你是一个工作日志助手。请根据以下今日工作记录，生成一份结构化的 Markdown 格式日报。

要求：
1. 按时间顺序组织
2. 提取关键工作内容和技术关键词
3. 总结今日工作成果和遇到的问题
4. 输出纯 Markdown 格式，不要有其他说明文字

今日记录：
{records}

请生成日报："#;

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
    // 日报生成优先使用 summary_model_name，未配置时回退到 model_name
    let model_name = settings
        .summary_model_name
        .clone()
        .filter(|s| !s.is_empty())
        .or_else(|| settings.model_name.clone())
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
                .map(|dt| dt.with_timezone(&chrono::Local).format("%H:%M").to_string())
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

    let prompt_template = settings
        .summary_prompt
        .as_deref()
        .filter(|s| !s.is_empty())
        .unwrap_or(DEFAULT_SUMMARY_PROMPT);

    let prompt = prompt_template.replace("{records}", &records_text);

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

    let masked_key = crate::mask_api_key(&api_key);
    let endpoint = format!("{}/chat/completions", api_base_url);
    tracing::info!(
        "{}",
        serde_json::json!({
            "event": "llm_request",
            "caller": "generate_daily_summary",
            "endpoint": endpoint,
            "model": model_name,
            "max_tokens": 2000,
            "api_key_masked": masked_key,
            "has_image": false,
            "prompt": prompt,
            "records_count": records.len(),
        })
    );

    let start = std::time::Instant::now();
    let response = client
        .post(&endpoint)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await
        .map_err(|e| {
            let elapsed_ms = start.elapsed().as_millis();
            tracing::error!(
                "{}",
                serde_json::json!({
                    "event": "llm_error",
                    "caller": "generate_daily_summary",
                    "error": format!("API request failed: {}", e),
                    "elapsed_ms": elapsed_ms,
                })
            );
            format!("API request failed: {}", e)
        })?;
    let elapsed_ms = start.elapsed().as_millis();

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        tracing::error!(
            "{}",
            serde_json::json!({
                "event": "llm_error",
                "caller": "generate_daily_summary",
                "status": status.as_u16(),
                "response_body": body,
                "elapsed_ms": elapsed_ms,
            })
        );
        return Err(format!("API error ({}): {}", status, body));
    }

    let response_json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    let summary = response_json["choices"][0]["message"]["content"]
        .as_str()
        .ok_or("No content in response")?;

    tracing::info!(
        "{}",
        serde_json::json!({
            "event": "llm_response",
            "caller": "generate_daily_summary",
            "status": 200,
            "elapsed_ms": elapsed_ms,
            "usage": response_json.get("usage"),
            "model": response_json.get("model"),
            "response_id": response_json.get("id"),
            "content": summary,
        })
    );

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
