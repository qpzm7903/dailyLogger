//! Slack integration for sending report notifications.
//!
//! This module provides functionality to send report summaries to Slack channels
//! using Incoming Webhooks.

use crate::create_http_client;
use crate::errors::AppResult;
use tauri::command;

/// Check if Slack is configured in settings
pub fn is_slack_configured(settings: &crate::memory_storage::Settings) -> bool {
    settings
        .slack_webhook_url
        .as_ref()
        .map(|url| !url.is_empty())
        .unwrap_or(false)
}

/// Send a report notification to Slack via Incoming Webhook.
///
/// Returns true on success, false if Slack is not configured or the send fails.
pub async fn send_to_slack(
    settings: &crate::memory_storage::Settings,
    title: &str,
    content: &str,
) -> Option<bool> {
    let webhook_url = settings.slack_webhook_url.as_ref()?;

    if webhook_url.is_empty() {
        return None;
    }

    let client = create_http_client(webhook_url, 30).ok()?;

    // Truncate content if too long (Slack has a 4000 character limit for message text)
    let truncated_content = if content.len() > 3900 {
        format!("{}...\n_(内容已截断)_", &content[..3900])
    } else {
        content.to_string()
    };

    // Format message with Slack markdown-like formatting
    let message = format!("*{}*\n\n```{}```", title, truncated_content);

    let body = serde_json::json!({
        "text": message,
        "mrkdwn": true
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
                // Slack webhook returns "ok" as plain text on success
                let text = resp.text().await.unwrap_or_default();
                if text == "ok" {
                    tracing::info!("Report sent to Slack: {}", title);
                    Some(true)
                } else {
                    tracing::warn!("Unexpected Slack response: {}", text);
                    Some(false)
                }
            } else {
                let status = resp.status();
                let error_text = resp.text().await.unwrap_or_default();
                tracing::warn!("Failed to send to Slack: {} - {}", status, error_text);
                Some(false)
            }
        }
        Err(e) => {
            tracing::warn!("Failed to call Slack webhook: {}", e);
            Some(false)
        }
    }
}

/// Internal: test Slack webhook connection
pub async fn test_slack_connection_service() -> AppResult<bool> {
    let settings = crate::memory_storage::get_settings_sync()?;

    let webhook_url = match settings.slack_webhook_url {
        Some(ref url) if !url.is_empty() => url,
        _ => return Ok(false), // Not configured
    };

    let client = create_http_client(webhook_url, 30)?;

    // Send a test message
    let body = serde_json::json!({
        "text": "_DailyLogger 连接测试成功_",
        "mrkdwn": true
    });

    let response = client
        .post(webhook_url)
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await?;

    if response.status().is_success() {
        let text = response.text().await.unwrap_or_default();
        if text == "ok" {
            Ok(true)
        } else {
            Err(crate::errors::AppError::network(format!(
                "Unexpected Slack response: {}",
                text
            )))
        }
    } else {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        Err(crate::errors::AppError::network(format!(
            "Slack API error: {} - {}",
            status, error_text
        )))
    }
}

/// Test Slack webhook connection (Tauri command wrapper)
#[command]
pub async fn test_slack_connection() -> Result<bool, String> {
    test_slack_connection_service()
        .await
        .map_err(|e| e.to_string())
}

/// Send a report to Slack synchronously (wrapper for async function)
/// This is used in report generation functions that need to send to Slack
pub fn send_to_slack_sync(
    settings: &crate::memory_storage::Settings,
    title: &str,
    content: &str,
) -> Option<bool> {
    // For synchronous contexts, we use tokio::runtime
    let rt = tokio::runtime::Runtime::new().ok()?;
    rt.block_on(send_to_slack(settings, title, content))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_slack_configured_returns_false_when_no_webhook() {
        let settings = crate::memory_storage::Settings {
            slack_webhook_url: None,
            ..Default::default()
        };
        assert!(!is_slack_configured(&settings));
    }

    #[test]
    fn is_slack_configured_returns_false_when_empty() {
        let settings = crate::memory_storage::Settings {
            slack_webhook_url: Some("".to_string()),
            ..Default::default()
        };
        assert!(!is_slack_configured(&settings));
    }

    #[test]
    fn is_slack_configured_returns_true_when_configured() {
        let settings = crate::memory_storage::Settings {
            slack_webhook_url: Some("https://hooks.slack.com/services/XXX/YYY/ZZZ".to_string()),
            ..Default::default()
        };
        assert!(is_slack_configured(&settings));
    }
}
