use crate::db::queries::{self, is_bot_user};
use crate::db::AppState;
use crate::github::cli::GitHubCli;
use crate::github::graphql::{self, GraphQLExecuteError, *};
use crate::github::rest_api;
use crate::embeddings::{generate_embeddings, generator};
use anyhow::{Context, Result};
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

    // Phase 2B: Generate embeddings for new items
    tracing::info!("Starting embedding generation phase...");
    match generate_embeddings_for_new_items(app, state).await {
        Ok(()) => tracing::info!("Embedding generation completed successfully"),
        Err(e) => {
            tracing::error!("Embedding generation failed: {}", e);
            // Don't fail the whole sync, just log the error
        }
    }

    emit_progress(app, "complete", total_repos, total_repos, "Sync complete!");

    tracing::info!("Sync completed successfully");
    Ok(())
}

/// Sync a single repository by ID
pub async fn sync_single_repo(app: &AppHandle, state: &AppState, token: &str, repo_id: i64) -> Result<()> {
    // Load config to get history_days and excluded_bots
    let config = crate::config::commands::load_config_internal(app)?;
    
    // Get the repository from database
    let repo = {
        let conn = state.sqlite.lock().unwrap();
        queries::get_repository_by_id(&conn, repo_id)?
    };

    let since_date = Utc::now() - Duration::days(config.history_days as i64);
    let since = since_date.format("%Y-%m-%dT%H:%M:%SZ").to_string();

    tracing::info!("Starting sync for {}/{}", repo.owner, repo.name);
    emit_progress(app, "syncing", 1, 1, &format!("Syncing {}/{}", repo.owner, repo.name));

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

    // Generate embeddings for new items from this repo
    tracing::info!("Starting embedding generation for new items...");
    match generate_embeddings_for_new_items(app, state).await {
        Ok(()) => tracing::info!("Embedding generation completed successfully"),
        Err(e) => {
            tracing::error!("Embedding generation failed: {}", e);
            // Don't fail the whole sync, just log the error
        }
    }

    emit_progress(app, "complete", 1, 1, "Sync complete!");
    tracing::info!("Sync completed successfully for {}/{}", repo.owner, repo.name);
    Ok(())
}

