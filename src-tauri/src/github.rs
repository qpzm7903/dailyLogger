//! GitHub API integration for work time statistics.
//!
//! This module provides functionality to fetch commit and PR data from GitHub
//! for calculating work time statistics.

use crate::create_http_client;
use chrono::Timelike;
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

/// Work time statistics calculated from GitHub commits
#[derive(Debug, Clone, Default)]
pub struct GitHubWorkStats {
    /// Total number of commits
    pub commit_count: usize,
    /// Total number of pull requests
    pub pr_count: usize,
    /// Estimated work hours (based on commit clustering)
    pub estimated_hours: f64,
    /// List of repositories with activity
    pub active_repos: Vec<String>,
    /// Commit messages grouped by hour
    pub commits_by_hour: std::collections::HashMap<u32, Vec<String>>,
    /// Pull request titles
    pub pull_requests: Vec<String>,
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

    let url = format!("{}/user", GITHUB_API_BASE);
    let client =
        create_http_client(&url, 30).map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    // Try to get the authenticated user to verify the token
    let response = client
        .get(&url)
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
    let url = format!("{}/repos/{}/{}/commits", GITHUB_API_BASE, owner, repo);
    let client = create_http_client(&url, 60)?;

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
    let url = format!("{}/repos/{}/{}/pulls", GITHUB_API_BASE, owner, repo);
    let client = create_http_client(&url, 60)?;

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

/// Calculate work statistics from a list of commits
///
/// Uses commit clustering to estimate work time:
/// - Commits within 2 hours are considered part of the same work session
/// - Each session contributes to estimated work hours
pub fn calculate_work_stats_from_commits(
    commits: &[GitHubCommit],
    pull_requests: &[GitHubPullRequest],
    repo_name: &str,
) -> GitHubWorkStats {
    use std::collections::HashMap;

    let mut stats = GitHubWorkStats::default();

    if commits.is_empty() && pull_requests.is_empty() {
        return stats;
    }

    // Track active repos
    if !commits.is_empty() || !pull_requests.is_empty() {
        stats.active_repos.push(repo_name.to_string());
    }

    // Process commits
    let mut commit_times: Vec<chrono::DateTime<chrono::Utc>> = Vec::new();
    let mut commits_by_hour: HashMap<u32, Vec<String>> = HashMap::new();

    for commit in commits {
        stats.commit_count += 1;

        // Parse commit date
        if let Some(ref author) = commit.commit.author {
            if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&author.date) {
                let utc_dt = dt.with_timezone(&chrono::Utc);
                commit_times.push(utc_dt);

                // Group by hour
                let hour = utc_dt.hour();
                let message = commit
                    .commit
                    .message
                    .lines()
                    .next()
                    .unwrap_or("")
                    .to_string();
                commits_by_hour.entry(hour).or_default().push(message);
            }
        }
    }

    stats.commits_by_hour = commits_by_hour;

    // Sort commit times for clustering
    commit_times.sort();

    // Estimate work hours using commit clustering
    // Commits within 2 hours are considered same session
    const SESSION_GAP_HOURS: i64 = 2;
    let mut total_minutes: i64 = 0;

    if !commit_times.is_empty() {
        let mut session_start = commit_times[0];
        let mut session_end = commit_times[0];

        for time in commit_times.iter().skip(1) {
            let gap = (*time - session_end).num_minutes();
            if gap <= SESSION_GAP_HOURS * 60 {
                // Same session, extend end time
                session_end = *time;
            } else {
                // New session, record previous session duration
                let session_minutes = (session_end - session_start).num_minutes();
                // Minimum 30 minutes per session
                total_minutes += std::cmp::max(session_minutes, 30);
                session_start = *time;
                session_end = *time;
            }
        }

        // Record last session
        let session_minutes = (session_end - session_start).num_minutes();
        total_minutes += std::cmp::max(session_minutes, 30);
    }

    stats.estimated_hours = total_minutes as f64 / 60.0;

    // Process pull requests
    for pr in pull_requests {
        stats.pr_count += 1;
        stats
            .pull_requests
            .push(format!("#{}: {}", pr.number, pr.title));
    }

    stats
}

