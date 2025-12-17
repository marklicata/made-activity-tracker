use anyhow::Result;
use serde::Deserialize;

const GITHUB_API_BASE: &str = "https://api.github.com";

/// Fallback: Fetch issues using REST API (may work when GraphQL fails due to SAML)
pub async fn fetch_issues_rest(
    token: &str,
    owner: &str,
    repo: &str,
    since: &str,
) -> Result<Vec<RestIssue>> {
    let client = reqwest::Client::new();
    let mut all_issues = Vec::new();
    let mut page = 1;

    loop {
        let url = format!(
            "{}/repos/{}/{}/issues?state=all&since={}&per_page=100&page={}",
            GITHUB_API_BASE, owner, repo, since, page
        );

        let response = client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .header("User-Agent", "MADE-Activity-Tracker")
            .header("Accept", "application/vnd.github.v3+json")
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await?;
            anyhow::bail!("REST API error ({}): {}", status, body);
        }

        let issues: Vec<RestIssue> = response.json().await?;

        if issues.is_empty() {
            break;
        }

        all_issues.extend(issues);
        page += 1;

        // REST API returns 100 per page, if we get less than 100, we're done
        if all_issues.len() % 100 != 0 {
            break;
        }
    }

    Ok(all_issues)
}

/// Fallback: Fetch pull requests using REST API
pub async fn fetch_pull_requests_rest(
    token: &str,
    owner: &str,
    repo: &str,
) -> Result<Vec<RestPullRequest>> {
    let client = reqwest::Client::new();
    let mut all_prs = Vec::new();
    let mut page = 1;

    loop {
        let url = format!(
            "{}/repos/{}/{}/pulls?state=all&per_page=100&page={}",
            GITHUB_API_BASE, owner, repo, page
        );

        let response = client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .header("User-Agent", "MADE-Activity-Tracker")
            .header("Accept", "application/vnd.github.v3+json")
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await?;
            anyhow::bail!("REST API error ({}): {}", status, body);
        }

        let prs: Vec<RestPullRequest> = response.json().await?;

        if prs.is_empty() {
            break;
        }

        all_prs.extend(prs);
        page += 1;

        if all_prs.len() % 100 != 0 {
            break;
        }
    }

    Ok(all_prs)
}

/// Fallback: Fetch milestones using REST API
pub async fn fetch_milestones_rest(
    token: &str,
    owner: &str,
    repo: &str,
) -> Result<Vec<RestMilestone>> {
    let client = reqwest::Client::new();
    let url = format!(
        "{}/repos/{}/{}/milestones?state=all&per_page=100",
        GITHUB_API_BASE, owner, repo
    );

    let response = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", token))
        .header("User-Agent", "MADE-Activity-Tracker")
        .header("Accept", "application/vnd.github.v3+json")
        .send()
        .await?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await?;
        anyhow::bail!("REST API error ({}): {}", status, body);
    }

    let milestones: Vec<RestMilestone> = response.json().await?;
    Ok(milestones)
}

// REST API response types
#[derive(Debug, Deserialize)]
pub struct RestIssue {
    pub id: i64,
    pub number: i32,
    pub title: String,
    pub body: Option<String>,
    pub state: String,
    pub user: Option<RestUser>,
    pub assignee: Option<RestUser>,
    pub milestone: Option<RestMilestone>,
    pub labels: Vec<RestLabel>,
    pub created_at: String,
    pub updated_at: String,
    pub closed_at: Option<String>,
    pub pull_request: Option<RestPullRequestRef>,
}

#[derive(Debug, Deserialize)]
pub struct RestPullRequest {
    pub id: i64,
    pub number: i32,
    pub title: String,
    pub body: Option<String>,
    pub state: String,
    pub user: Option<RestUser>,
    pub labels: Vec<RestLabel>,
    pub created_at: String,
    pub updated_at: String,
    pub merged_at: Option<String>,
    pub closed_at: Option<String>,
    pub additions: Option<i32>,
    pub deletions: Option<i32>,
    pub changed_files: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct RestMilestone {
    pub id: i64,
    pub number: i32,
    pub title: String,
    pub description: Option<String>,
    pub state: String,
    pub due_on: Option<String>,
    pub open_issues: i32,
    pub closed_issues: i32,
}

#[derive(Debug, Deserialize)]
pub struct RestUser {
    pub id: i64,
    pub login: String,
}

#[derive(Debug, Deserialize)]
pub struct RestLabel {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct RestPullRequestRef {
    pub url: String,
}
