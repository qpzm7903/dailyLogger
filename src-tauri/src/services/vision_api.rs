//! Vision API helpers shared by session_service.
//!
//! Encapsulates the common pattern of:
//!   1. Loading API config (model, headers, proxy)
//!   2. Encoding screenshots as base64
//!   3. Building multi-image requests
//!   4. Calling the Vision chat/completions endpoint with retry logic

use crate::infrastructure::retry;
use crate::memory_storage::SessionScreenshot;
use crate::services::session_service::SessionAnalysisResponse;

// STAB-001: Retry configuration for Vision API calls
const VISION_MAX_RETRIES: u32 = 3;
const VISION_INITIAL_RETRY_DELAY_MS: u64 = 1000;
const VISION_MAX_RETRY_DELAY_MS: u64 = 10000;

/// Default prompt for session batch analysis
pub const DEFAULT_SESSION_ANALYSIS_PROMPT: &str = r#"你是一个工作分析助手。用户在一段时间内连续工作了 N 分钟，期间截取了多张屏幕截图。

请分析这些截图，理解用户在这段时间内的工作内容，返回以下 JSON 格式：

{
  "per_screenshot_analysis": [
    {
      "timestamp": "2026-03-22T10:05:00Z",
      "current_focus": "正在编写 Rust 代码",
      "active_software": "VS Code",
      "context_keywords": ["Rust", "Tauri", "异步"],
      "tags": ["开发"]
    }
  ],
  "session_summary": "用户在这段时间主要进行 Rust 后端开发，实现了工作时段管理功能...",
  "context_for_next": "正在开发 session_manager 模块，下一步需要实现 analyze_session 函数..."
}

注意：
1. per_screenshot_analysis 数组长度必须与输入截图数量一致
2. session_summary 应概括整个时段的工作内容
3. context_for_next 用于帮助下一时段理解连续性工作
4. tags 从以下列表选择 1-3 个最相关的: ["开发", "会议", "写作", "学习", "研究", "沟通", "规划", "文档", "测试", "设计"]

上一时段上下文（如有）：
{previous_context}

返回纯 JSON，不要添加任何其他文字。"#;

/// Read and encode screenshot as base64
pub fn encode_screenshot(path: &str) -> Result<String, String> {
    let bytes =
        std::fs::read(path).map_err(|e| format!("Failed to read screenshot {}: {}", path, e))?;
    Ok(base64::Engine::encode(
        &base64::engine::general_purpose::STANDARD,
        &bytes,
    ))
}

/// Build multi-image request for Vision API
pub fn build_multi_image_request(
    screenshots: &[SessionScreenshot],
    previous_context: Option<&str>,
    config: &crate::synthesis::ApiConfig,
) -> Result<serde_json::Value, String> {
    let prompt = DEFAULT_SESSION_ANALYSIS_PROMPT
        .replace("{previous_context}", previous_context.unwrap_or("无"));

    let mut content: Vec<serde_json::Value> = vec![serde_json::json!({
        "type": "text",
        "text": prompt
    })];

    for screenshot in screenshots {
        let base64_image = encode_screenshot(&screenshot.screenshot_path)?;
        content.push(serde_json::json!({
            "type": "image_url",
            "image_url": {
                "url": format!("data:image/png;base64,{}", base64_image)
            }
        }));
    }

    Ok(serde_json::json!({
        "model": config.model_name(),
        "messages": [{
            "role": "user",
            "content": content
        }],
        "max_tokens": 4000
    }))
}

/// Call Vision API for batch analysis
pub async fn call_vision_api_batch(
    request: &serde_json::Value,
    config: &crate::synthesis::ApiConfig,
) -> Result<SessionAnalysisResponse, String> {
    let endpoint = format!("{}/chat/completions", config.api_base_url());
    let client =
        crate::create_http_client_with_proxy(&endpoint, 180, Some(config.proxy_config().clone()))?;

    let masked_key = crate::mask_api_key(config.api_key());

    tracing::info!(
        "{}",
        serde_json::json!({
            "event": "session_analysis_request",
            "endpoint": endpoint,
            "model": config.model_name(),
            "api_key_masked": masked_key,
        })
    );

    let start = std::time::Instant::now();
    let mut request_builder = client
        .post(&endpoint)
        .header("Content-Type", "application/json")
        .json(request);

    // Check if custom headers contain auth
    let has_custom_auth = config
        .custom_headers()
        .iter()
        .any(|h| h.key.to_lowercase() == "authorization" || h.key.to_lowercase() == "api-key");

    if !config.api_key().is_empty() && !has_custom_auth {
        request_builder =
            request_builder.header("Authorization", format!("Bearer {}", config.api_key()));
    }

    for header in config.custom_headers() {
        request_builder = request_builder.header(&header.key, &header.value);
    }

    let response = request_builder.send().await.map_err(|e| {
        tracing::error!("Session analysis API call failed: {}", e);
        format!("API request failed: {}", e)
    })?;

    let elapsed_ms = start.elapsed().as_millis();

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        tracing::error!(
            "{}",
            serde_json::json!({
                "event": "session_analysis_error",
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

    let content = response_json["choices"][0]["message"]["content"]
        .as_str()
        .ok_or("No content in response")?;

    tracing::info!(
        "{}",
        serde_json::json!({
            "event": "session_analysis_response",
            "elapsed_ms": elapsed_ms,
            "usage": response_json.get("usage"),
        })
    );

    // Strip markdown code fences if present
    let content = content.trim();
    let content = if let Some(inner) = content
        .strip_prefix("```json")
        .or_else(|| content.strip_prefix("```"))
    {
        inner.trim_end_matches("```").trim()
    } else {
        content
    };

    let analysis: SessionAnalysisResponse = serde_json::from_str(content)
        .map_err(|e| format!("Failed to parse analysis JSON: {}. Content: {}", e, content))?;

    Ok(analysis)
}

/// Wrapper for call_vision_api_batch with retry logic
pub async fn call_vision_api_batch_with_retry(
    request: &serde_json::Value,
    config: &crate::synthesis::ApiConfig,
) -> Result<SessionAnalysisResponse, String> {
    let mut last_error = String::new();

    for attempt in 1..=VISION_MAX_RETRIES {
        match call_vision_api_batch(request, config).await {
            Ok(result) => return Ok(result),
            Err(e) => {
                last_error = e.clone();
                if attempt < VISION_MAX_RETRIES && retry::is_retryable_error(&e) {
                    let delay = retry::calculate_retry_delay(
                        attempt,
                        VISION_INITIAL_RETRY_DELAY_MS,
                        VISION_MAX_RETRY_DELAY_MS,
                    );
                    tracing::warn!(
                        "Vision API call failed (attempt {}/{}), retrying in {}ms: {}",
                        attempt,
                        VISION_MAX_RETRIES,
                        delay,
                        e
                    );
                    tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
                }
            }
        }
    }

    Err(last_error)
}
