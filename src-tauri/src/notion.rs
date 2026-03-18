//! Notion API integration for exporting reports.
//!
//! This module provides functionality to write reports to Notion databases
//! using the Notion API.

use reqwest::Client;
use serde::Deserialize;
use tauri::command;

/// Notion API base URL
const NOTION_API_BASE: &str = "https://api.notion.com/v1";

/// Notion API version
const NOTION_API_VERSION: &str = "2022-06-28";

/// Response from Notion API when creating a page
#[derive(Debug, Deserialize)]
struct NotionPageResponse {
    id: String,
    url: Option<String>,
}

/// Check if Notion is configured in settings
pub fn is_notion_configured(settings: &crate::memory_storage::Settings) -> bool {
    settings.notion_api_key.is_some()
        && !settings.notion_api_key.as_ref().unwrap().is_empty()
        && settings.notion_database_id.is_some()
        && !settings.notion_database_id.as_ref().unwrap().is_empty()
}

/// Write a report to Notion as a new page in the configured database.
///
/// Returns the URL of the created page on success, or None if Notion is not configured
/// or the write fails.
pub async fn write_report_to_notion(
    settings: &crate::memory_storage::Settings,
    title: &str,
    _content: &str,
) -> Option<String> {
    let api_key = settings.notion_api_key.as_ref()?;
    let database_id = settings.notion_database_id.as_ref()?;

    if api_key.is_empty() || database_id.is_empty() {
        return None;
    }

    let client = Client::new();

    // Notion API requires the title property name to match the database's title property
    // Most databases use "Name" or "Title" as the title property
    // We need to send the request with the correct property name
    let body = serde_json::json!({
        "parent": {
            "database_id": database_id
        },
        "properties": {
            "Name": {
                "title": [
                    {
                        "text": {
                            "content": title
                        }
                    }
                ]
            }
        }
    });

    let response = client
        .post(format!("{}/pages", NOTION_API_BASE))
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Notion-Version", NOTION_API_VERSION)
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await;

    match response {
        Ok(resp) => {
            if resp.status().is_success() {
                match resp.json::<NotionPageResponse>().await {
                    Ok(page) => {
                        let url = page.url.unwrap_or_else(|| {
                            format!("https://notion.so/{}", page.id.replace("-", ""))
                        });
                        tracing::info!("Report written to Notion: {}", url);
                        Some(url)
                    }
                    Err(e) => {
                        tracing::warn!("Failed to parse Notion response: {}", e);
                        None
                    }
                }
            } else {
                let status = resp.status();
                let error_text = resp.text().await.unwrap_or_default();
                tracing::warn!("Failed to create Notion page: {} - {}", status, error_text);
                None
            }
        }
        Err(e) => {
            tracing::warn!("Failed to call Notion API: {}", e);
            None
        }
    }
}

/// Test Notion API connection
/// Returns Ok(true) if connection is successful, Ok(false) if Notion is not configured,
/// or Err with error message if connection fails.
#[command]
pub async fn test_notion_connection() -> Result<bool, String> {
    let settings = crate::memory_storage::get_settings_sync()?;

    let api_key = match settings.notion_api_key {
        Some(ref key) if !key.is_empty() => key,
        _ => return Ok(false), // Not configured
    };

    let database_id = match settings.notion_database_id {
        Some(ref id) if !id.is_empty() => id,
        _ => return Ok(false), // Not configured
    };

    let client = Client::new();

    // Try to retrieve the database to verify access
    let response = client
        .get(format!("{}/databases/{}", NOTION_API_BASE, database_id))
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Notion-Version", NOTION_API_VERSION)
        .send()
        .await
        .map_err(|e| format!("Connection error: {}", e))?;

    if response.status().is_success() {
        Ok(true)
    } else {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        Err(format!("Notion API error: {} - {}", status, error_text))
    }
}

/// Write a report to Notion synchronously (wrapper for async function)
/// This is used in report generation functions that need to write to Notion
pub fn write_report_to_notion_sync(
    settings: &crate::memory_storage::Settings,
    title: &str,
    content: &str,
) -> Option<String> {
    // For synchronous contexts, we use tokio::runtime
    // Since this is called from async functions in synthesis, we should use the async version
    // This sync version is provided for potential future use
    let rt = tokio::runtime::Runtime::new().ok()?;
    rt.block_on(write_report_to_notion(settings, title, content))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_notion_configured_returns_false_when_no_api_key() {
        let settings = crate::memory_storage::Settings {
            notion_api_key: None,
            notion_database_id: Some("test-db-id".to_string()),
            ..Default::default()
        };
        assert!(!is_notion_configured(&settings));
    }

    #[test]
    fn is_notion_configured_returns_false_when_no_database_id() {
        let settings = crate::memory_storage::Settings {
            notion_api_key: Some("test-api-key".to_string()),
            notion_database_id: None,
            ..Default::default()
        };
        assert!(!is_notion_configured(&settings));
    }

    #[test]
    fn is_notion_configured_returns_false_when_empty() {
        let settings = crate::memory_storage::Settings {
            notion_api_key: Some("".to_string()),
            notion_database_id: Some("".to_string()),
            ..Default::default()
        };
        assert!(!is_notion_configured(&settings));
    }

    #[test]
    fn is_notion_configured_returns_true_when_configured() {
        let settings = crate::memory_storage::Settings {
            notion_api_key: Some("secret-key".to_string()),
            notion_database_id: Some("db-id".to_string()),
            ..Default::default()
        };
        assert!(is_notion_configured(&settings));
    }
}
