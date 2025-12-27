// User-centric sync functions

use crate::db::queries::{self, is_bot_user};
use crate::db::AppState;
use crate::github::sync::generate_embeddings_for_new_items;
use anyhow::Result;
use chrono::{Duration, Utc};
use serde::Serialize;
use tauri::{AppHandle, Manager};

#[derive(Clone, Serialize)]
struct SyncProgress {
    phase: String,
    current: usize,
    total: usize,
    message: String,
}

fn emit_progress(app: &AppHandle, phase: &str, current: usize, total: usize, message: &str) {
    app.emit_all(
        "sync-progress",
        SyncProgress {
            phase: phase.to_string(),
            current,
            total,
            message: message.to_string(),
        },
    )
    .ok();
}

/// Sync activity for a specific tracked user across all enabled repositories
pub async fn sync_tracked_user(app: &AppHandle, state: &AppState, token: &str, username: &str) -> Result<()> {
    tracing::info!("Starting user-centric sync for '{}'", username);

    // Load settings
    let (history_days, excluded_bots) = {
        let conn = state.sqlite.lock().unwrap();
        let settings = queries::get_settings(&conn)?;
        (settings.history_days, settings.excluded_bots)
    };

    // Get enabled repos from database
    let repos = {
        let conn = state.sqlite.lock().unwrap();
        queries::get_enabled_repositories(&conn)?
    };

    if repos.is_empty() {
        tracing::warn!("No repositories configured for sync");
        return Ok(());
    }

    let since_date = Utc::now() - Duration::days(history_days as i64);
    let since = since_date.format("%Y-%m-%d").to_string();

    emit_progress(app, "syncing", 0, 3, &format!("Fetching data for user {}", username));

    // Sync PRs authored by this user across all tracked repos
    let mut total_prs = 0;
    for repo in &repos {
        match sync_user_prs(state, token, &repo.owner, &repo.name, repo.id, username, &since, &excluded_bots).await {
            Ok(count) => {
                total_prs += count;
                tracing::info!("Synced {} PRs for {} in {}/{}", count, username, repo.owner, repo.name);
            }
            Err(e) => {
                tracing::error!("Failed to sync PRs for {} in {}/{}: {}", username, repo.owner, repo.name, e);
            }
        }
    }

    emit_progress(app, "syncing", 1, 3, &format!("Found {} PRs for {}", total_prs, username));

    // Sync issues authored by this user across all tracked repos
    let mut total_issues = 0;
    for repo in &repos {
        match sync_user_issues(state, token, &repo.owner, &repo.name, repo.id, username, &since, &excluded_bots).await {
            Ok(count) => {
                total_issues += count;
                tracing::info!("Synced {} issues for {} in {}/{}", count, username, repo.owner, repo.name);
            }
            Err(e) => {
                tracing::error!("Failed to sync issues for {} in {}/{}: {}", username, repo.owner, repo.name, e);
            }
        }
    }

    emit_progress(app, "syncing", 2, 3, &format!("Found {} issues for {}", total_issues, username));

    // Generate embeddings for new items
    if total_prs > 0 || total_issues > 0 {
        tracing::info!("Generating embeddings for user {}'s new activity...", username);
        if let Err(e) = generate_embeddings_for_new_items(app, state).await {
            tracing::error!("Embedding generation failed: {}", e);
        }
    }

    emit_progress(app, "complete", 3, 3, &format!("Synced {} PRs and {} issues for {}", total_prs, total_issues, username));

    tracing::info!("User-centric sync completed for '{}': {} PRs, {} issues", username, total_prs, total_issues);
    Ok(())
}

