use anyhow::Result;
use serde::Deserialize;

const GITHUB_GRAPHQL_URL: &str = "https://api.github.com/graphql";

/// Execute a GraphQL query against GitHub's API
pub async fn execute_query<T: for<'de> Deserialize<'de>>(
    token: &str,
    query: &str,
    variables: serde_json::Value,
) -> Result<T> {
    let client = reqwest::Client::new();
    
    let response = client
        .post(GITHUB_GRAPHQL_URL)
        .header("Authorization", format!("Bearer {}", token))
        .header("User-Agent", "MADE-Activity-Tracker")
        .json(&serde_json::json!({
            "query": query,
            "variables": variables
        }))
        .send()
        .await?;
    
    let status = response.status();
    let body = response.text().await?;
    
    if !status.is_success() {
        anyhow::bail!("GitHub API error ({}): {}", status, body);
    }
    
    let response_body: GraphQLResponse<T> = serde_json::from_str(&body)
        .map_err(|e| anyhow::anyhow!("Failed to parse response: {}. Body: {}", e, body))?;
    
    if let Some(errors) = response_body.errors {
        let error_messages: Vec<String> = errors.iter().map(|e| e.message.clone()).collect();
        anyhow::bail!("GraphQL errors: {}", error_messages.join(", "));
    }
    
    response_body.data.ok_or_else(|| anyhow::anyhow!("No data in response"))
}

#[derive(Debug, Deserialize)]
struct GraphQLResponse<T> {
    data: Option<T>,
    errors: Option<Vec<GraphQLError>>,
}

#[derive(Debug, Deserialize)]
struct GraphQLError {
    message: String,
}

// ============================================================================
// GRAPHQL QUERIES
// ============================================================================

/// Query for fetching issues
pub const ISSUES_QUERY: &str = r#"
query($owner: String!, $name: String!, $cursor: String, $since: DateTime) {
    repository(owner: $owner, name: $name) {
        issues(first: 100, after: $cursor, filterBy: { since: $since }, orderBy: {field: UPDATED_AT, direction: DESC}) {
            pageInfo {
                hasNextPage
                endCursor
            }
            nodes {
                id
                databaseId
                number
                title
                body
                state
                createdAt
                updatedAt
                closedAt
                author {
                    login
                }
                assignees(first: 1) {
                    nodes {
                        login
                    }
                }
                labels(first: 20) {
                    nodes {
                        name
                    }
                }
                milestone {
                    id
                    number
                    title
                }
            }
        }
    }
}
"#;

/// Query for fetching pull requests
pub const PULL_REQUESTS_QUERY: &str = r#"
query($owner: String!, $name: String!, $cursor: String) {
    repository(owner: $owner, name: $name) {
        pullRequests(first: 100, after: $cursor, orderBy: {field: UPDATED_AT, direction: DESC}) {
            pageInfo {
                hasNextPage
                endCursor
            }
            nodes {
                id
                databaseId
                number
                title
                body
                state
                createdAt
                updatedAt
                mergedAt
                closedAt
                additions
                deletions
                changedFiles
                author {
                    login
                }
                labels(first: 20) {
                    nodes {
                        name
                    }
                }
                reviews(first: 50) {
                    nodes {
                        id
                        databaseId
                        state
                        submittedAt
                        author {
                            login
                        }
                    }
                }
            }
        }
    }
}
"#;

/// Query for fetching milestones
pub const MILESTONES_QUERY: &str = r#"
query($owner: String!, $name: String!) {
    repository(owner: $owner, name: $name) {
        milestones(first: 100, orderBy: {field: DUE_DATE, direction: ASC}) {
            nodes {
                id
                number
                title
                description
                state
                dueOn
                issues {
                    totalCount
                }
                closedIssues: issues(states: CLOSED) {
                    totalCount
                }
            }
        }
    }
}
"#;

// ============================================================================
// ISSUES RESPONSE TYPES
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct IssuesResponse {
    pub repository: RepositoryIssues,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryIssues {
    pub issues: IssueConnection,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueConnection {
    pub page_info: PageInfo,
    pub nodes: Vec<IssueNode>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueNode {
    pub id: String,
    pub database_id: i64,
    pub number: i32,
    pub title: String,
    pub body: Option<String>,
    pub state: String,
    pub created_at: String,
    pub updated_at: String,
    pub closed_at: Option<String>,
    pub author: Option<Actor>,
    pub assignees: AssigneeConnection,
    pub labels: LabelConnection,
    pub milestone: Option<MilestoneRef>,
}

// ============================================================================
// PULL REQUESTS RESPONSE TYPES
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct PullRequestsResponse {
    pub repository: RepositoryPullRequests,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryPullRequests {
    pub pull_requests: PullRequestConnection,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PullRequestConnection {
    pub page_info: PageInfo,
    pub nodes: Vec<PullRequestNode>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PullRequestNode {
    pub id: String,
    pub database_id: i64,
    pub number: i32,
    pub title: String,
    pub body: Option<String>,
    pub state: String,
    pub created_at: String,
    pub updated_at: String,
    pub merged_at: Option<String>,
    pub closed_at: Option<String>,
    pub additions: i32,
    pub deletions: i32,
    pub changed_files: i32,
    pub author: Option<Actor>,
    pub labels: LabelConnection,
    pub reviews: ReviewConnection,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewConnection {
    pub nodes: Vec<ReviewNode>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewNode {
    pub id: String,
    pub database_id: i64,
    pub state: String,
    pub submitted_at: Option<String>,
    pub author: Option<Actor>,
}

// ============================================================================
// MILESTONES RESPONSE TYPES
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct MilestonesResponse {
    pub repository: RepositoryMilestones,
}

#[derive(Debug, Deserialize)]
pub struct RepositoryMilestones {
    pub milestones: MilestoneConnection,
}

#[derive(Debug, Deserialize)]
pub struct MilestoneConnection {
    pub nodes: Vec<MilestoneNode>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MilestoneNode {
    pub id: String,
    pub number: i32,
    pub title: String,
    pub description: Option<String>,
    pub state: String,
    pub due_on: Option<String>,
    pub issues: IssueCount,
    pub closed_issues: IssueCount,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueCount {
    pub total_count: i32,
}

// ============================================================================
// SHARED TYPES
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageInfo {
    pub has_next_page: bool,
    pub end_cursor: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Actor {
    pub login: String,
}

#[derive(Debug, Deserialize)]
pub struct AssigneeConnection {
    pub nodes: Vec<Actor>,
}

#[derive(Debug, Deserialize)]
pub struct LabelConnection {
    pub nodes: Vec<LabelNode>,
}

#[derive(Debug, Deserialize)]
pub struct LabelNode {
    pub name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MilestoneRef {
    pub id: String,
    pub number: i32,
    pub title: String,
}
