use super::generate_embeddings;
use crate::db::models::{Issue, PullRequest};
use anyhow::Result;

/// Prepare text for embedding from an issue
pub fn issue_to_embedding_text(issue: &Issue) -> String {
    let mut parts = vec![issue.title.clone()];
    
    if let Some(body) = &issue.body {
        // Truncate body to avoid very long texts
        let truncated = truncate_text(body, 1000);
        parts.push(truncated);
    }
    
    // Add labels as context
    if !issue.labels.is_empty() {
        parts.push(format!("Labels: {}", issue.labels.join(", ")));
    }
    
    parts.join("\n\n")
}

/// Prepare text for embedding from a PR
pub fn pr_to_embedding_text(pr: &PullRequest) -> String {
    let mut parts = vec![pr.title.clone()];
    
    if let Some(body) = &pr.body {
        let truncated = truncate_text(body, 1000);
        parts.push(truncated);
    }
    
    if !pr.labels.is_empty() {
        parts.push(format!("Labels: {}", pr.labels.join(", ")));
    }
    
    parts.join("\n\n")
}

/// Generate embeddings for issues that don't have them
pub fn generate_issue_embeddings(issues: &[Issue]) -> Result<Vec<(i64, Vec<f32>)>> {
    let texts: Vec<String> = issues
        .iter()
        .map(|i| issue_to_embedding_text(i))
        .collect();
    
    if texts.is_empty() {
        return Ok(vec![]);
    }
    
    let embeddings = generate_embeddings(&texts)?;
    
    let results: Vec<(i64, Vec<f32>)> = issues
        .iter()
        .zip(embeddings.into_iter())
        .map(|(issue, embedding)| (issue.id, embedding))
        .collect();
    
    Ok(results)
}

/// Generate embeddings for PRs that don't have them
pub fn generate_pr_embeddings(prs: &[PullRequest]) -> Result<Vec<(i64, Vec<f32>)>> {
    let texts: Vec<String> = prs
        .iter()
        .map(|p| pr_to_embedding_text(p))
        .collect();
    
    if texts.is_empty() {
        return Ok(vec![]);
    }
    
    let embeddings = generate_embeddings(&texts)?;
    
    let results: Vec<(i64, Vec<f32>)> = prs
        .iter()
        .zip(embeddings.into_iter())
        .map(|(pr, embedding)| (pr.id, embedding))
        .collect();
    
    Ok(results)
}

/// Truncate text to a maximum number of characters, preserving word boundaries
fn truncate_text(text: &str, max_chars: usize) -> String {
    if text.len() <= max_chars {
        return text.to_string();
    }
    
    // Find the last space before the limit
    let truncated = &text[..max_chars];
    match truncated.rfind(' ') {
        Some(pos) => format!("{}...", &text[..pos]),
        None => format!("{}...", truncated),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_issue_to_embedding_text() {
        let issue = Issue {
            id: 1,
            github_id: 1,
            repo_id: 1,
            number: 1,
            title: "Add authentication".to_string(),
            body: Some("We need OAuth support for the app".to_string()),
            state: "open".to_string(),
            author_id: None,
            assignee_id: None,
            milestone_id: None,
            created_at: "2024-01-01".to_string(),
            updated_at: "2024-01-01".to_string(),
            closed_at: None,
            labels: vec!["feature".to_string(), "auth".to_string()],
            embedding_id: None,
        };
        
        let text = issue_to_embedding_text(&issue);
        
        assert!(text.contains("Add authentication"));
        assert!(text.contains("OAuth support"));
        assert!(text.contains("Labels: feature, auth"));
    }

    #[test]
    fn test_truncate_text() {
        let long_text = "This is a very long text that should be truncated at some point to avoid issues with embedding generation";
        let truncated = truncate_text(long_text, 50);
        
        assert!(truncated.len() <= 53); // 50 + "..."
        assert!(truncated.ends_with("..."));
    }

    #[test]
    fn test_truncate_preserves_short_text() {
        let short_text = "Short text";
        let result = truncate_text(short_text, 50);
        assert_eq!(result, short_text);
    }
}
