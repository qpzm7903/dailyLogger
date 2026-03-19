//! Ollama integration module for local LLM support.
//!
//! This module provides functions to detect and interact with Ollama endpoints,
//! allowing users to use locally deployed models for screenshot analysis and
//! daily summary generation.

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tauri::command;

/// Detailed information about an Ollama model.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OllamaModelInfo {
    pub name: String,
    pub modified_at: Option<String>,
    pub size: Option<u64>,
    pub digest: Option<String>,
    pub details: Option<OllamaModelDetails>,
}

/// Additional details about an Ollama model.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OllamaModelDetails {
    pub family: Option<String>,
    pub parameter_size: Option<String>,
    pub quantization_level: Option<String>,
}

/// Result structure for Ollama model list retrieval.
#[derive(Debug, Serialize, Deserialize)]
pub struct OllamaModelsResult {
    pub success: bool,
    pub models: Vec<String>,
    pub model_details: Vec<OllamaModelInfo>,
    pub message: String,
}

/// Result structure for API connection test.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConnectionTestResult {
    pub success: bool,
    pub message: String,
    pub latency_ms: Option<u64>,
    pub is_ollama: bool,
}

/// Detect if the given API base URL is an Ollama endpoint.
///
/// Ollama endpoints typically use port 11434, e.g.:
/// - `http://localhost:11434/v1`
/// - `http://127.0.0.1:11434/v1`
/// - `http://192.168.1.100:11434/v1`
///
/// Returns `true` if the URL matches known Ollama patterns.
pub fn is_ollama_endpoint(url: &str) -> bool {
    let url_lower = url.to_lowercase();

    // Standard Ollama port patterns
    url_lower.contains("localhost:11434")
        || url_lower.contains("127.0.0.1:11434")
        || url_lower.contains(":11434/v1")
        || url_lower.contains(":11434/")
        // Also check for :11434 at the end (without trailing slash or /v1)
        || url_lower.ends_with(":11434")
}

/// Get the list of installed models from an Ollama endpoint.
///
/// Uses Ollama's native API endpoint `/api/tags` to retrieve the model list.
/// The base_url should be the Ollama server URL (e.g., `http://localhost:11434`).
#[command]
pub async fn get_ollama_models(base_url: String) -> Result<OllamaModelsResult, String> {
    let client = Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    // Normalize URL: remove /v1 suffix if present, then append /api/tags
    let base = base_url.trim_end_matches('/').trim_end_matches("/v1");
    let url = format!("{}/api/tags", base);

    tracing::info!("Fetching Ollama models from: {}", url);

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format_connection_error(&e.to_string(), true))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Ok(OllamaModelsResult {
            success: false,
            models: vec![],
            model_details: vec![],
            message: format!("Ollama API error ({}): {}", status, body),
        });
    }

    let json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    let (models, model_details): (Vec<String>, Vec<OllamaModelInfo>) = json["models"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|m| {
                    let name = m["name"].as_str()?.to_string();
                    let details = OllamaModelInfo {
                        name: name.clone(),
                        modified_at: m["modified_at"].as_str().map(String::from),
                        size: m["size"].as_u64(),
                        digest: m["digest"].as_str().map(String::from),
                        details: m["details"].as_object().map(|d| OllamaModelDetails {
                            family: d.get("family").and_then(|v| v.as_str()).map(String::from),
                            parameter_size: d
                                .get("parameter_size")
                                .and_then(|v| v.as_str())
                                .map(String::from),
                            quantization_level: d
                                .get("quantization_level")
                                .and_then(|v| v.as_str())
                                .map(String::from),
                        }),
                    };
                    Some((name, details))
                })
                .unzip()
        })
        .unwrap_or_default();

    let model_count = models.len();
    tracing::info!("Found {} Ollama models: {:?}", model_count, models);

    Ok(OllamaModelsResult {
        success: true,
        models,
        model_details,
        message: format!("Found {} models", model_count),
    })
}

/// Result structure for model pull operation.
#[derive(Debug, Serialize, Deserialize)]
pub struct PullModelResult {
    pub success: bool,
    pub message: String,
    pub status: String,
}

/// Result structure for model delete operation.
#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteModelResult {
    pub success: bool,
    pub message: String,
}

/// Information about a currently running Ollama model.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RunningModelInfo {
    pub name: String,
    pub model: String,
    pub size: Option<u64>,
    pub digest: Option<String>,
    pub details: Option<OllamaModelDetails>,
    pub expires_at: Option<String>,
    pub size_vram: Option<u64>,
}