/// Format GitHub activity for inclusion in reports
///
/// Creates a markdown-formatted section with commit and PR statistics
pub fn format_github_activity_for_report(stats: &GitHubWorkStats) -> String {
    if stats.commit_count == 0 && stats.pr_count == 0 {
        return String::new();
    }

    let mut output = String::new();
    output.push_str("### 🐙 GitHub 活动\n\n");

    // Summary
    output.push_str(&format!("- **提交数**: {} 次\n", stats.commit_count));
    output.push_str(&format!("- **Pull Requests**: {} 个\n", stats.pr_count));
    output.push_str(&format!(
        "- **预估工时**: {:.1} 小时\n",
        stats.estimated_hours
    ));
    output.push_str(&format!(
        "- **活跃仓库**: {}\n",
        stats.active_repos.join(", ")
    ));

    // Commits by hour
    if !stats.commits_by_hour.is_empty() {
        output.push_str("\n#### 提交时间分布\n\n");
        let mut hours: Vec<_> = stats.commits_by_hour.keys().collect();
        hours.sort();

        for hour in hours {
            let messages = stats.commits_by_hour.get(hour).unwrap();
            output.push_str(&format!("- **{}:00** - {} 次提交\n", hour, messages.len()));
            for msg in messages {
                let truncated = if msg.len() > 60 {
                    format!("{}...", &msg[..57])
                } else {
                    msg.clone()
                };
                output.push_str(&format!("  - {}\n", truncated));
            }
        }
    }

    // Pull requests
    if !stats.pull_requests.is_empty() {
        output.push_str("\n#### Pull Requests\n\n");
        for pr in &stats.pull_requests {
            output.push_str(&format!("- {}\n", pr));
        }
    }

    output
}