/// Sync PRs authored by a specific user in a specific repository using GitHub Search API
async fn sync_user_prs(
    state: &AppState,
    token: &str,
    owner: &str,
    name: &str,
    repo_id: i64,
    username: &str,
    since: &str,
    excluded_bots: &[String],
) -> Result<usize> {
    use reqwest::header::{ACCEPT, AUTHORIZATION, USER_AGENT};

    if is_bot_user(username, excluded_bots) {
        return Ok(0);
    }

    let client = reqwest::Client::new();
    let mut page = 1;
    let mut total_synced = 0;

    loop {
        // Use GitHub Search API to find PRs by this user in this repo
        let url = format!(
            "https://api.github.com/search/issues?q=type:pr+author:{}+repo:{}/{}+created:>={}&per_page=100&page={}",
            username, owner, name, since, page
        );

        let response = client
            .get(&url)
            .header(AUTHORIZATION, format!("Bearer {}", token))
            .header(ACCEPT, "application/vnd.github+json")
            .header(USER_AGENT, "MADE-Activity-Tracker")
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            tracing::error!("GitHub Search API error ({}): {}", status, body);
            break;
        }

        let search_result: serde_json::Value = response.json().await?;
        let items = search_result["items"].as_array()
            .ok_or_else(|| anyhow::anyhow!("Invalid search response"))?;

        if items.is_empty() {
            break;
        }

        for item in items {
            // Fetch full PR details
            let pr_number = item["number"].as_i64()
                .ok_or_else(|| anyhow::anyhow!("Missing PR number"))?;

            // Use REST API to get full PR details including additions/deletions
            let pr_url = format!("https://api.github.com/repos/{}/{}/pulls/{}", owner, name, pr_number);
            let pr_response = client
                .get(&pr_url)
                .header(AUTHORIZATION, format!("Bearer {}", token))
                .header(ACCEPT, "application/vnd.github+json")
                .header(USER_AGENT, "MADE-Activity-Tracker")
                .send()
                .await?;

            if !pr_response.status().is_success() {
                tracing::warn!("Failed to fetch PR details for {}/{} #{}", owner, name, pr_number);
                continue;
            }

            let pr: serde_json::Value = pr_response.json().await?;

            // Get or create author
            let author_id = if let Some(author) = pr["user"].as_object() {
                if let Some(github_id) = author["id"].as_i64() {
                    let login = author["login"].as_str().unwrap_or("");
                    let conn = state.sqlite.lock().unwrap();
                    Some(queries::get_or_create_user(&conn, github_id, login, None, None, None, None, None, None)?)
                } else {
                    None
                }
            } else {
                None
            };

            // Extract labels
            let labels: Vec<String> = pr["labels"].as_array()
                .map(|arr| arr.iter().filter_map(|l| l["name"].as_str().map(String::from)).collect())
                .unwrap_or_default();

            // Upsert PR
            let conn = state.sqlite.lock().unwrap();
            queries::upsert_pull_request(
                &conn,
                pr["id"].as_i64().unwrap_or(0),
                repo_id,
                pr_number as i32,
                pr["title"].as_str().unwrap_or(""),
                pr["body"].as_str(),
                pr["state"].as_str().unwrap_or("open"),
                author_id,
                pr["created_at"].as_str().unwrap_or(""),
                pr["updated_at"].as_str().unwrap_or(""),
                pr["merged_at"].as_str(),
                pr["closed_at"].as_str(),
                pr["additions"].as_i64().unwrap_or(0) as i32,
                pr["deletions"].as_i64().unwrap_or(0) as i32,
                pr["changed_files"].as_i64().unwrap_or(0) as i32,
                &labels,
                pr["updated_at"].as_str().unwrap_or(""),
            )?;

            total_synced += 1;
        }

        // Check if there are more pages
        let total_count = search_result["total_count"].as_i64().unwrap_or(0);
        if (page * 100) >= total_count as i32 {
            break;
        }

        page += 1;
    }

    Ok(total_synced)
}