/// Result structure for running models retrieval.
#[derive(Debug, Serialize, Deserialize)]
pub struct RunningModelsResult {
    pub success: bool,
    pub running_models: Vec<RunningModelInfo>,
    pub message: String,
}

/// Pull (download) a model from Ollama registry.
///
/// Uses Ollama's native API endpoint `/api/pull` to download a model.
/// This is an async operation that may take a long time for large models.
///
/// # Arguments
/// * `base_url` - The Ollama server URL (e.g., `http://localhost:11434`)
/// * `model_name` - The name of the model to pull (e.g., `llama3.2`)
/// * `quantization` - Optional quantization level (e.g., `q4_0`, `q5_0`, `q8_0`)
///
/// # Example
/// ```ignore
/// // Pull with default quantization
/// pull_ollama_model("http://localhost:11434", "llama3.2", None).await?;
///
/// // Pull with specific quantization
/// pull_ollama_model("http://localhost:11434", "llama3.2", Some("q4_0".to_string())).await?;
/// ```
#[command]
pub async fn pull_ollama_model(
    base_url: String,
    model_name: String,
    quantization: Option<String>,
) -> Result<PullModelResult, String> {
    let client = Client::builder()
        .timeout(Duration::from_secs(600)) // 10 minutes timeout for large models
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    // Normalize URL: remove /v1 suffix if present, then append /api/pull
    let base = base_url.trim_end_matches('/').trim_end_matches("/v1");
    let url = format!("{}/api/pull", base);

    tracing::info!(
        "Pulling Ollama model '{}' (quantization: {:?}) from: {}",
        model_name,
        quantization,
        url
    );

    let mut request_body = serde_json::json!({
        "name": model_name,
        "stream": false
    });

    // Add quantization if specified
    if let Some(ref q) = quantization {
        request_body["quantization"] = serde_json::json!(q);
    }

    let response = client
        .post(&url)
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await
        .map_err(|e| format_connection_error(&e.to_string(), true))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Ok(PullModelResult {
            success: false,
            message: format!("Failed to pull model ({}): {}", status, body),
            status: "error".to_string(),
        });
    }

    // Parse the response to get status
    let json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    let status = json["status"].as_str().unwrap_or("completed").to_string();
    let message = if status == "success" || status == "completed" {
        format!("Model '{}' pulled successfully", model_name)
    } else {
        format!("Model pull status: {}", status)
    };

    tracing::info!("Pull result: {}", message);

    Ok(PullModelResult {
        success: true,
        message,
        status,
    })
}

/// Delete a model from Ollama.
///
/// Uses Ollama's native API endpoint `/api/delete` to remove an installed model.
#[command]
pub async fn delete_ollama_model(
    base_url: String,
    model_name: String,
) -> Result<DeleteModelResult, String> {
    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    // Normalize URL: remove /v1 suffix if present, then append /api/delete
    let base = base_url.trim_end_matches('/').trim_end_matches("/v1");
    let url = format!("{}/api/delete", base);

    tracing::info!("Deleting Ollama model '{}' from: {}", model_name, url);

    let request_body = serde_json::json!({
        "name": model_name
    });

    let response = client
        .delete(&url)
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await
        .map_err(|e| format_connection_error(&e.to_string(), true))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Ok(DeleteModelResult {
            success: false,
            message: format!("Failed to delete model ({}): {}", status, body),
        });
    }

    tracing::info!("Model '{}' deleted successfully", model_name);

    Ok(DeleteModelResult {
        success: true,
        message: format!("Model '{}' deleted successfully", model_name),
    })
}

