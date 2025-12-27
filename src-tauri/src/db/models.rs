use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    pub id: i64,
    pub owner: String,
    pub name: String,
    pub github_id: Option<i64>,
    pub enabled: bool,
    pub last_synced_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub github_id: i64,
    pub login: String,
    pub name: Option<String>,
    pub avatar_url: Option<String>,
    pub is_bot: bool,
    pub tracked: bool,
    pub tracked_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    pub id: i64,
    pub github_id: i64,
    pub repo_id: i64,
    pub number: i32,
    pub title: String,
    pub body: Option<String>,
    pub state: String,
    pub author_id: Option<i64>,
    pub assignee_id: Option<i64>,
    pub milestone_id: Option<i64>,
    pub created_at: String,
    pub updated_at: String,
    pub sync_updated_at: Option<String>,
    pub closed_at: Option<String>,
    pub labels: Vec<String>,
    // Note: embedding BLOB is stored in DB but not loaded in this model for performance
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequest {
    pub id: i64,
    pub github_id: i64,
    pub repo_id: i64,
    pub number: i32,
    pub title: String,
    pub body: Option<String>,
    pub state: String,
    pub author_id: Option<i64>,
    pub created_at: String,
    pub updated_at: String,
    pub sync_updated_at: Option<String>,
    pub merged_at: Option<String>,
    pub closed_at: Option<String>,
    pub additions: i32,
    pub deletions: i32,
    pub changed_files: i32,
    pub review_comments: i32,
    pub labels: Vec<String>,
    // Note: embedding BLOB is stored in DB but not loaded in this model for performance
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrReview {
    pub id: i64,
    pub github_id: i64,
    pub pr_id: i64,
    pub reviewer_id: Option<i64>,
    pub state: String,
    pub submitted_at: String,
    pub sync_updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Milestone {
    pub id: i64,
    pub github_id: i64,
    pub repo_id: i64,
    pub title: String,
    pub description: Option<String>,
    pub state: String,
    pub due_on: Option<String>,
    pub open_issues: i32,
    pub closed_issues: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Squad {
    pub id: String,
    pub name: String,
    pub color: Option<String>,
    pub members: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub id: i64,
    pub history_days: i32,
    pub excluded_bots: Vec<String>,
    pub bug_labels: Vec<String>,
    pub feature_labels: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}
