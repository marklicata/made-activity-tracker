use anyhow::{anyhow, Result};
use serde::Deserialize;
use std::process::Command;
use tokio::process::Command as AsyncCommand;

use crate::db::models::{Issue, Milestone, PrReview, PullRequest, User};

/// GitHub CLI client for fallback when GraphQL fails
pub struct GitHubCli {
    command_path: String,
    is_available: bool,
    is_authenticated: bool,
}

impl GitHubCli {
    /// Create a new GitHubCli instance and check availability
    pub async fn new() -> Result<Self> {
        let command_path = "gh".to_string();

        // Check if gh CLI is installed
        let is_available = Self::check_installed(&command_path).await;

        // Check if gh CLI is authenticated
        let is_authenticated = if is_available {
            Self::check_auth_internal(&command_path).await
        } else {
            false
        };

        Ok(Self {
            command_path,
            is_available,
            is_authenticated,
        })
    }

    /// Check if gh CLI is installed
    async fn check_installed(command: &str) -> bool {
        AsyncCommand::new(command)
            .arg("--version")
            .output()
            .await
            .is_ok()
    }

    /// Check if gh CLI is authenticated
    async fn check_auth_internal(command: &str) -> bool {
        let output = AsyncCommand::new(command)
            .arg("auth")
            .arg("status")
            .output()
            .await;

        match output {
            Ok(output) => output.status.success(),
            Err(_) => false,
        }
    }

    /// Check if CLI is available and authenticated
    pub fn check_auth(&self) -> Result<bool> {
        if !self.is_available {
            return Err(anyhow!(
                "GitHub CLI (gh) is not installed. Install from: https://cli.github.com"
            ));
        }

        if !self.is_authenticated {
            return Err(anyhow!(
                "GitHub CLI is not authenticated. Run: gh auth login"
            ));
        }

        Ok(true)
    }

    /// Fetch issues using gh CLI
    pub async fn fetch_issues(&self, owner: &str, repo: &str) -> Result<Vec<Issue>> {
        self.check_auth()?;

        tracing::info!("Fetching issues via CLI for {}/{}", owner, repo);

        let output = AsyncCommand::new(&self.command_path)
            .arg("issue")
            .arg("list")
            .arg("--repo")
            .arg(format!("{}/{}", owner, repo))
            .arg("--state")
            .arg("all")
            .arg("--limit")
            .arg("1000")
            .arg("--json")
            .arg("number,title,body,state,author,assignees,milestone,createdAt,updatedAt,closedAt,labels")
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("gh issue list failed: {}", stderr));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let cli_issues: Vec<CliIssue> = serde_json::from_str(&stdout)
            .map_err(|e| anyhow!("Failed to parse CLI issue response: {}", e))?;

        // Convert CLI format to our Issue model
        let issues: Vec<Issue> = cli_issues
            .into_iter()
            .map(|cli_issue| Issue {
                id: 0, // Will be assigned by database
                github_id: cli_issue.number as i64, // Using number as temp ID
                repo_id: 0, // Will be set by caller
                number: cli_issue.number,
                title: cli_issue.title,
                body: cli_issue.body,
                state: cli_issue.state,
                author_id: None, // Will be resolved later
                assignee_id: None, // Will be resolved later
                milestone_id: None, // Will be resolved later
                created_at: cli_issue.created_at.clone(),
                updated_at: cli_issue.updated_at.clone(),
                sync_updated_at: Some(cli_issue.updated_at),
                closed_at: cli_issue.closed_at,
                labels: cli_issue
                    .labels
                    .iter()
                    .map(|l| l.name.clone())
                    .collect(),
            })
            .collect();

