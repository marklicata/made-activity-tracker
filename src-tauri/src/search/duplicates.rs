use anyhow::Result;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

use super::vector_store::{find_similar_excluding, ItemType};

const DUPLICATE_THRESHOLD: f32 = 0.85;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateMatch {
    pub id: String,
    pub title: String,
    pub repo: String,
    pub number: i32,
    pub similarity: f32,
    pub url: String,
}

/// Find potential duplicates for a given item using vector similarity
pub fn find_duplicates_for_item(
    item_id: i64,
    item_type: ItemType,
    item_embedding: &[f32],
    conn: &Connection,
    exclude_same_repo: bool,
    item_repo_id: Option<i64>,
) -> Result<Vec<DuplicateMatch>> {
    // Find similar items excluding the item itself
    let similar_items = find_similar_excluding(
        item_embedding,
        conn,
        item_id,
        item_type.clone(),
        10, // Top 10 potential duplicates
        DUPLICATE_THRESHOLD,
    )?;

    let mut duplicates = Vec::new();

    for sim in similar_items {
        // If exclude_same_repo is true, skip items from the same repo
        if exclude_same_repo {
            if let Some(repo_id) = item_repo_id {
                if sim.repo_id == repo_id {
                    continue;
                }
            }
        }

        // Fetch additional details
        let duplicate = match sim.item_type {
            ItemType::Issue => {
                conn.query_row(
                    "SELECT i.id, i.title, i.number, r.owner || '/' || r.name as repo
                     FROM issues i
                     JOIN repositories r ON i.repo_id = r.id
                     WHERE i.id = ?1",
                    [sim.id],
                    |row| {
                        Ok(DuplicateMatch {
                            id: format!("issue-{}", sim.id),
                            title: row.get(1)?,
                            repo: row.get(3)?,
                            number: row.get(2)?,
                            similarity: sim.similarity,
                            url: format!("https://github.com/{}/issues/{}", row.get::<_, String>(3)?, row.get::<_, i32>(2)?),
                        })
                    },
                ).ok()
            }
            ItemType::PullRequest => {
                conn.query_row(
                    "SELECT pr.id, pr.title, pr.number, r.owner || '/' || r.name as repo
                     FROM pull_requests pr
                     JOIN repositories r ON pr.repo_id = r.id
                     WHERE pr.id = ?1",
                    [sim.id],
                    |row| {
                        Ok(DuplicateMatch {
                            id: format!("pr-{}", sim.id),
                            title: row.get(1)?,
                            repo: row.get(3)?,
                            number: row.get(2)?,
                            similarity: sim.similarity,
                            url: format!("https://github.com/{}/pull/{}", row.get::<_, String>(3)?, row.get::<_, i32>(2)?),
                        })
                    },
                ).ok()
            }
        };

        if let Some(dup) = duplicate {
            duplicates.push(dup);
        }
    }

    Ok(duplicates)
}

/// Batch find duplicates for all open issues
pub fn find_all_duplicates(
    conn: &Connection,
) -> Result<Vec<(String, Vec<DuplicateMatch>)>> {
    // Get all open issues with embeddings
    let mut stmt = conn.prepare(
        "SELECT i.id, i.repo_id
         FROM issues i
         WHERE i.state = 'open' AND i.embedding IS NOT NULL"
    )?;

    let issues: Vec<(i64, i64)> = stmt.query_map([], |row| {
        Ok((row.get(0)?, row.get(1)?))
    })?.collect::<Result<Vec<_>, _>>()?;

    let mut all_duplicates = Vec::new();

    for (issue_id, repo_id) in issues {
        // Get embedding
        if let Some(embedding) = crate::db::queries::get_issue_embedding(conn, issue_id)? {
            // Find duplicates
            let duplicates = find_duplicates_for_item(
                issue_id,
                ItemType::Issue,
                &embedding,
                conn,
                false, // Don't exclude same repo for batch processing
                Some(repo_id),
            )?;

            if !duplicates.is_empty() {
                all_duplicates.push((format!("issue-{}", issue_id), duplicates));
            }
        }
    }

    Ok(all_duplicates)
}

/// Calculate cosine similarity between two embeddings
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return 0.0;
    }
    
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    
    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }
    
    dot / (norm_a * norm_b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_similarity_identical() {
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![1.0, 2.0, 3.0];
        
        let sim = cosine_similarity(&a, &b);
        assert!((sim - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_cosine_similarity_orthogonal() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![0.0, 1.0, 0.0];
        
        let sim = cosine_similarity(&a, &b);
        assert!(sim.abs() < 0.001);
    }

    #[test]
    fn test_cosine_similarity_opposite() {
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![-1.0, -2.0, -3.0];
        
        let sim = cosine_similarity(&a, &b);
        assert!((sim + 1.0).abs() < 0.001);
    }

    #[test]
    fn test_threshold() {
        assert!(DUPLICATE_THRESHOLD >= 0.0 && DUPLICATE_THRESHOLD <= 1.0);
    }
}