/// Fetch today's GitHub activity for all configured repositories
pub async fn fetch_today_github_activity(
    settings: &crate::memory_storage::Settings,
) -> Result<GitHubWorkStats, String> {
    let token = match &settings.github_token {
        Some(token) if !token.is_empty() => token,
        _ => return Ok(GitHubWorkStats::default()),
    };

    let repos = parse_repositories(settings);
    if repos.is_empty() {
        return Ok(GitHubWorkStats::default());
    }

    // Calculate today's date range in UTC
    let now = chrono::Utc::now();
    let today_start = now.date_naive().and_hms_opt(0, 0, 0).unwrap();
    let today_end = now.date_naive().and_hms_opt(23, 59, 59).unwrap();
    let today_start_dt: chrono::DateTime<chrono::Utc> =
        chrono::DateTime::from_naive_utc_and_offset(today_start, chrono::Utc);
    let today_end_dt: chrono::DateTime<chrono::Utc> =
        chrono::DateTime::from_naive_utc_and_offset(today_end, chrono::Utc);

    let since = today_start.format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let until = today_end.format("%Y-%m-%dT%H:%M:%SZ").to_string();

    let mut all_commits: Vec<GitHubCommit> = Vec::new();
    let mut all_prs: Vec<GitHubPullRequest> = Vec::new();
    let mut active_repos: Vec<String> = Vec::new();

    for repo_full in &repos {
        let parts: Vec<&str> = repo_full.split('/').collect();
        if parts.len() != 2 {
            continue;
        }
        let owner = parts[0];
        let repo = parts[1];

        // Fetch commits
        match fetch_commits(token, owner, repo, Some(&since), Some(&until)).await {
            Ok(commits) => {
                if !commits.is_empty() {
                    active_repos.push(repo_full.clone());
                    all_commits.extend(commits);
                }
            }
            Err(e) => {
                tracing::warn!("Failed to fetch commits for {}: {}", repo_full, e);
            }
        }

        // Fetch PRs (created today)
        match fetch_pull_requests(token, owner, repo, "all").await {
            Ok(prs) => {
                for pr in prs {
                    // Check if PR was created today
                    if let Ok(created) = chrono::DateTime::parse_from_rfc3339(&pr.created_at) {
                        let created_utc = created.with_timezone(&chrono::Utc);
                        if created_utc >= today_start_dt && created_utc <= today_end_dt {
                            if !active_repos.contains(repo_full) {
                                active_repos.push(repo_full.clone());
                            }
                            all_prs.push(pr);
                        }
                    }
                }
            }
            Err(e) => {
                tracing::warn!("Failed to fetch PRs for {}: {}", repo_full, e);
            }
        }
    }

    // Calculate combined stats
    let mut commit_count = 0usize;
    let mut commits_by_hour: std::collections::HashMap<u32, Vec<String>> =
        std::collections::HashMap::new();
    let mut pull_requests: Vec<String> = Vec::new();

    for commit in &all_commits {
        commit_count += 1;

        if let Some(ref author) = commit.commit.author {
            if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&author.date) {
                let utc_dt = dt.with_timezone(&chrono::Utc);
                let hour = utc_dt.hour();
                let message = commit
                    .commit
                    .message
                    .lines()
                    .next()
                    .unwrap_or("")
                    .to_string();
                commits_by_hour.entry(hour).or_default().push(message);
            }
        }
    }

    let mut pr_count = 0usize;
    for pr in &all_prs {
        pr_count += 1;
        pull_requests.push(format!("#{}: {}", pr.number, pr.title));
    }

    // Calculate estimated hours
    let mut commit_times: Vec<chrono::DateTime<chrono::Utc>> = Vec::new();
    for commit in &all_commits {
        if let Some(ref author) = commit.commit.author {
            if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&author.date) {
                commit_times.push(dt.with_timezone(&chrono::Utc));
            }
        }
    }
    commit_times.sort();

    const SESSION_GAP_HOURS: i64 = 2;
    let mut total_minutes: i64 = 0;

    if !commit_times.is_empty() {
        let mut session_start = commit_times[0];
        let mut session_end = commit_times[0];

        for time in commit_times.iter().skip(1) {
            let gap = (*time - session_end).num_minutes();
            if gap <= SESSION_GAP_HOURS * 60 {
                session_end = *time;
            } else {
                let session_minutes = (session_end - session_start).num_minutes();
                total_minutes += std::cmp::max(session_minutes, 30);
                session_start = *time;
                session_end = *time;
            }
        }

        let session_minutes = (session_end - session_start).num_minutes();
        total_minutes += std::cmp::max(session_minutes, 30);
    }

    let estimated_hours = total_minutes as f64 / 60.0;

    Ok(GitHubWorkStats {
        commit_count,
        pr_count,
        estimated_hours,
        active_repos,
        commits_by_hour,
        pull_requests,
    })
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

    #[test]
    fn calculate_work_stats_empty_inputs() {
        let stats = calculate_work_stats_from_commits(&[], &[], "owner/repo");
        assert_eq!(stats.commit_count, 0);
        assert_eq!(stats.pr_count, 0);
        assert_eq!(stats.estimated_hours, 0.0);
        assert!(stats.active_repos.is_empty());
    }

    #[test]
    fn calculate_work_stats_single_commit() {
        let commit = GitHubCommit {
            sha: "abc123".to_string(),
            html_url: "https://github.com/owner/repo/commit/abc123".to_string(),
            commit: GitHubCommitInfo {
                message: "feat: add new feature".to_string(),
                author: Some(GitHubCommitAuthor {
                    name: "Test User".to_string(),
                    email: "test@example.com".to_string(),
                    date: "2024-01-15T10:30:00Z".to_string(),
                }),
            },
        };

        let stats = calculate_work_stats_from_commits(&[commit], &[], "owner/repo");
        assert_eq!(stats.commit_count, 1);
        assert_eq!(stats.pr_count, 0);
        assert!(stats.estimated_hours >= 0.5); // Minimum 30 minutes
        assert_eq!(stats.active_repos, vec!["owner/repo"]);
    }

    #[test]
    fn calculate_work_stats_multiple_commits_same_session() {
        let commits = vec![
            GitHubCommit {
                sha: "abc123".to_string(),
                html_url: "https://github.com/owner/repo/commit/abc123".to_string(),
                commit: GitHubCommitInfo {
                    message: "feat: add feature".to_string(),
                    author: Some(GitHubCommitAuthor {
                        name: "Test User".to_string(),
                        email: "test@example.com".to_string(),
                        date: "2024-01-15T10:00:00Z".to_string(),
                    }),
                },
            },
            GitHubCommit {
                sha: "def456".to_string(),
                html_url: "https://github.com/owner/repo/commit/def456".to_string(),
                commit: GitHubCommitInfo {
                    message: "fix: fix bug".to_string(),
                    author: Some(GitHubCommitAuthor {
                        name: "Test User".to_string(),
                        email: "test@example.com".to_string(),
                        date: "2024-01-15T11:30:00Z".to_string(),
                    }),
                },
            },
        ];

        let stats = calculate_work_stats_from_commits(&commits, &[], "owner/repo");
        assert_eq!(stats.commit_count, 2);
        // Both commits within 2 hours, so they form one session
        assert!(stats.estimated_hours >= 1.5); // 1.5 hours between commits
    }

    #[test]
    fn calculate_work_stats_with_pull_requests() {
        let pr = GitHubPullRequest {
            id: 1,
            number: 42,
            title: "Add new feature".to_string(),
            html_url: "https://github.com/owner/repo/pull/42".to_string(),
            state: "open".to_string(),
            created_at: "2024-01-15T10:00:00Z".to_string(),
            merged_at: None,
            closed_at: None,
        };

        let stats = calculate_work_stats_from_commits(&[], &[pr], "owner/repo");
        assert_eq!(stats.commit_count, 0);
        assert_eq!(stats.pr_count, 1);
        assert_eq!(stats.pull_requests, vec!["#42: Add new feature"]);
    }

    #[test]
    fn format_github_activity_empty_stats() {
        let stats = GitHubWorkStats::default();
        let output = format_github_activity_for_report(&stats);
        assert!(output.is_empty());
    }

    #[test]
    fn format_github_activity_with_commits() {
        let stats = GitHubWorkStats {
            commit_count: 5,
            pr_count: 2,
            estimated_hours: 3.5,
            active_repos: vec!["owner/repo1".to_string(), "owner/repo2".to_string()],
            pull_requests: vec!["#42: Add feature".to_string()],
            commits_by_hour: std::collections::HashMap::new(),
        };

        let output = format_github_activity_for_report(&stats);
        assert!(output.contains("提交数"));
        assert!(output.contains("5 次"));
        assert!(output.contains("Pull Requests"));
        assert!(output.contains("2 个"));
        assert!(output.contains("预估工时"));
        assert!(output.contains("3.5 小时"));
        assert!(output.contains("owner/repo1"));
    }

    #[test]
    fn format_github_activity_with_hour_distribution() {
        let mut commits_by_hour = std::collections::HashMap::new();
        commits_by_hour.insert(10, vec!["commit 1".to_string(), "commit 2".to_string()]);
        commits_by_hour.insert(14, vec!["commit 3".to_string()]);

        let stats = GitHubWorkStats {
            commit_count: 3,
            pr_count: 0,
            estimated_hours: 1.0,
            active_repos: vec!["owner/repo".to_string()],
            commits_by_hour,
            pull_requests: Vec::new(),
        };

        let output = format_github_activity_for_report(&stats);
        assert!(output.contains("10:00"));
        assert!(output.contains("14:00"));
        assert!(output.contains("2 次提交"));
    }

    #[test]
    fn calculate_work_stats_multiple_sessions() {
        // Test commits that span multiple sessions (gap > 2 hours)
        let commits = vec![
            GitHubCommit {
                sha: "abc123".to_string(),
                html_url: "https://github.com/owner/repo/commit/abc123".to_string(),
                commit: GitHubCommitInfo {
                    message: "feat: morning work".to_string(),
                    author: Some(GitHubCommitAuthor {
                        name: "Test User".to_string(),
                        email: "test@example.com".to_string(),
                        date: "2024-01-15T09:00:00Z".to_string(),
                    }),
                },
            },
            GitHubCommit {
                sha: "def456".to_string(),
                html_url: "https://github.com/owner/repo/commit/def456".to_string(),
                commit: GitHubCommitInfo {
                    message: "feat: afternoon work".to_string(),
                    author: Some(GitHubCommitAuthor {
                        name: "Test User".to_string(),
                        email: "test@example.com".to_string(),
                        date: "2024-01-15T14:00:00Z".to_string(), // 5 hours gap
                    }),
                },
            },
        ];

        let stats = calculate_work_stats_from_commits(&commits, &[], "owner/repo");
        assert_eq!(stats.commit_count, 2);
        // Two separate sessions: 09:00 (min 30 min) + 14:00 (min 30 min) = 1 hour min
        assert!(stats.estimated_hours >= 1.0);
    }

    #[test]
    fn format_github_activity_truncates_long_messages() {
        let mut commits_by_hour = std::collections::HashMap::new();
        let long_message = "This is a very long commit message that should be truncated because it exceeds the 60 character limit for display in reports";
        commits_by_hour.insert(10, vec![long_message.to_string()]);

        let stats = GitHubWorkStats {
            commit_count: 1,
            pr_count: 0,
            estimated_hours: 0.5,
            active_repos: vec!["owner/repo".to_string()],
            commits_by_hour,
            pull_requests: Vec::new(),
        };

        let output = format_github_activity_for_report(&stats);
        // Should contain truncated version with "..."
        assert!(output.contains("..."));
        // Original long message should NOT appear in full
        assert!(!output.contains(&long_message[..70]));
    }

    #[test]
    fn parse_repositories_handles_invalid_format() {
        // Test that invalid repository formats don't cause panic
        let settings = crate::memory_storage::Settings {
            github_repositories: Some(r#"["valid/repo", "invalid-no-slash", "also/valid/repo2", ""]"#.to_string()),
            ..Default::default()
        };
        let repos = parse_repositories(&settings);
        // All entries are parsed as-is; filtering happens in fetch_today_github_activity
        assert_eq!(repos.len(), 4);
    }
}
