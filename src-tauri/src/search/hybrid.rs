use anyhow::{Context, Result};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

use crate::embeddings::generate_embedding;
use super::vector_store::{search_similar, ItemType};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: String,
    pub item_type: String, // "issue" or "pull_request"
    pub title: String,
    pub body_preview: String,
    pub repo: String,
    pub number: i32,
    pub state: String,
    pub author: String,
    pub created_at: String,
    pub url: String,
    pub score: f32,
}

/// Perform hybrid search using semantic similarity and keyword boost
pub fn hybrid_search(
    query: &str,
    conn: &Connection,
    limit: usize,
) -> Result<Vec<SearchResult>> {
    // Step 1: Generate query embedding
    let query_embedding = generate_embedding(query)
        .context("Failed to generate query embedding")?;

    // Step 2: Vector similarity search (get top 100 to allow for keyword reranking)
    let similarity_matches = search_similar(&query_embedding, conn, limit * 2, 0.3)?;

    if similarity_matches.is_empty() {
        return Ok(vec![]);
    }

    // Step 3: Convert to SearchResult and enrich with data
    let mut results = Vec::new();
    for m in similarity_matches {
        let search_result = match m.item_type {
            ItemType::Issue => {
                // Fetch full issue data
                let issue_opt = conn.query_row(
                    "SELECT i.id, i.title, i.body, i.number, i.state, i.created_at,
                            r.owner || '/' || r.name as repo, u.login as author
                     FROM issues i
                     JOIN repositories r ON i.repo_id = r.id
                     LEFT JOIN users u ON i.author_id = u.id
                     WHERE i.id = ?1",
                    [m.id],
                    |row| {
                        let body: String = row.get(2)?;
                        let body_preview = if body.len() > 200 {
                            format!("{}...", &body[..200])
                        } else {
                            body
                        };

                        Ok(SearchResult {
                            id: format!("issue-{}", m.id),
                            item_type: "issue".to_string(),
                            title: row.get(1)?,
                            body_preview,
                            repo: row.get(6)?,
                            number: row.get(3)?,
                            state: row.get(4)?,
                            author: row.get::<_, Option<String>>(7)?.unwrap_or_default(),
                            created_at: row.get(5)?,
                            url: format!("https://github.com/{}/issues/{}", row.get::<_, String>(6)?, row.get::<_, i32>(3)?),
                            score: m.similarity,
                        })
                    },
                );
                issue_opt.ok()
            }
            ItemType::PullRequest => {
                // Fetch full PR data
                let pr_opt = conn.query_row(
                    "SELECT pr.id, pr.title, pr.body, pr.number, pr.state, pr.created_at,
                            r.owner || '/' || r.name as repo, u.login as author
                     FROM pull_requests pr
                     JOIN repositories r ON pr.repo_id = r.id
                     LEFT JOIN users u ON pr.author_id = u.id
                     WHERE pr.id = ?1",
                    [m.id],
                    |row| {
                        let body: String = row.get(2)?;
                        let body_preview = if body.len() > 200 {
                            format!("{}...", &body[..200])
                        } else {
                            body
                        };

                        Ok(SearchResult {
                            id: format!("pr-{}", m.id),
                            item_type: "pull_request".to_string(),
                            title: row.get(1)?,
                            body_preview,
                            repo: row.get(6)?,
                            number: row.get(3)?,
                            state: row.get(4)?,
                            author: row.get::<_, Option<String>>(7)?.unwrap_or_default(),
                            created_at: row.get(5)?,
                            url: format!("https://github.com/{}/pull/{}", row.get::<_, String>(6)?, row.get::<_, i32>(3)?),
                            score: m.similarity,
                        })
                    },
                );
                pr_opt.ok()
            }
        };

        if let Some(result) = search_result {
            results.push(result);
        }
    }

    // Step 4: Apply keyword boost for reranking
    apply_keyword_boost(&mut results, query);

    // Step 5: Re-sort by boosted score and limit
    results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
    results.truncate(limit);

    Ok(results)
}

/// Rerank results using keyword matching boost
pub fn apply_keyword_boost(results: &mut [SearchResult], query: &str) {
    // Fixed: Create owned String first to avoid lifetime issues
    let query_lower = query.to_lowercase();
    let query_terms: Vec<&str> = query_lower.split_whitespace().collect();
    
    for result in results.iter_mut() {
        let title_lower = result.title.to_lowercase();
        let body_lower = result.body_preview.to_lowercase();
        
        let mut keyword_boost: f32 = 0.0;
        for term in &query_terms {
            if title_lower.contains(term) {
                keyword_boost += 0.1;
            }
            if body_lower.contains(term) {
                keyword_boost += 0.05;
            }
        }
        
        // Apply boost (max 30% boost)
        result.score *= 1.0 + keyword_boost.min(0.3);
    }
}
