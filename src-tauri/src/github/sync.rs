use crate::db::queries::{self, is_bot_user};
use crate::db::AppState;
use crate::github::graphql::{self, *};
use anyhow::Result;
use chrono::{Duration, Utc};
use tauri::{AppHandle, Manager};

/// Sync all data for all enabled repositories
pub async fn sync_all_repos(app: &AppHandle, state: &AppState, token: &str) -> Result<()> {
    // Load config to get history_days and excluded_bots
    let config = crate::config::commands::load_config_internal(app)?;
    
    // Ensure repositories from config are in the database
    {
        let conn = state.sqlite.lock().unwrap();
        for repo_config in &config.repositories {
            if repo_config.enabled {
                queries::upsert_repository(&conn, &repo_config.owner, &repo_config.name, None, true)?;
            }
        }
    }
    
    // Get enabled repos from database
    let repos = {
        let conn = state.sqlite.lock().unwrap();
        queries::get_enabled_repositories(&conn)?
    };

    if repos.is_empty() {
        tracing::warn!("No repositories configured for sync");
        return Ok(());
    }

    let total_repos = repos.len();
    let since_date = Utc::now() - Duration::days(config.history_days as i64);
    let since = since_date.format("%Y-%m-%dT%H:%M:%SZ").to_string();

    tracing::info!("Starting sync for {} repos, since {}", total_repos, since);

    for (idx, repo) in repos.iter().enumerate() {
        emit_progress(app, "syncing", idx + 1, total_repos, 
            &format!("Syncing {}/{}", repo.owner, repo.name));

        // Sync milestones first (needed for issue references)
        if let Err(e) = sync_milestones(state, token, repo.id, &repo.owner, &repo.name).await {
            tracing::error!("Failed to sync milestones for {}/{}: {}", repo.owner, repo.name, e);
        }

        // Sync issues
        if let Err(e) = sync_issues(state, token, repo.id, &repo.owner, &repo.name, &since, &config.excluded_bots).await {
            tracing::error!("Failed to sync issues for {}/{}: {}", repo.owner, repo.name, e);
        }

        // Sync PRs
        if let Err(e) = sync_pull_requests(state, token, repo.id, &repo.owner, &repo.name, &config.excluded_bots).await {
            tracing::error!("Failed to sync PRs for {}/{}: {}", repo.owner, repo.name, e);
        }

        // Update last synced timestamp
        {
            let conn = state.sqlite.lock().unwrap();
            queries::update_repo_synced_at(&conn, repo.id)?;
        }
    }

    // Phase 3 will add: Generate embeddings for new items
    // emit_progress(app, "embeddings", 0, 0, "Generating embeddings for new items...");

    emit_progress(app, "complete", total_repos, total_repos, "Sync complete!");
    
    tracing::info!("Sync completed successfully");
    Ok(())
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

#[derive(Clone, serde::Serialize)]
pub struct SyncProgress {
    pub phase: String,
    pub current: usize,
    pub total: usize,
    pub message: String,
}

async fn sync_issues(
    state: &AppState,
    token: &str,
    repo_id: i64,
    owner: &str,
    name: &str,
    since: &str,
    excluded_bots: &[String],
) -> Result<()> {
    tracing::info!("Syncing issues for {}/{}", owner, name);
    
    // Record sync start
    let log_id = {
        let conn = state.sqlite.lock().unwrap();
        queries::record_sync_start(&conn, repo_id, "issues")?
    };
    
    let mut cursor: Option<String> = None;
    let mut total_synced = 0;
    
    loop {
        let variables = serde_json::json!({
            "owner": owner,
            "name": name,
            "cursor": cursor,
            "since": since
        });
        
        let response: IssuesResponse = graphql::execute_query(token, ISSUES_QUERY, variables).await?;
        let issues = response.repository.issues;
        
        for issue_node in &issues.nodes {
            // Skip bot authors
            if let Some(author) = &issue_node.author {
                if is_bot_user(&author.login, excluded_bots) {
                    continue;
                }
            }
            
            // Get or create author
            let author_id = if let Some(author) = &issue_node.author {
                let conn = state.sqlite.lock().unwrap();
                // We don't have full user info from issues query, so use placeholder
                Some(queries::get_or_create_user(&conn, 0, &author.login, None, None, false)?)
            } else {
                None
            };
            
            // Get assignee
            let assignee_id = if let Some(assignee) = issue_node.assignees.nodes.first() {
                let conn = state.sqlite.lock().unwrap();
                Some(queries::get_or_create_user(&conn, 0, &assignee.login, None, None, false)?)
            } else {
                None
            };
            
            // Get milestone ID
            let milestone_id = if let Some(milestone) = &issue_node.milestone {
                let conn = state.sqlite.lock().unwrap();
                queries::get_milestone_id_by_github_id(&conn, milestone.number as i64)?
            } else {
                None
            };
            
            // Extract labels
            let labels: Vec<String> = issue_node.labels.nodes.iter()
                .map(|l| l.name.clone())
                .collect();
            
            // Upsert issue
            {
                let conn = state.sqlite.lock().unwrap();
                queries::upsert_issue(
                    &conn,
                    issue_node.database_id,
                    repo_id,
                    issue_node.number,
                    &issue_node.title,
                    issue_node.body.as_deref(),
                    &issue_node.state,
                    author_id,
                    assignee_id,
                    milestone_id,
                    &issue_node.created_at,
                    &issue_node.updated_at,
                    issue_node.closed_at.as_deref(),
                    &labels,
                )?;
            }
            
            total_synced += 1;
        }
        
        if issues.page_info.has_next_page {
            cursor = issues.page_info.end_cursor;
        } else {
            break;
        }
    }
    
    // Record sync complete
    {
        let conn = state.sqlite.lock().unwrap();
        queries::record_sync_complete(&conn, log_id, total_synced)?;
    }
    
    tracing::info!("Synced {} issues for {}/{}", total_synced, owner, name);
    Ok(())
}

async fn sync_pull_requests(
    state: &AppState,
    token: &str,
    repo_id: i64,
    owner: &str,
    name: &str,
    excluded_bots: &[String],
) -> Result<()> {
    tracing::info!("Syncing PRs for {}/{}", owner, name);
    
    let log_id = {
        let conn = state.sqlite.lock().unwrap();
        queries::record_sync_start(&conn, repo_id, "pull_requests")?
    };
    
    let mut cursor: Option<String> = None;
    let mut total_synced = 0;
    
    loop {
        let variables = serde_json::json!({
            "owner": owner,
            "name": name,
            "cursor": cursor
        });
        
        let response: PullRequestsResponse = graphql::execute_query(token, PULL_REQUESTS_QUERY, variables).await?;
        let prs = response.repository.pull_requests;
        
        for pr_node in &prs.nodes {
            // Skip bot authors
            if let Some(author) = &pr_node.author {
                if is_bot_user(&author.login, excluded_bots) {
                    continue;
                }
            }
            
            // Get or create author
            let author_id = if let Some(author) = &pr_node.author {
                let conn = state.sqlite.lock().unwrap();
                Some(queries::get_or_create_user(&conn, 0, &author.login, None, None, false)?)
            } else {
                None
            };
            
            // Extract labels
            let labels: Vec<String> = pr_node.labels.nodes.iter()
                .map(|l| l.name.clone())
                .collect();
            
            // Upsert PR
            let pr_id = {
                let conn = state.sqlite.lock().unwrap();
                queries::upsert_pull_request(
                    &conn,
                    pr_node.database_id,
                    repo_id,
                    pr_node.number,
                    &pr_node.title,
                    pr_node.body.as_deref(),
                    &pr_node.state,
                    author_id,
                    &pr_node.created_at,
                    &pr_node.updated_at,
                    pr_node.merged_at.as_deref(),
                    pr_node.closed_at.as_deref(),
                    pr_node.additions,
                    pr_node.deletions,
                    pr_node.changed_files,
                    &labels,
                )?
            };
            
            // Sync reviews for this PR
            for review in &pr_node.reviews.nodes {
                let reviewer_id = if let Some(author) = &review.author {
                    let conn = state.sqlite.lock().unwrap();
                    Some(queries::get_or_create_user(&conn, 0, &author.login, None, None, false)?)
                } else {
                    None
                };
                
                if let Some(submitted_at) = &review.submitted_at {
                    let conn = state.sqlite.lock().unwrap();
                    queries::upsert_pr_review(
                        &conn,
                        review.database_id,
                        pr_id,
                        reviewer_id,
                        &review.state,
                        submitted_at,
                    )?;
                }
            }
            
            total_synced += 1;
        }
        
        if prs.page_info.has_next_page {
            cursor = prs.page_info.end_cursor;
        } else {
            break;
        }
    }
    
    {
        let conn = state.sqlite.lock().unwrap();
        queries::record_sync_complete(&conn, log_id, total_synced)?;
    }
    
    tracing::info!("Synced {} PRs for {}/{}", total_synced, owner, name);
    Ok(())
}

async fn sync_milestones(
    state: &AppState,
    token: &str,
    repo_id: i64,
    owner: &str,
    name: &str,
) -> Result<()> {
    tracing::info!("Syncing milestones for {}/{}", owner, name);
    
    let log_id = {
        let conn = state.sqlite.lock().unwrap();
        queries::record_sync_start(&conn, repo_id, "milestones")?
    };
    
    let variables = serde_json::json!({
        "owner": owner,
        "name": name
    });
    
    let response: MilestonesResponse = graphql::execute_query(token, MILESTONES_QUERY, variables).await?;
    let milestones = response.repository.milestones.nodes;
    let total_synced = milestones.len() as i32;
    
    for milestone in &milestones {
        let conn = state.sqlite.lock().unwrap();
        queries::upsert_milestone(
            &conn,
            milestone.number as i64,
            repo_id,
            &milestone.title,
            milestone.description.as_deref(),
            &milestone.state,
            milestone.due_on.as_deref(),
            milestone.issues.total_count,
            milestone.closed_issues.total_count,
        )?;
    }
    
    {
        let conn = state.sqlite.lock().unwrap();
        queries::record_sync_complete(&conn, log_id, total_synced)?;
    }
    
    tracing::info!("Synced {} milestones for {}/{}", total_synced, owner, name);
    Ok(())
}