        tracing::info!("Fetched {} issues via CLI", issues.len());
        Ok(issues)
    }

    /// Fetch pull requests using gh CLI with author info
    pub async fn fetch_pull_requests_with_authors(&self, owner: &str, repo: &str) -> Result<Vec<(PullRequest, Option<String>)>> {
        self.check_auth()?;

        tracing::info!("Fetching PRs via CLI for {}/{}", owner, repo);

        let output = AsyncCommand::new(&self.command_path)
            .arg("pr")
            .arg("list")
            .arg("--repo")
            .arg(format!("{}/{}", owner, repo))
            .arg("--state")
            .arg("all")
            .arg("--limit")
            .arg("1000")
            .arg("--json")
            .arg("number,title,body,state,author,createdAt,updatedAt,mergedAt,closedAt,additions,deletions,changedFiles,labels")
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("gh pr list failed: {}", stderr));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let cli_prs: Vec<CliPullRequest> = serde_json::from_str(&stdout)
            .map_err(|e| anyhow!("Failed to parse CLI PR response: {}", e))?;

        let result: Vec<(PullRequest, Option<String>)> = cli_prs
            .into_iter()
            .map(|cli_pr| {
                let author_login = cli_pr.author.as_ref().map(|a| a.login.clone());
                let pr = PullRequest {
                    id: 0,
                    github_id: cli_pr.number as i64,
                    repo_id: 0,
                    number: cli_pr.number,
                    title: cli_pr.title.clone(),
                    body: cli_pr.body.clone(),
                    state: cli_pr.state.clone(),
                    author_id: None,
                    created_at: cli_pr.created_at.clone(),
                    updated_at: cli_pr.updated_at.clone(),
                    sync_updated_at: Some(cli_pr.updated_at),
                    merged_at: cli_pr.merged_at,
                    closed_at: cli_pr.closed_at,
                    additions: cli_pr.additions,
                    deletions: cli_pr.deletions,
                    changed_files: cli_pr.changed_files,
                    review_comments: 0,
                    labels: cli_pr.labels.iter().map(|l| l.name.clone()).collect(),
                };
                (pr, author_login)
            })
            .collect();

        tracing::info!("Fetched {} PRs via CLI", result.len());
        Ok(result)
    }

    /// Fetch pull requests using gh CLI
    pub async fn fetch_pull_requests(&self, owner: &str, repo: &str) -> Result<Vec<PullRequest>> {
        self.check_auth()?;

        tracing::info!("Fetching PRs via CLI for {}/{}", owner, repo);

        let output = AsyncCommand::new(&self.command_path)
            .arg("pr")
            .arg("list")
            .arg("--repo")
            .arg(format!("{}/{}", owner, repo))
            .arg("--state")
            .arg("all")
            .arg("--limit")
            .arg("1000")
            .arg("--json")
            .arg("number,title,body,state,author,createdAt,updatedAt,mergedAt,closedAt,additions,deletions,changedFiles,labels")
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("gh pr list failed: {}", stderr));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let cli_prs: Vec<CliPullRequest> = serde_json::from_str(&stdout)
            .map_err(|e| anyhow!("Failed to parse CLI PR response: {}", e))?;

        // Return raw CLI data - will be processed by sync function
        tracing::info!("Fetched {} PRs via CLI", cli_prs.len());
        
        // Convert to our model but preserve author login for later resolution
        let prs: Vec<PullRequest> = cli_prs
            .into_iter()
            .map(|cli_pr| PullRequest {
                id: 0,
                github_id: cli_pr.number as i64,
                repo_id: 0,
                number: cli_pr.number,
                title: cli_pr.title,
                body: cli_pr.body,
                state: cli_pr.state,
                author_id: None,
                created_at: cli_pr.created_at.clone(),
                updated_at: cli_pr.updated_at.clone(),
                sync_updated_at: Some(cli_pr.updated_at),
                merged_at: cli_pr.merged_at,
                closed_at: cli_pr.closed_at,
                additions: cli_pr.additions,
                deletions: cli_pr.deletions,
                changed_files: cli_pr.changed_files,
                review_comments: 0,
                labels: cli_pr.labels.iter().map(|l| l.name.clone()).collect(),
            })
            .collect();
        
        Ok(prs)
    }

    /// Fetch milestones using gh API via CLI
    pub async fn fetch_milestones(&self, owner: &str, repo: &str) -> Result<Vec<Milestone>> {
        self.check_auth()?;

        tracing::info!("Fetching milestones via CLI for {}/{}", owner, repo);

        let output = AsyncCommand::new(&self.command_path)
            .arg("api")
            .arg(format!("repos/{}/{}/milestones", owner, repo))
            .arg("--paginate")
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("gh api milestones failed: {}", stderr));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let cli_milestones: Vec<CliMilestone> = serde_json::from_str(&stdout)
            .map_err(|e| anyhow!("Failed to parse CLI milestone response: {}", e))?;

        // Convert CLI format to our Milestone model
        let milestones: Vec<Milestone> = cli_milestones
            .into_iter()
            .map(|cli_ms| Milestone {
                id: 0, // Will be assigned by database
                github_id: cli_ms.id,
                repo_id: 0, // Will be set by caller
                title: cli_ms.title,
                description: cli_ms.description,
                state: cli_ms.state,
                due_on: cli_ms.due_on,
                open_issues: cli_ms.open_issues,
                closed_issues: cli_ms.closed_issues,
            })
            .collect();

        tracing::info!("Fetched {} milestones via CLI", milestones.len());
        Ok(milestones)
    }

    /// Fetch PR reviews using gh API via CLI
    pub async fn fetch_pr_reviews(
        &self,
        owner: &str,
        repo: &str,
        pr_number: i32,
    ) -> Result<Vec<PrReview>> {
        self.check_auth()?;

        tracing::debug!(
            "Fetching reviews via CLI for {}/{} PR #{}",
            owner,
            repo,
            pr_number
        );

        let output = AsyncCommand::new(&self.command_path)
            .arg("api")
            .arg(format!(
                "repos/{}/{}/pulls/{}/reviews",
                owner, repo, pr_number
            ))
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("gh api reviews failed: {}", stderr));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let cli_reviews: Vec<CliReview> = serde_json::from_str(&stdout)
            .map_err(|e| anyhow!("Failed to parse CLI review response: {}", e))?;

        // Convert CLI format to our PrReview model
        let reviews: Vec<PrReview> = cli_reviews
            .into_iter()
            .map(|cli_rev| PrReview {
                id: 0, // Will be assigned by database
                github_id: cli_rev.id,
                pr_id: 0, // Will be set by caller
                reviewer_id: None, // Will be resolved later
                state: cli_rev.state.clone(),
                submitted_at: cli_rev.submitted_at.clone(),
                sync_updated_at: Some(cli_rev.submitted_at),
            })
            .collect();

        Ok(reviews)
    }
}

