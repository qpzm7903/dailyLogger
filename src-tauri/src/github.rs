//! GitHub API integration for work time statistics.
//!
//! This module provides functionality to fetch commit and PR data from GitHub
//! for calculating work time statistics.

use reqwest::Client;
use serde::Deserialize;
use tauri::command;

/// GitHub API base URL
const GITHUB_API_BASE: &str = "https://api.github.com";

/// Response from GitHub API when listing repositories
#[derive(Debug, Deserialize)]
pub struct GitHubRepository {
    pub id: u64,
    pub name: String,
    pub full_name: String,
    pub html_url: String,
}

/// Response from GitHub API when listing commits
#[derive(Debug, Deserialize)]
pub struct GitHubCommit {
    pub sha: String,
    pub html_url: String,
    pub commit: GitHubCommitInfo,
}

#[derive(Debug, Deserialize)]
pub struct GitHubCommitInfo {
    pub message: String,
    pub author: Option<GitHubCommitAuthor>,
}

#[derive(Debug, Deserialize)]
pub struct GitHubCommitAuthor {
    pub name: String,
    pub email: String,
    pub date: String,
}

/// Response from GitHub API when listing pull requests
#[derive(Debug, Deserialize)]
pub struct GitHubPullRequest {
    pub id: u64,
    pub number: u64,
    pub title: String,
    pub html_url: String,
    pub state: String,
    pub created_at: String,
    pub merged_at: Option<String>,
    pub closed_at: Option<String>,
}

/// Check if GitHub is configured in settings
pub fn is_github_configured(settings: &crate::memory_storage::Settings) -> bool {
    settings.github_token.is_some() && !settings.github_token.as_ref().unwrap().is_empty()
}

/// Parse repositories from settings JSON
pub fn parse_repositories(settings: &crate::memory_storage::Settings) -> Vec<String> {
    match &settings.github_repositories {
        Some(json) => serde_json::from_str(json).unwrap_or_default(),
        None => vec![],
    }
}

/// Test GitHub API connection
/// Returns Ok(true) if connection is successful, Ok(false) if GitHub is not configured,
/// or Err with error message if connection fails.
#[command]
pub async fn test_github_connection() -> Result<bool, String> {
    let settings = crate::memory_storage::get_settings_sync()?;

    let token = match settings.github_token {
        Some(ref token) if !token.is_empty() => token,
        _ => return Ok(false), // Not configured
    };

    let client = Client::new();

    // Try to get the authenticated user to verify the token
    let response = client
        .get(format!("{}/user", GITHUB_API_BASE))
        .header("Authorization", format!("Bearer {}", token))
        .header("User-Agent", "DailyLogger/1.0")
        .header("Accept", "application/vnd.github.v3+json")
        .send()
        .await
        .map_err(|e| format!("Connection error: {}", e))?;

    if response.status().is_success() {
        Ok(true)
    } else {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        Err(format!("GitHub API error: {} - {}", status, error_text))
    }
}

/// Fetch commits for a repository within a date range
pub async fn fetch_commits(
    token: &str,
    owner: &str,
    repo: &str,
    since: Option<&str>,
    until: Option<&str>,
) -> Result<Vec<GitHubCommit>, String> {
    let client = Client::new();
    let url = format!("{}/repos/{}/{}/commits", GITHUB_API_BASE, owner, repo);

    let mut query_params = vec![];
    if let Some(since) = since {
        query_params.push(("since", since.to_string()));
    }
    if let Some(until) = until {
        query_params.push(("until", until.to_string()));
    }

    let response = client
        .get(&url)
        .query(&query_params)
        .header("Authorization", format!("Bearer {}", token))
        .header("User-Agent", "DailyLogger/1.0")
        .header("Accept", "application/vnd.github.v3+json")
        .send()
        .await
        .map_err(|e| format!("Failed to fetch commits: {}", e))?;

    if response.status().is_success() {
        response
            .json::<Vec<GitHubCommit>>()
            .await
            .map_err(|e| format!("Failed to parse commits: {}", e))
    } else {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        Err(format!("GitHub API error: {} - {}", status, error_text))
    }
}

/// Fetch pull requests for a repository
pub async fn fetch_pull_requests(
    token: &str,
    owner: &str,
    repo: &str,
    state: &str,
) -> Result<Vec<GitHubPullRequest>, String> {
    let client = Client::new();
    let url = format!("{}/repos/{}/{}/pulls", GITHUB_API_BASE, owner, repo);

    let response = client
        .get(&url)
        .query(&[("state", state)])
        .header("Authorization", format!("Bearer {}", token))
        .header("User-Agent", "DailyLogger/1.0")
        .header("Accept", "application/vnd.github.v3+json")
        .send()
        .await
        .map_err(|e| format!("Failed to fetch PRs: {}", e))?;

    if response.status().is_success() {
        response
            .json::<Vec<GitHubPullRequest>>()
            .await
            .map_err(|e| format!("Failed to parse PRs: {}", e))
    } else {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        Err(format!("GitHub API error: {} - {}", status, error_text))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_github_configured_returns_false_when_no_token() {
        let settings = crate::memory_storage::Settings {
            github_token: None,
            ..Default::default()
        };
        assert!(!is_github_configured(&settings));
    }

    #[test]
    fn is_github_configured_returns_false_when_empty() {
        let settings = crate::memory_storage::Settings {
            github_token: Some("".to_string()),
            ..Default::default()
        };
        assert!(!is_github_configured(&settings));
    }

    #[test]
    fn is_github_configured_returns_true_when_configured() {
        let settings = crate::memory_storage::Settings {
            github_token: Some("ghp_test_token".to_string()),
            ..Default::default()
        };
        assert!(is_github_configured(&settings));
    }

    #[test]
    fn parse_repositories_returns_empty_when_none() {
        let settings = crate::memory_storage::Settings {
            github_repositories: None,
            ..Default::default()
        };
        assert!(parse_repositories(&settings).is_empty());
    }

    #[test]
    fn parse_repositories_parses_valid_json() {
        let settings = crate::memory_storage::Settings {
            github_repositories: Some(r#"["owner/repo1", "owner/repo2"]"#.to_string()),
            ..Default::default()
        };
        let repos = parse_repositories(&settings);
        assert_eq!(repos, vec!["owner/repo1", "owner/repo2"]);
    }

    #[test]
    fn parse_repositories_returns_empty_for_invalid_json() {
        let settings = crate::memory_storage::Settings {
            github_repositories: Some("not valid json".to_string()),
            ..Default::default()
        };
        assert!(parse_repositories(&settings).is_empty());
    }
}