/// Sync issues authored by a specific user in a specific repository using GitHub Search API
async fn sync_user_issues(
    state: &AppState,
    token: &str,
    owner: &str,
    name: &str,
    repo_id: i64,
    username: &str,
    since: &str,
    excluded_bots: &[String],
) -> Result<usize> {
    use reqwest::header::{ACCEPT, AUTHORIZATION, USER_AGENT};

    if is_bot_user(username, excluded_bots) {
        return Ok(0);
    }

    let client = reqwest::Client::new();
    let mut page = 1;
    let mut total_synced = 0;

    loop {
        // Use GitHub Search API to find issues by this user in this repo
        let url = format!(
            "https://api.github.com/search/issues?q=type:issue+author:{}+repo:{}/{}+created:>={}&per_page=100&page={}",
            username, owner, name, since, page
        );

        let response = client
            .get(&url)
            .header(AUTHORIZATION, format!("Bearer {}", token))
            .header(ACCEPT, "application/vnd.github+json")
            .header(USER_AGENT, "MADE-Activity-Tracker")
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            tracing::error!("GitHub Search API error ({}): {}", status, body);
            break;
        }

        let search_result: serde_json::Value = response.json().await?;
        let items = search_result["items"].as_array()
            .ok_or_else(|| anyhow::anyhow!("Invalid search response"))?;

        if items.is_empty() {
            break;
        }

        for item in items {
            let issue_number = item["number"].as_i64()
                .ok_or_else(|| anyhow::anyhow!("Missing issue number"))?;

            // Get full issue details
            let issue_url = format!("https://api.github.com/repos/{}/{}/issues/{}", owner, name, issue_number);
            let issue_response = client
                .get(&issue_url)
                .header(AUTHORIZATION, format!("Bearer {}", token))
                .header(ACCEPT, "application/vnd.github+json")
                .header(USER_AGENT, "MADE-Activity-Tracker")
                .send()
                .await?;

            if !issue_response.status().is_success() {
                tracing::warn!("Failed to fetch issue details for {}/{} #{}", owner, name, issue_number);
                continue;
            }

            let issue: serde_json::Value = issue_response.json().await?;

            // Skip if it's actually a PR (GitHub API returns PRs in issues search)
            if issue.get("pull_request").is_some() {
                continue;
            }

            // Get or create author
            let author_id = if let Some(author) = issue["user"].as_object() {
                if let Some(github_id) = author["id"].as_i64() {
                    let login = author["login"].as_str().unwrap_or("");
                    let conn = state.sqlite.lock().unwrap();
                    Some(queries::get_or_create_user(&conn, github_id, login, None, None, None, None, None, None)?)
                } else {
                    None
                }
            } else {
                None
            };

            // Get assignee
            let assignee_id = if let Some(assignee) = issue["assignee"].as_object() {
                if let Some(github_id) = assignee["id"].as_i64() {
                    let login = assignee["login"].as_str().unwrap_or("");
                    let conn = state.sqlite.lock().unwrap();
                    Some(queries::get_or_create_user(&conn, github_id, login, None, None, None, None, None, None)?)
                } else {
                    None
                }
            } else {
                None
            };

            // Extract labels
            let labels: Vec<String> = issue["labels"].as_array()
                .map(|arr| arr.iter().filter_map(|l| l["name"].as_str().map(String::from)).collect())
                .unwrap_or_default();

            // Upsert issue
            let conn = state.sqlite.lock().unwrap();
            queries::upsert_issue(
                &conn,
                issue["id"].as_i64().unwrap_or(0),
                repo_id,
                issue_number as i32,
                issue["title"].as_str().unwrap_or(""),
                issue["body"].as_str(),
                issue["state"].as_str().unwrap_or("open"),
                author_id,
                assignee_id,
                None, // milestone_id - would need separate lookup
                issue["created_at"].as_str().unwrap_or(""),
                issue["updated_at"].as_str().unwrap_or(""),
                issue["closed_at"].as_str(),
                &labels,
                issue["updated_at"].as_str().unwrap_or(""),
            )?;

            total_synced += 1;
        }

        // Check if there are more pages
        let total_count = search_result["total_count"].as_i64().unwrap_or(0);
        if (page * 100) >= total_count as i32 {
            break;
        }

        page += 1;
    }

    Ok(total_synced)
}
