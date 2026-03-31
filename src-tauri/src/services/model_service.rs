//! Model service - Business logic for model information retrieval
//!
//! This module contains the business logic for querying AI model information.
//! Commands should delegate to these service functions rather than implementing logic directly.

use crate::errors::AppResult;
use crate::memory_storage::ModelInfo;

/// Get model information including context window from an OpenAI-compatible API.
///
/// This function handles the HTTP communication with the model API to retrieve
/// model metadata such as context window size.
pub async fn get_model_info_service(
    api_base_url: String,
    api_key: String,
    model_name: String,
) -> AppResult<ModelInfo> {
    // OpenAI compatible API /models endpoint
    let url = if api_base_url.ends_with('/') {
        format!("{}models/{}", api_base_url, model_name)
    } else {
        format!("{}/models/{}", api_base_url, model_name)
    };

    // Create HTTP client with proxy bypass for local URLs
    let client = crate::create_http_client(&url, 30)?;

    let response = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .send()
        .await;

    match response {
        Ok(resp) if resp.status().is_success() => {
            let json: serde_json::Value = resp.json().await.unwrap_or(serde_json::json!({}));

            // OpenAI returns format: {"id": "gpt-4o", "context_window": 128000}
            // Or in some APIs it's max_tokens
            let context_window = json
                .get("context_window")
                .or_else(|| json.get("max_tokens"))
                .or_else(|| {
                    // Some APIs return it in model_info
                    json.get("model_info")
                        .and_then(|mi| mi.get("context_window"))
                })
                .and_then(|v| v.as_u64());

            Ok(ModelInfo {
                model_id: model_name,
                context_window,
                error: None,
            })
        }
        Ok(resp) => {
            let status = resp.status();
            Ok(ModelInfo {
                model_id: model_name,
                context_window: None,
                error: Some(format!("无法获取模型信息 (状态: {})", status)),
            })
        }
        Err(e) => Ok(ModelInfo {
            model_id: model_name,
            context_window: None,
            error: Some(format!("请求失败: {}", e)),
        }),
    }
}