// CLI response structures matching gh CLI JSON output

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CliIssue {
    number: i32,
    title: String,
    body: Option<String>,
    state: String,
    author: Option<CliUser>,
    assignees: Vec<CliUser>,
    milestone: Option<CliMilestoneRef>,
    created_at: String,
    updated_at: String,
    closed_at: Option<String>,
    labels: Vec<CliLabel>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CliPullRequest {
    number: i32,
    title: String,
    body: Option<String>,
    state: String,
    author: Option<CliUser>,
    created_at: String,
    updated_at: String,
    merged_at: Option<String>,
    closed_at: Option<String>,
    additions: i32,
    deletions: i32,
    changed_files: i32,
    labels: Vec<CliLabel>,
}

#[derive(Debug, Deserialize)]
struct CliMilestone {
    id: i64,
    title: String,
    description: Option<String>,
    state: String,
    due_on: Option<String>,
    open_issues: i32,
    closed_issues: i32,
}

#[derive(Debug, Deserialize)]
struct CliMilestoneRef {
    title: String,
}

#[derive(Debug, Deserialize)]
struct CliReview {
    id: i64,
    user: Option<CliUser>,
    state: String,
    submitted_at: String,
}

#[derive(Debug, Deserialize)]
struct CliUser {
    login: String,
}

#[derive(Debug, Deserialize)]
struct CliLabel {
    name: String,
}