/// Get the list of currently running models from an Ollama endpoint.
///
/// Uses Ollama's native API endpoint `/api/ps` to retrieve currently loaded models.
/// This helps users see which models are loaded in memory and their resource usage.
#[command]
pub async fn get_running_models(base_url: String) -> Result<RunningModelsResult, String> {
    let client = Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    // Normalize URL: remove /v1 suffix if present, then append /api/ps
    let base = base_url.trim_end_matches('/').trim_end_matches("/v1");
    let url = format!("{}/api/ps", base);

    tracing::info!("Fetching running models from: {}", url);

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format_connection_error(&e.to_string(), true))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Ok(RunningModelsResult {
            success: false,
            running_models: vec![],
            message: format!("Ollama API error ({}): {}", status, body),
        });
    }

    let json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    let running_models: Vec<RunningModelInfo> = json["models"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|m| {
                    Some(RunningModelInfo {
                        name: m["name"].as_str()?.to_string(),
                        model: m["model"].as_str().unwrap_or("").to_string(),
                        size: m["size"].as_u64(),
                        digest: m["digest"].as_str().map(String::from),
                        details: m["details"].as_object().map(|d| OllamaModelDetails {
                            family: d.get("family").and_then(|v| v.as_str()).map(String::from),
                            parameter_size: d
                                .get("parameter_size")
                                .and_then(|v| v.as_str())
                                .map(String::from),
                            quantization_level: d
                                .get("quantization_level")
                                .and_then(|v| v.as_str())
                                .map(String::from),
                        }),
                        expires_at: m["expires_at"].as_str().map(String::from),
                        size_vram: m["size_vram"].as_u64(),
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    let model_count = running_models.len();
    tracing::info!("Found {} running models", model_count);

    Ok(RunningModelsResult {
        success: true,
        running_models,
        message: format!("Found {} running models", model_count),
    })
}

/// Test API connection for both OpenAI and Ollama endpoints.
///
/// This function detects if the endpoint is Ollama and adjusts the request accordingly:
/// - For Ollama: No API key required, uses empty string if not provided
/// - For OpenAI: API key is required
#[command]
pub async fn test_api_connection_with_ollama(
    api_base_url: String,
    api_key: Option<String>,
    model_name: String,
) -> Result<ConnectionTestResult, String> {
    let is_ollama = is_ollama_endpoint(&api_base_url);
    let effective_api_key = api_key.unwrap_or_default();

    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let start = std::time::Instant::now();

    // Send a simple "Say 'ok'" request
    let request_body = serde_json::json!({
        "model": model_name,
        "messages": [{"role": "user", "content": "Say 'ok'"}],
        "max_tokens": 5
    });

    let url = if api_base_url.ends_with('/') {
        format!("{}chat/completions", api_base_url)
    } else {
        format!("{}/chat/completions", api_base_url)
    };

    let mut request = client
        .post(&url)
        .header("Content-Type", "application/json")
        .json(&request_body);

    // Only add Authorization header if API key is provided (not required for Ollama)
    if !effective_api_key.is_empty() {
        request = request.header("Authorization", format!("Bearer {}", effective_api_key));
    }

    let response = request.send().await;

    match response {
        Ok(resp) if resp.status().is_success() => Ok(ConnectionTestResult {
            success: true,
            message: if is_ollama {
                "Ollama 连接成功".to_string()
            } else {
                "连接成功".to_string()
            },
            latency_ms: Some(start.elapsed().as_millis() as u64),
            is_ollama,
        }),
        Ok(resp) => {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            let message = if status.as_u16() == 401 {
                if is_ollama {
                    "Ollama 认证失败（某些配置可能需要 API Key）".to_string()
                } else {
                    "API Key 无效".to_string()
                }
            } else if status.as_u16() == 404 {
                if is_ollama {
                    "模型不存在，请使用 'ollama pull <model>' 下载模型".to_string()
                } else {
                    "API 端点不存在或模型不支持".to_string()
                }
            } else {
                format!("API 错误 ({}): {}", status, body)
            };
            Ok(ConnectionTestResult {
                success: false,
                message,
                latency_ms: Some(start.elapsed().as_millis() as u64),
                is_ollama,
            })
        }
        Err(e) => {
            let message = format_connection_error(&e.to_string(), is_ollama);
            Ok(ConnectionTestResult {
                success: false,
                message,
                latency_ms: None,
                is_ollama,
            })
        }
    }
}

/// Format connection error message with Ollama-specific guidance.
pub fn format_connection_error(error: &str, is_ollama: bool) -> String {
    if error.contains("connection refused") || error.contains("Connection refused") {
        if is_ollama {
            "Ollama 服务未运行，请先启动 Ollama (运行 'ollama serve' 或打开 Ollama 应用)"
                .to_string()
        } else {
            "无法连接到 API 服务器，请检查网络和 Base URL".to_string()
        }
    } else if error.contains("timed out") || error.contains("timeout") {
        if is_ollama {
            "Ollama 连接超时，请确认 Ollama 服务正在运行".to_string()
        } else {
            "连接超时，请检查网络或 API 地址".to_string()
        }
    } else if error.contains("dns") || error.contains("resolve") {
        "无法解析服务器地址，请检查 URL 是否正确".to_string()
    } else {
        format!("连接失败: {}", error)
    }
}

/// Parameters for creating a custom Ollama model.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateModelParams {
    /// Name for the new model.
    pub name: String,
    /// Base model to use (e.g., "llama3.2").
    pub from: String,
    /// System prompt for the model.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
    /// Model parameters (temperature, num_ctx, etc.).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<std::collections::HashMap<String, serde_json::Value>>,
    /// Template for the model (overrides base model's template).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template: Option<String>,
}

/// Result structure for model create operation.
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateModelResult {
    pub success: bool,
    pub message: String,
    pub model_name: String,
}

/// Create a custom model in Ollama.
///
/// Uses Ollama's native API endpoint `/api/create` to create a custom model
/// from a base model with optional custom system prompt and parameters.
///
/// # Example
/// ```ignore
/// let params = CreateModelParams {
///     name: "my-custom-llama".to_string(),
///     from: "llama3.2".to_string(),
///     system: Some("You are a helpful assistant.".to_string()),
///     parameters: Some({
///         let mut p = HashMap::new();
///         p.insert("temperature".to_string(), json!(0.7));
///         p
///     }),
///     template: None,
/// };
/// let result = create_ollama_model("http://localhost:11434", params).await?;
/// ```
#[command]
pub async fn create_ollama_model(
    base_url: String,
    params: CreateModelParams,
) -> Result<CreateModelResult, String> {
    let client = Client::builder()
        .timeout(Duration::from_secs(300)) // 5 minutes timeout for model creation
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    // Normalize URL: remove /v1 suffix if present, then append /api/create
    let base = base_url.trim_end_matches('/').trim_end_matches("/v1");
    let url = format!("{}/api/create", base);

    let model_name = params.name.clone();
    tracing::info!(
        "Creating Ollama model '{}' from '{}' at: {}",
        model_name,
        params.from,
        url
    );

    // Build request body
    let mut request_body = serde_json::json!({
        "name": params.name,
        "from": params.from,
        "stream": false,
    });

    // Add optional fields
    if let Some(system) = &params.system {
        request_body["system"] = serde_json::json!(system);
    }
    if let Some(parameters) = &params.parameters {
        request_body["parameters"] = serde_json::json!(parameters);
    }
    if let Some(template) = &params.template {
        request_body["template"] = serde_json::json!(template);
    }

    let response = client
        .post(&url)
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await
        .map_err(|e| format_connection_error(&e.to_string(), true))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Ok(CreateModelResult {
            success: false,
            message: format!("Failed to create model ({}): {}", status, body),
            model_name,
        });
    }

    tracing::info!("Model '{}' created successfully", model_name);

    Ok(CreateModelResult {
        success: true,
        message: format!("Model '{}' created successfully", model_name),
        model_name,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Tests for is_ollama_endpoint ──

    #[test]
    fn is_ollama_endpoint_detects_localhost_11434() {
        assert!(is_ollama_endpoint("http://localhost:11434/v1"));
        assert!(is_ollama_endpoint("http://localhost:11434"));
        assert!(is_ollama_endpoint("http://localhost:11434/"));
    }

    #[test]
    fn is_ollama_endpoint_detects_127_0_0_1_11434() {
        assert!(is_ollama_endpoint("http://127.0.0.1:11434/v1"));
        assert!(is_ollama_endpoint("http://127.0.0.1:11434"));
    }

    #[test]
    fn is_ollama_endpoint_detects_custom_host_with_11434() {
        assert!(is_ollama_endpoint("http://192.168.1.100:11434/v1"));
        assert!(is_ollama_endpoint("http://myserver.local:11434/v1"));
        assert!(is_ollama_endpoint("https://ollama.example.com:11434/v1"));
    }

    #[test]
    fn is_ollama_endpoint_rejects_openai_url() {
        assert!(!is_ollama_endpoint("https://api.openai.com/v1"));
        assert!(!is_ollama_endpoint("https://api.openai.com"));
    }

    #[test]
    fn is_ollama_endpoint_rejects_other_ports() {
        assert!(!is_ollama_endpoint("http://localhost:8080/v1"));
        assert!(!is_ollama_endpoint("http://localhost:3000"));
        assert!(!is_ollama_endpoint("https://api.example.com:443/v1"));
    }

    #[test]
    fn is_ollama_endpoint_case_insensitive() {
        assert!(is_ollama_endpoint("HTTP://LOCALHOST:11434/V1"));
        assert!(is_ollama_endpoint("Http://Localhost:11434"));
    }

    #[test]
    fn is_ollama_endpoint_empty_string() {
        assert!(!is_ollama_endpoint(""));
    }

    // ── Tests for format_connection_error ──

    #[test]
    fn format_error_ollama_connection_refused() {
        let error = format_connection_error("connection refused", true);
        assert!(error.contains("Ollama 服务未运行"));
        assert!(error.contains("ollama serve"));
    }

    #[test]
    fn format_error_openai_connection_refused() {
        let error = format_connection_error("connection refused", false);
        assert!(error.contains("无法连接到 API 服务器"));
    }

    #[test]
    fn format_error_ollama_timeout() {
        let error = format_connection_error("connection timed out", true);
        assert!(error.contains("Ollama 连接超时"));
    }

    #[test]
    fn format_error_openai_timeout() {
        let error = format_connection_error("operation timed out", false);
        assert!(error.contains("连接超时"));
    }

    #[test]
    fn format_error_dns_error() {
        let error = format_connection_error("dns error: failed to resolve", true);
        assert!(error.contains("无法解析服务器地址"));
    }

    #[test]
    fn format_error_generic() {
        let error = format_connection_error("some other error", true);
        assert!(error.contains("连接失败"));
        assert!(error.contains("some other error"));
    }

    // ── Tests for PullModelResult and DeleteModelResult structs ──

    #[test]
    fn pull_model_result_serialization() {
        let result = PullModelResult {
            success: true,
            message: "Model pulled".to_string(),
            status: "completed".to_string(),
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"success\":true"));
        assert!(json.contains("\"message\":\"Model pulled\""));
        assert!(json.contains("\"status\":\"completed\""));
    }

    #[test]
    fn delete_model_result_serialization() {
        let result = DeleteModelResult {
            success: true,
            message: "Model deleted".to_string(),
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"success\":true"));
        assert!(json.contains("\"message\":\"Model deleted\""));
    }

    #[test]
    fn pull_model_result_deserialization() {
        let json = r#"{"success":false,"message":"Failed to pull","status":"error"}"#;
        let result: PullModelResult = serde_json::from_str(json).unwrap();
        assert!(!result.success);
        assert_eq!(result.message, "Failed to pull");
        assert_eq!(result.status, "error");
    }

    #[test]
    fn pull_model_with_quantization_request_body() {
        // Test that the request body is correctly formed with quantization
        let mut request_body = serde_json::json!({
            "name": "llama3.2",
            "stream": false
        });
        request_body["quantization"] = serde_json::json!("q4_0");

        assert_eq!(request_body["name"], "llama3.2");
        assert_eq!(request_body["stream"], false);
        assert_eq!(request_body["quantization"], "q4_0");
    }

    #[test]
    fn pull_model_without_quantization_request_body() {
        // Test that the request body is correctly formed without quantization
        let request_body = serde_json::json!({
            "name": "llama3.2",
            "stream": false
        });

        assert_eq!(request_body["name"], "llama3.2");
        assert_eq!(request_body["stream"], false);
        assert!(request_body.get("quantization").is_none());
    }

    #[test]
    fn pull_model_common_quantization_values() {
        // Test that common quantization values are valid JSON strings
        let quantization_levels = vec!["q4_0", "q4_1", "q5_0", "q5_1", "q8_0", "f16"];
        for q in quantization_levels {
            let json = serde_json::json!({"quantization": q});
            assert_eq!(json["quantization"], q);
        }
    }

    #[test]
    fn delete_model_result_deserialization() {
        let json = r#"{"success":false,"message":"Model not found"}"#;
        let result: DeleteModelResult = serde_json::from_str(json).unwrap();
        assert!(!result.success);
        assert_eq!(result.message, "Model not found");
    }

    // ── Tests for RunningModelInfo and RunningModelsResult structs ──

    #[test]
    fn running_model_info_serialization() {
        let info = RunningModelInfo {
            name: "llama3:latest".to_string(),
            model: "llama3:latest".to_string(),
            size: Some(4661224676),
            digest: Some("abc123".to_string()),
            details: Some(OllamaModelDetails {
                family: Some("llama".to_string()),
                parameter_size: Some("8B".to_string()),
                quantization_level: Some("Q4_0".to_string()),
            }),
            expires_at: Some("2024-01-01T00:00:00Z".to_string()),
            size_vram: Some(4000000000),
        };
        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("\"name\":\"llama3:latest\""));
        assert!(json.contains("\"size_vram\":4000000000"));
    }

    #[test]
    fn running_model_info_deserialization() {
        let json = r#"{
            "name": "llama3:latest",
            "model": "llama3:latest",
            "size": 4661224676,
            "digest": "abc123",
            "details": {
                "family": "llama",
                "parameter_size": "8B",
                "quantization_level": "Q4_0"
            },
            "expires_at": "2024-01-01T00:00:00Z",
            "size_vram": 4000000000
        }"#;
        let info: RunningModelInfo = serde_json::from_str(json).unwrap();
        assert_eq!(info.name, "llama3:latest");
        assert_eq!(info.size, Some(4661224676));
        assert_eq!(info.size_vram, Some(4000000000));
        assert!(info.details.is_some());
        let details = info.details.unwrap();
        assert_eq!(details.family, Some("llama".to_string()));
        assert_eq!(details.parameter_size, Some("8B".to_string()));
    }

    #[test]
    fn running_models_result_serialization() {
        let result = RunningModelsResult {
            success: true,
            running_models: vec![RunningModelInfo {
                name: "test:model".to_string(),
                model: "test:model".to_string(),
                size: None,
                digest: None,
                details: None,
                expires_at: None,
                size_vram: None,
            }],
            message: "Found 1 running model".to_string(),
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"success\":true"));
        assert!(json.contains("\"name\":\"test:model\""));
    }

    #[test]
    fn running_models_result_deserialization() {
        let json = r#"{
            "success": false,
            "running_models": [],
            "message": "No running models"
        }"#;
        let result: RunningModelsResult = serde_json::from_str(json).unwrap();
        assert!(!result.success);
        assert!(result.running_models.is_empty());
        assert_eq!(result.message, "No running models");
    }

    // ── Tests for CreateModelParams and CreateModelResult structs ──

    #[test]
    fn create_model_params_minimal() {
        let params = CreateModelParams {
            name: "my-model".to_string(),
            from: "llama3.2".to_string(),
            system: None,
            parameters: None,
            template: None,
        };
        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("\"name\":\"my-model\""));
        assert!(json.contains("\"from\":\"llama3.2\""));
        assert!(!json.contains("\"system\""));
        assert!(!json.contains("\"parameters\""));
        assert!(!json.contains("\"template\""));
    }

    #[test]
    fn create_model_params_with_system() {
        let params = CreateModelParams {
            name: "custom-model".to_string(),
            from: "llama3.2".to_string(),
            system: Some("You are a helpful assistant.".to_string()),
            parameters: None,
            template: None,
        };
        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("\"system\":\"You are a helpful assistant.\""));
    }

    #[test]
    fn create_model_params_with_parameters() {
        use serde_json::json;
        use std::collections::HashMap;

        let mut params_map = HashMap::new();
        params_map.insert("temperature".to_string(), json!(0.7));
        params_map.insert("num_ctx".to_string(), json!(4096));

        let params = CreateModelParams {
            name: "tuned-model".to_string(),
            from: "llama3.2".to_string(),
            system: None,
            parameters: Some(params_map),
            template: None,
        };
        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("\"temperature\":0.7"));
        assert!(json.contains("\"num_ctx\":4096"));
    }

    #[test]
    fn create_model_params_full() {
        use serde_json::json;
        use std::collections::HashMap;

        let mut params_map = HashMap::new();
        params_map.insert("temperature".to_string(), json!(0.5));

        let params = CreateModelParams {
            name: "full-model".to_string(),
            from: "llama3.2".to_string(),
            system: Some("Be concise.".to_string()),
            parameters: Some(params_map),
            template: Some("{{ .System }}\n{{ .Prompt }}".to_string()),
        };
        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("\"name\":\"full-model\""));
        assert!(json.contains("\"from\":\"llama3.2\""));
        assert!(json.contains("\"system\":\"Be concise.\""));
        assert!(json.contains("\"temperature\":0.5"));
        assert!(json.contains("\"template\":\"{{ .System }}\\n{{ .Prompt }}\""));
    }

    #[test]
    fn create_model_result_serialization() {
        let result = CreateModelResult {
            success: true,
            message: "Model created".to_string(),
            model_name: "my-model".to_string(),
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"success\":true"));
        assert!(json.contains("\"message\":\"Model created\""));
        assert!(json.contains("\"model_name\":\"my-model\""));
    }

    #[test]
    fn create_model_result_deserialization() {
        let json = r#"{"success":false,"message":"Base model not found","model_name":"test"}"#;
        let result: CreateModelResult = serde_json::from_str(json).unwrap();
        assert!(!result.success);
        assert_eq!(result.message, "Base model not found");
        assert_eq!(result.model_name, "test");
    }
}
