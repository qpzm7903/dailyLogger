//! Ollama integration module for local LLM support.
//!
//! This module provides functions to detect and interact with Ollama endpoints,
//! allowing users to use locally deployed models for screenshot analysis and
//! daily summary generation.

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tauri::command;

/// Result structure for Ollama model list retrieval.
#[derive(Debug, Serialize, Deserialize)]
pub struct OllamaModelsResult {
    pub success: bool,
    pub models: Vec<String>,
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
            message: format!("Ollama API error ({}): {}", status, body),
        });
    }

    let json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    let models: Vec<String> = json["models"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|m| m["name"].as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    let model_count = models.len();
    tracing::info!("Found {} Ollama models: {:?}", model_count, models);

    Ok(OllamaModelsResult {
        success: true,
        models,
        message: format!("Found {} models", model_count),
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
}
