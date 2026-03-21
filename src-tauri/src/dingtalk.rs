//! DingTalk integration for sending report notifications.
//!
//! This module provides functionality to send report summaries to DingTalk groups
//! using Robot Webhooks.

use crate::create_http_client;
use tauri::command;

/// Check if DingTalk is configured in settings
pub fn is_dingtalk_configured(settings: &crate::memory_storage::Settings) -> bool {
    settings
        .dingtalk_webhook_url
        .as_ref()
        .map(|url| !url.is_empty())
        .unwrap_or(false)
}

/// Send a report notification to DingTalk via Robot Webhook.
///
/// Returns true on success, false if DingTalk is not configured or the send fails.
pub async fn send_to_dingtalk(
    settings: &crate::memory_storage::Settings,
    title: &str,
    content: &str,
) -> Option<bool> {
    let webhook_url = settings.dingtalk_webhook_url.as_ref()?;

    if webhook_url.is_empty() {
        return None;
    }

    let client = create_http_client(webhook_url, 30).ok()?;

    // Truncate content if too long (DingTalk has limits around 20000 characters for markdown)
    let truncated_content = if content.len() > 19000 {
        format!("{}...\n_(内容已截断)_", &content[..19000])
    } else {
        content.to_string()
    };

    // Format message with DingTalk Markdown format
    // Reference: https://open.dingtalk.com/document/robots/custom-robot-access
    let body = serde_json::json!({
        "msgtype": "markdown",
        "markdown": {
            "title": title,
            "text": format!("### {}\n\n{}", title, truncated_content)
        }
    });

    let response = client
        .post(webhook_url)
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await;

    match response {
        Ok(resp) => {
            if resp.status().is_success() {
                // DingTalk webhook returns {"errcode":0,"errmsg":"ok"} on success
                match resp.json::<serde_json::Value>().await {
                    Ok(json) => {
                        if json.get("errcode").and_then(|v| v.as_i64()) == Some(0) {
                            tracing::info!("Report sent to DingTalk: {}", title);
                            Some(true)
                        } else {
                            let errmsg = json
                                .get("errmsg")
                                .and_then(|v| v.as_str())
                                .unwrap_or("unknown error");
                            tracing::warn!("DingTalk API error: {}", errmsg);
                            Some(false)
                        }
                    }
                    Err(e) => {
                        tracing::warn!("Failed to parse DingTalk response: {}", e);
                        Some(false)
                    }
                }
            } else {
                let status = resp.status();
                let error_text = resp.text().await.unwrap_or_default();
                tracing::warn!("Failed to send to DingTalk: {} - {}", status, error_text);
                Some(false)
            }
        }
        Err(e) => {
            tracing::warn!("Failed to call DingTalk webhook: {}", e);
            Some(false)
        }
    }
}

/// Test DingTalk webhook connection
/// Returns Ok(true) if connection is successful, Ok(false) if DingTalk is not configured,
/// or Err with error message if connection fails.
#[command]
pub async fn test_dingtalk_connection() -> Result<bool, String> {
    let settings = crate::memory_storage::get_settings_sync()?;

    let webhook_url = match settings.dingtalk_webhook_url {
        Some(ref url) if !url.is_empty() => url,
        _ => return Ok(false), // Not configured
    };

    let client = create_http_client(webhook_url, 30)
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    // Send a test message
    let body = serde_json::json!({
        "msgtype": "text",
        "text": {
            "content": "DailyLogger 连接测试成功"
        }
    });

    let response = client
        .post(webhook_url)
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Connection error: {}", e))?;

    if response.status().is_success() {
        match response.json::<serde_json::Value>().await {
            Ok(json) => {
                if json.get("errcode").and_then(|v| v.as_i64()) == Some(0) {
                    Ok(true)
                } else {
                    let errmsg = json
                        .get("errmsg")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown error");
                    Err(format!("DingTalk API error: {}", errmsg))
                }
            }
            Err(e) => Err(format!("Failed to parse response: {}", e)),
        }
    } else {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        Err(format!("DingTalk API error: {} - {}", status, error_text))
    }
}

/// Send a report to DingTalk synchronously (wrapper for async function)
/// This is used in report generation functions that need to send to DingTalk
pub fn send_to_dingtalk_sync(
    settings: &crate::memory_storage::Settings,
    title: &str,
    content: &str,
) -> Option<bool> {
    // For synchronous contexts, we use tokio::runtime
    let rt = tokio::runtime::Runtime::new().ok()?;
    rt.block_on(send_to_dingtalk(settings, title, content))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_dingtalk_configured_returns_false_when_no_webhook() {
        let settings = crate::memory_storage::Settings {
            dingtalk_webhook_url: None,
            ..Default::default()
        };
        assert!(!is_dingtalk_configured(&settings));
    }

    #[test]
    fn is_dingtalk_configured_returns_false_when_empty() {
        let settings = crate::memory_storage::Settings {
            dingtalk_webhook_url: Some("".to_string()),
            ..Default::default()
        };
        assert!(!is_dingtalk_configured(&settings));
    }

    #[test]
    fn is_dingtalk_configured_returns_true_when_configured() {
        let settings = crate::memory_storage::Settings {
            dingtalk_webhook_url: Some(
                "https://oapi.dingtalk.com/robot/send?access_token=XXX".to_string(),
            ),
            ..Default::default()
        };
        assert!(is_dingtalk_configured(&settings));
    }
}