/// Generate embeddings for issues and PRs that don't have them yet
async fn generate_embeddings_for_new_items(app: &AppHandle, state: &AppState) -> Result<()> {
    const BATCH_SIZE: i64 = 50;

    tracing::debug!("Entered generate_embeddings_for_new_items function");
    emit_progress(app, "embeddings", 0, 0, "Checking for items without embeddings...");

    // Get issues without embeddings
    tracing::debug!("Querying for issues without embeddings...");
    let issues_to_process = {
        let conn = state.sqlite.lock().unwrap();
        queries::get_issues_without_embeddings(&conn, BATCH_SIZE)?
    };
    tracing::debug!("Found {} issues without embeddings", issues_to_process.len());

    // Get PRs without embeddings
    let prs_to_process = {
        let conn = state.sqlite.lock().unwrap();
        queries::get_prs_without_embeddings(&conn, BATCH_SIZE)?
    };

    let total_items = issues_to_process.len() + prs_to_process.len();

    if total_items == 0 {
        tracing::info!("No items need embeddings");
        return Ok(());
    }

    tracing::info!("Generating embeddings for {} items", total_items);
    emit_progress(app, "embeddings", 0, total_items, &format!("Generating embeddings for {} items...", total_items));

    let mut processed = 0;

    // Process issues
    for issue in issues_to_process {
        let text = generator::prepare_issue_text(&issue.title, &issue.body);

        match generate_embeddings(&[text]) {
            Ok(embeddings) => {
                if let Some(embedding) = embeddings.first() {
                    let conn = state.sqlite.lock().unwrap();
                    queries::set_issue_embedding(&conn, issue.id, embedding)
                        .context("Failed to store issue embedding")?;
                    processed += 1;

                    if processed % 10 == 0 {
                        emit_progress(app, "embeddings", processed, total_items, &format!("Generated {}/{} embeddings...", processed, total_items));
                    }
                } else {
                    tracing::warn!("No embedding generated for issue {}", issue.id);
                }
            }
            Err(e) => {
                tracing::error!("Failed to generate embedding for issue {}: {}", issue.id, e);
                // Continue processing other items
            }
        }
    }

    // Process PRs
    for pr in prs_to_process {
        let text = generator::prepare_pr_text(&pr.title, &pr.body);

        match generate_embeddings(&[text]) {
            Ok(embeddings) => {
                if let Some(embedding) = embeddings.first() {
                    let conn = state.sqlite.lock().unwrap();
                    queries::set_pr_embedding(&conn, pr.id, embedding)
                        .context("Failed to store PR embedding")?;
                    processed += 1;

                    if processed % 10 == 0 {
                        emit_progress(app, "embeddings", processed, total_items, &format!("Generated {}/{} embeddings...", processed, total_items));
                    }
                } else {
                    tracing::warn!("No embedding generated for PR {}", pr.id);
                }
            }
            Err(e) => {
                tracing::error!("Failed to generate embedding for PR {}: {}", pr.id, e);
                // Continue processing other items
            }
        }
    }

    tracing::info!("Successfully generated {} embeddings", processed);
    emit_progress(app, "embeddings", processed, total_items, &format!("Generated {} embeddings", processed));

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

    // Compute watermark-based 'since' for incremental sync
    let watermark_since = {
        let conn = state.sqlite.lock().unwrap();
        queries::get_issues_watermark(&conn, repo_id)?
    };

    // Use watermark if available, otherwise fall back to provided since
    let effective_since = watermark_since.as_deref().unwrap_or(since);
    tracing::info!("Using since={} for issues (watermark: {:?})", effective_since, watermark_since);

    let mut cursor: Option<String> = None;
    let mut total_synced = 0;

    loop {
        let variables = serde_json::json!({
            "owner": owner,
            "name": name,
            "cursor": cursor,
            "since": effective_since
        });

        let response: IssuesResponse = match graphql::execute_query(token, ISSUES_QUERY, variables).await {
            Ok(resp) => resp,
            Err(GraphQLExecuteError::SamlRequired { owner: err_owner, repo: err_repo, org }) => {
                tracing::warn!(
                    "⚠️  SAML SSO required for {}/{}, trying REST API fallback...",
                    err_owner, err_repo
                );

                // Try REST API fallback
                return sync_issues_rest_fallback(state, token, repo_id, owner, name, since, excluded_bots).await;
            }
            Err(e) => {
                return Err(anyhow::anyhow!("GraphQL error: {}", e));
            }
        };
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
                if let Some(github_id) = author.database_id {
                    let conn = state.sqlite.lock().unwrap();
                    Some(queries::get_or_create_user(&conn, github_id, &author.login, None, None, None, None, None)?)
                } else {
                    None
                }
            } else {
                None
            };
            
            // Get assignee
            let assignee_id = if let Some(assignee) = issue_node.assignees.nodes.first() {
                if let Some(github_id) = assignee.database_id {
                    let conn = state.sqlite.lock().unwrap();
                    Some(queries::get_or_create_user(&conn, github_id, &assignee.login, None, None, None, None, None)?)
                } else {
                    None
                }
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
                    &issue_node.updated_at, // Use updated_at as sync_updated_at
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

    // Get watermark for PRs (note: PRs query doesn't support 'since' filter like issues, so we rely on upsert guards)
    let watermark = {
        let conn = state.sqlite.lock().unwrap();
        queries::get_prs_watermark(&conn, repo_id)?
    };
    tracing::info!("PR watermark for {}/{}: {:?}", owner, name, watermark);

    let mut cursor: Option<String> = None;
    let mut total_synced = 0;

    loop {
        let variables = serde_json::json!({
            "owner": owner,
            "name": name,
            "cursor": cursor
        });

        let response: PullRequestsResponse = match graphql::execute_query(token, PULL_REQUESTS_QUERY, variables).await {
            Ok(resp) => resp,
            Err(GraphQLExecuteError::SamlRequired { owner: err_owner, repo: err_repo, org: _org }) => {
                tracing::warn!(
                    "⚠️  SAML SSO required for {}/{}, trying REST API fallback...",
                    err_owner, err_repo
                );

                // Try REST API fallback
                return sync_pull_requests_rest_fallback(state, token, repo_id, owner, name, excluded_bots).await;
            }
            Err(e) => {
                return Err(anyhow::anyhow!("GraphQL error: {}", e));
            }
        };
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
                if let Some(github_id) = author.database_id {
                    let conn = state.sqlite.lock().unwrap();
                    Some(queries::get_or_create_user(&conn, github_id, &author.login, None, None, None, None, None)?)
                } else {
                    None
                }
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
                    &pr_node.updated_at, // Use updated_at as sync_updated_at
                )?
            };
            
            // Sync reviews for this PR
            for review in &pr_node.reviews.nodes {
                let reviewer_id = if let Some(author) = &review.author {
                    if let Some(github_id) = author.database_id {
                        let conn = state.sqlite.lock().unwrap();
                        Some(queries::get_or_create_user(&conn, github_id, &author.login, None, None, None, None, None)?)
                    } else {
                        None
                    }
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
                        submitted_at, // Use submitted_at as sync_updated_at for reviews
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

    let response: MilestonesResponse = match graphql::execute_query(token, MILESTONES_QUERY, variables).await {
        Ok(resp) => resp,
        Err(GraphQLExecuteError::SamlRequired { owner: err_owner, repo: err_repo, org: _org }) => {
            tracing::warn!(
                "⚠️  SAML SSO required for {}/{}, trying REST API fallback...",
                err_owner, err_repo
            );

            // Try REST API fallback
            return sync_milestones_rest_fallback(state, token, repo_id, owner, name).await;
        }
        Err(e) => {
            return Err(anyhow::anyhow!("GraphQL error: {}", e));
        }
    };
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

/// REST API fallback for syncing issues when GraphQL fails due to SAML
async fn sync_issues_rest_fallback(
    state: &AppState,
    token: &str,
    repo_id: i64,
    owner: &str,
    name: &str,
    since: &str,
    excluded_bots: &[String],
) -> Result<()> {
    tracing::info!("Using REST API fallback for issues in {}/{}", owner, name);

    let log_id = {
        let conn = state.sqlite.lock().unwrap();
        queries::record_sync_start(&conn, repo_id, "issues")?
    };

    match rest_api::fetch_issues_rest(token, owner, name, since).await {
        Ok(issues) => {
            let mut total_synced = 0;

            for issue in &issues {
                // Skip pull requests (they come mixed in the issues endpoint)
                if issue.pull_request.is_some() {
                    continue;
                }

                // Skip bot authors
                if let Some(user) = &issue.user {
                    if is_bot_user(&user.login, excluded_bots) {
                        continue;
                    }
                }

                // Get or create author
                let author_id = if let Some(user) = &issue.user {
                    let conn = state.sqlite.lock().unwrap();
                    Some(queries::get_or_create_user(&conn, user.id, &user.login, None, None, None, None, None)?)
                } else {
                    None
                };

                // Get assignee
                let assignee_id = if let Some(assignee) = &issue.assignee {
                    let conn = state.sqlite.lock().unwrap();
                    Some(queries::get_or_create_user(&conn, assignee.id, &assignee.login, None, None, None, None, None)?)
                } else {
                    None
                };

                // Get milestone ID
                let milestone_id = if let Some(milestone) = &issue.milestone {
                    let conn = state.sqlite.lock().unwrap();
                    queries::get_milestone_id_by_github_id(&conn, milestone.id)?
                } else {
                    None
                };

                // Extract labels
                let labels: Vec<String> = issue.labels.iter().map(|l| l.name.clone()).collect();

                // Upsert issue
                {
                    let conn = state.sqlite.lock().unwrap();
                    queries::upsert_issue(
                        &conn,
                        issue.id,
                        repo_id,
                        issue.number,
                        &issue.title,
                        issue.body.as_deref(),
                        &issue.state,
                        author_id,
                        assignee_id,
                        milestone_id,
                        &issue.created_at,
                        &issue.updated_at,
                        issue.closed_at.as_deref(),
                        &labels,
                        &issue.updated_at, // Use updated_at as sync_updated_at
                    )?;
                }

                total_synced += 1;
            }

            let conn = state.sqlite.lock().unwrap();
            queries::record_sync_complete(&conn, log_id, total_synced)?;

            tracing::info!("✅ REST API fallback succeeded: Synced {} issues for {}/{}", total_synced, owner, name);
            Ok(())
        }
        Err(rest_error) => {
            tracing::warn!("❌ REST API fallback failed for {}/{}: {}", owner, name, rest_error);
            tracing::info!("⚙️  Trying GitHub CLI fallback...");

            // Try CLI as final fallback
            return sync_issues_cli_fallback(state, repo_id, owner, name, excluded_bots).await;
        }
    }
}

/// REST API fallback for syncing pull requests when GraphQL fails due to SAML
async fn sync_pull_requests_rest_fallback(
    state: &AppState,
    token: &str,
    repo_id: i64,
    owner: &str,
    name: &str,
    excluded_bots: &[String],
) -> Result<()> {
    tracing::info!("Using REST API fallback for PRs in {}/{}", owner, name);

    let log_id = {
        let conn = state.sqlite.lock().unwrap();
        queries::record_sync_start(&conn, repo_id, "pull_requests")?
    };

    match rest_api::fetch_pull_requests_rest(token, owner, name).await {
        Ok(prs) => {
            let mut total_synced = 0;

            for pr in &prs {
                // Skip bot authors
                if let Some(user) = &pr.user {
                    if is_bot_user(&user.login, excluded_bots) {
                        continue;
                    }
                }

                // Get or create author
                let author_id = if let Some(user) = &pr.user {
                    let conn = state.sqlite.lock().unwrap();
                    Some(queries::get_or_create_user(&conn, user.id, &user.login, None, None, None, None, None)?)
                } else {
                    None
                };

                // Extract labels
                let labels: Vec<String> = pr.labels.iter().map(|l| l.name.clone()).collect();

                // Upsert PR
                {
                    let conn = state.sqlite.lock().unwrap();
                    queries::upsert_pull_request(
                        &conn,
                        pr.id,
                        repo_id,
                        pr.number,
                        &pr.title,
                        pr.body.as_deref(),
                        &pr.state,
                        author_id,
                        &pr.created_at,
                        &pr.updated_at,
                        pr.merged_at.as_deref(),
                        pr.closed_at.as_deref(),
                        pr.additions.unwrap_or(0),
                        pr.deletions.unwrap_or(0),
                        pr.changed_files.unwrap_or(0),
                        &labels,
                        &pr.updated_at, // Use updated_at as sync_updated_at
                    )?;
                }

                total_synced += 1;
            }

            let conn = state.sqlite.lock().unwrap();
            queries::record_sync_complete(&conn, log_id, total_synced)?;

            tracing::info!("✅ REST API fallback succeeded: Synced {} PRs for {}/{}", total_synced, owner, name);
            Ok(())
        }
        Err(rest_error) => {
            tracing::warn!("❌ REST API fallback failed for {}/{}: {}", owner, name, rest_error);
            tracing::info!("⚙️  Trying GitHub CLI fallback...");

            // Try CLI as final fallback
            return sync_pull_requests_cli_fallback(state, repo_id, owner, name, excluded_bots).await;
        }
    }
}

/// REST API fallback for syncing milestones when GraphQL fails due to SAML
async fn sync_milestones_rest_fallback(
    state: &AppState,
    token: &str,
    repo_id: i64,
    owner: &str,
    name: &str,
) -> Result<()> {
    tracing::info!("Using REST API fallback for milestones in {}/{}", owner, name);

    let log_id = {
        let conn = state.sqlite.lock().unwrap();
        queries::record_sync_start(&conn, repo_id, "milestones")?
    };

    match rest_api::fetch_milestones_rest(token, owner, name).await {
        Ok(milestones) => {
            let total_synced = milestones.len() as i32;

            for milestone in &milestones {
                let conn = state.sqlite.lock().unwrap();
                queries::upsert_milestone(
                    &conn,
                    milestone.id,
                    repo_id,
                    &milestone.title,
                    milestone.description.as_deref(),
                    &milestone.state,
                    milestone.due_on.as_deref(),
                    milestone.open_issues,
                    milestone.closed_issues,
                )?;
            }

            let conn = state.sqlite.lock().unwrap();
            queries::record_sync_complete(&conn, log_id, total_synced)?;

            tracing::info!("✅ REST API fallback succeeded: Synced {} milestones for {}/{}", total_synced, owner, name);
            Ok(())
        }
        Err(rest_error) => {
            tracing::warn!("❌ REST API fallback failed for {}/{}: {}", owner, name, rest_error);
            tracing::info!("⚙️  Trying GitHub CLI fallback...");

            // Try CLI as final fallback
            return sync_milestones_cli_fallback(state, repo_id, owner, name).await;
        }
    }
}

/// GitHub CLI fallback for syncing issues when both GraphQL and REST API fail
async fn sync_issues_cli_fallback(
    state: &AppState,
    repo_id: i64,
    owner: &str,
    name: &str,
    excluded_bots: &[String],
) -> Result<()> {
    tracing::info!("🔧 Using GitHub CLI fallback for issues in {}/{}", owner, name);

    let log_id = {
        let conn = state.sqlite.lock().unwrap();
        queries::record_sync_start(&conn, repo_id, "issues")?
    };

    // Initialize CLI client
    let cli = match GitHubCli::new().await {
        Ok(cli) => cli,
        Err(e) => {
            tracing::error!("❌ GitHub CLI not available: {}", e);
            tracing::warn!("   Install GitHub CLI from: https://cli.github.com");
            return Ok(()); // Don't fail the entire sync
        }
    };

    // Check if CLI is authenticated
    if let Err(e) = cli.check_auth() {
        tracing::error!("❌ GitHub CLI not authenticated: {}", e);
        tracing::warn!("   Run: gh auth login");
        return Ok(()); // Don't fail the entire sync
    }

    match cli.fetch_issues(owner, name).await {
        Ok(mut issues) => {
            let mut total_synced = 0;

            for issue in &mut issues {
                issue.repo_id = repo_id;

                // Upsert issue (author and assignee IDs will be resolved later)
                let conn = state.sqlite.lock().unwrap();
                queries::upsert_issue(
                    &conn,
                    issue.github_id,
                    repo_id,
                    issue.number,
                    &issue.title,
                    issue.body.as_deref(),
                    &issue.state,
                    issue.author_id,
                    issue.assignee_id,
                    issue.milestone_id,
                    &issue.created_at,
                    &issue.updated_at,
                    issue.closed_at.as_deref(),
                    &issue.labels,
                    &issue.updated_at, // Use updated_at as sync_updated_at
                )?;

                total_synced += 1;
            }

            let conn = state.sqlite.lock().unwrap();
            queries::record_sync_complete(&conn, log_id, total_synced)?;

            tracing::info!("✅ GitHub CLI fallback succeeded: Synced {} issues for {}/{}", total_synced, owner, name);
            Ok(())
        }
        Err(e) => {
            tracing::error!("❌ GitHub CLI fallback failed for {}/{}: {}", owner, name, e);
            tracing::warn!("   All sync methods failed. Please ensure:");
            tracing::warn!("   1. You have access to this repository");
            tracing::warn!("   2. GitHub CLI is installed and authenticated: gh auth login");
            tracing::warn!("   3. For SAML-protected repos: gh auth status");
            Ok(()) // Don't fail the entire sync
        }
    }
}

/// GitHub CLI fallback for syncing pull requests when both GraphQL and REST API fail
async fn sync_pull_requests_cli_fallback(
    state: &AppState,
    repo_id: i64,
    owner: &str,
    name: &str,
    excluded_bots: &[String],
) -> Result<()> {
    tracing::info!("🔧 Using GitHub CLI fallback for PRs in {}/{}", owner, name);

    let log_id = {
        let conn = state.sqlite.lock().unwrap();
        queries::record_sync_start(&conn, repo_id, "pull_requests")?
    };

    // Initialize CLI client
    let cli = match GitHubCli::new().await {
        Ok(cli) => cli,
        Err(e) => {
            tracing::error!("❌ GitHub CLI not available: {}", e);
            tracing::warn!("   Install GitHub CLI from: https://cli.github.com");
            return Ok(()); // Don't fail the entire sync
        }
    };

    // Check if CLI is authenticated
    if let Err(e) = cli.check_auth() {
        tracing::error!("❌ GitHub CLI not authenticated: {}", e);
        tracing::warn!("   Run: gh auth login");
        return Ok(()); // Don't fail the entire sync
    }

    match cli.fetch_pull_requests_with_authors(owner, name).await {
        Ok(pr_data) => {
            let mut total_synced = 0;

            for (cli_pr, author_login) in pr_data {
                // Skip bot authors
                if let Some(ref login) = author_login {
                    if is_bot_user(login, excluded_bots) {
                        continue;
                    }
                }

                // Try to find author by login in database
                let author_id = if let Some(login) = author_login {
                    let conn = state.sqlite.lock().unwrap();
                    queries::get_user_by_login(&conn, &login)
                        .ok()
                        .flatten()
                        .map(|u| u.id)
                } else {
                    None
                };

                // Upsert PR
                let pr_id = {
                    let conn = state.sqlite.lock().unwrap();
                    queries::upsert_pull_request(
                        &conn,
                        cli_pr.github_id,
                        repo_id,
                        cli_pr.number,
                        &cli_pr.title,
                        cli_pr.body.as_deref(),
                        &cli_pr.state,
                        author_id,
                        &cli_pr.created_at,
                        &cli_pr.updated_at,
                        cli_pr.merged_at.as_deref(),
                        cli_pr.closed_at.as_deref(),
                        cli_pr.additions,
                        cli_pr.deletions,
                        cli_pr.changed_files,
                        &cli_pr.labels,
                        &cli_pr.updated_at, // Use updated_at as sync_updated_at
                    )?
                };

                // Fetch and sync PR reviews
                if let Ok(reviews) = cli.fetch_pr_reviews(owner, name, cli_pr.number).await {
                    for review in reviews {
                        // Try to find reviewer in database (reviews don't include GitHub IDs via CLI)
                        let reviewer_id = None; // Would need REST API call to get reviewer GitHub ID
                        
                        let conn = state.sqlite.lock().unwrap();
                        queries::upsert_pr_review(
                            &conn,
                            review.github_id,
                            pr_id,
                            reviewer_id,
                            &review.state,
                            &review.submitted_at,
                            &review.submitted_at, // Use submitted_at as sync_updated_at for reviews
                        ).ok(); // Ignore errors for individual reviews
                    }
                }

                total_synced += 1;
            }

            let conn = state.sqlite.lock().unwrap();
            queries::record_sync_complete(&conn, log_id, total_synced)?;

            tracing::info!("✅ GitHub CLI fallback succeeded: Synced {} PRs for {}/{}", total_synced, owner, name);
            Ok(())
        }
        Err(e) => {
            tracing::error!("❌ GitHub CLI fallback failed for {}/{}: {}", owner, name, e);
            tracing::warn!("   All sync methods failed. Please ensure:");
            tracing::warn!("   1. You have access to this repository");
            tracing::warn!("   2. GitHub CLI is installed and authenticated: gh auth login");
            tracing::warn!("   3. For SAML-protected repos: gh auth status");
            Ok(()) // Don't fail the entire sync
        }
    }
}

/// GitHub CLI fallback for syncing milestones when both GraphQL and REST API fail
async fn sync_milestones_cli_fallback(
    state: &AppState,
    repo_id: i64,
    owner: &str,
    name: &str,
) -> Result<()> {
    tracing::info!("🔧 Using GitHub CLI fallback for milestones in {}/{}", owner, name);

    let log_id = {
        let conn = state.sqlite.lock().unwrap();
        queries::record_sync_start(&conn, repo_id, "milestones")?
    };

    // Initialize CLI client
    let cli = match GitHubCli::new().await {
        Ok(cli) => cli,
        Err(e) => {
            tracing::error!("❌ GitHub CLI not available: {}", e);
            tracing::warn!("   Install GitHub CLI from: https://cli.github.com");
            return Ok(()); // Don't fail the entire sync
        }
    };

    // Check if CLI is authenticated
    if let Err(e) = cli.check_auth() {
        tracing::error!("❌ GitHub CLI not authenticated: {}", e);
        tracing::warn!("   Run: gh auth login");
        return Ok(()); // Don't fail the entire sync
    }

    match cli.fetch_milestones(owner, name).await {
        Ok(milestones) => {
            let total_synced = milestones.len() as i32;

            for milestone in &milestones {
                let conn = state.sqlite.lock().unwrap();
                queries::upsert_milestone(
                    &conn,
                    milestone.github_id,
                    repo_id,
                    &milestone.title,
                    milestone.description.as_deref(),
                    &milestone.state,
                    milestone.due_on.as_deref(),
                    milestone.open_issues,
                    milestone.closed_issues,
                )?;
            }

            let conn = state.sqlite.lock().unwrap();
            queries::record_sync_complete(&conn, log_id, total_synced)?;

            tracing::info!("✅ GitHub CLI fallback succeeded: Synced {} milestones for {}/{}", total_synced, owner, name);
            Ok(())
        }
        Err(e) => {
            tracing::error!("❌ GitHub CLI fallback failed for {}/{}: {}", owner, name, e);
            tracing::warn!("   All sync methods failed. Please ensure:");
            tracing::warn!("   1. You have access to this repository");
            tracing::warn!("   2. GitHub CLI is installed and authenticated: gh auth login");
            tracing::warn!("   3. For SAML-protected repos: gh auth status");
            Ok(()) // Don't fail the entire sync
        }
    }
}
