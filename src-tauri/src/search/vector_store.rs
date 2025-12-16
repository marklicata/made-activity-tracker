use anyhow::Result;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

use crate::db::queries::{get_issue_embedding, get_pr_embedding};
use super::duplicates::cosine_similarity;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorItem {
    pub id: i64,
    pub item_type: ItemType,
    pub embedding: Vec<f32>,
    pub title: String,
    pub repo_id: i64,
    pub number: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ItemType {
    Issue,
    PullRequest,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimilarityMatch {
    pub id: i64,
    pub item_type: ItemType,
    pub similarity: f32,
    pub title: String,
    pub repo_id: i64,
    pub number: i32,
}

/// Get all issue embeddings from the database
pub fn get_all_issue_embeddings(conn: &Connection) -> Result<Vec<VectorItem>> {
    let mut stmt = conn.prepare(
        "SELECT id, title, repo_id, number FROM issues WHERE embedding IS NOT NULL"
    )?;

    let mut items = Vec::new();
    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, i64>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, i64>(2)?,
            row.get::<_, i32>(3)?,
        ))
    })?;

    for row in rows {
        let (id, title, repo_id, number) = row?;
        if let Some(embedding) = get_issue_embedding(conn, id)? {
            items.push(VectorItem {
                id,
                item_type: ItemType::Issue,
                embedding,
                title,
                repo_id,
                number,
            });
        }
    }

    Ok(items)
}

/// Get all pull request embeddings from the database
pub fn get_all_pr_embeddings(conn: &Connection) -> Result<Vec<VectorItem>> {
    let mut stmt = conn.prepare(
        "SELECT id, title, repo_id, number FROM pull_requests WHERE embedding IS NOT NULL"
    )?;

    let mut items = Vec::new();
    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, i64>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, i64>(2)?,
            row.get::<_, i32>(3)?,
        ))
    })?;

    for row in rows {
        let (id, title, repo_id, number) = row?;
        if let Some(embedding) = get_pr_embedding(conn, id)? {
            items.push(VectorItem {
                id,
                item_type: ItemType::PullRequest,
                embedding,
                title,
                repo_id,
                number,
            });
        }
    }

    Ok(items)
}

/// Get all embeddings (both issues and PRs)
pub fn get_all_embeddings(conn: &Connection) -> Result<Vec<VectorItem>> {
    let mut items = Vec::new();
    items.extend(get_all_issue_embeddings(conn)?);
    items.extend(get_all_pr_embeddings(conn)?);
    Ok(items)
}

/// Search for similar vectors using brute-force cosine similarity
pub fn search_similar(
    query_embedding: &[f32],
    conn: &Connection,
    limit: usize,
    min_similarity: f32,
) -> Result<Vec<SimilarityMatch>> {
    let all_items = get_all_embeddings(conn)?;

    let mut matches: Vec<SimilarityMatch> = all_items
        .into_iter()
        .map(|item| {
            let similarity = cosine_similarity(query_embedding, &item.embedding);
            SimilarityMatch {
                id: item.id,
                item_type: item.item_type,
                similarity,
                title: item.title,
                repo_id: item.repo_id,
                number: item.number,
            }
        })
        .filter(|m| m.similarity >= min_similarity)
        .collect();

    // Sort by similarity descending
    matches.sort_by(|a, b| b.similarity.partial_cmp(&a.similarity).unwrap());

    // Return top N
    matches.truncate(limit);

    Ok(matches)
}

/// Search for similar vectors within a specific item type
pub fn search_similar_by_type(
    query_embedding: &[f32],
    conn: &Connection,
    item_type: ItemType,
    limit: usize,
    min_similarity: f32,
) -> Result<Vec<SimilarityMatch>> {
    let items = match item_type {
        ItemType::Issue => get_all_issue_embeddings(conn)?,
        ItemType::PullRequest => get_all_pr_embeddings(conn)?,
    };

    let mut matches: Vec<SimilarityMatch> = items
        .into_iter()
        .map(|item| {
            let similarity = cosine_similarity(query_embedding, &item.embedding);
            SimilarityMatch {
                id: item.id,
                item_type: item.item_type,
                similarity,
                title: item.title,
                repo_id: item.repo_id,
                number: item.number,
            }
        })
        .filter(|m| m.similarity >= min_similarity)
        .collect();

    // Sort by similarity descending
    matches.sort_by(|a, b| b.similarity.partial_cmp(&a.similarity).unwrap());

    // Return top N
    matches.truncate(limit);

    Ok(matches)
}

/// Find similar items excluding a specific ID (useful for duplicate detection)
pub fn find_similar_excluding(
    query_embedding: &[f32],
    conn: &Connection,
    exclude_id: i64,
    exclude_type: ItemType,
    limit: usize,
    min_similarity: f32,
) -> Result<Vec<SimilarityMatch>> {
    let all_items = get_all_embeddings(conn)?;

    let mut matches: Vec<SimilarityMatch> = all_items
        .into_iter()
        .filter(|item| !(item.id == exclude_id && item.item_type == exclude_type))
        .map(|item| {
            let similarity = cosine_similarity(query_embedding, &item.embedding);
            SimilarityMatch {
                id: item.id,
                item_type: item.item_type,
                similarity,
                title: item.title,
                repo_id: item.repo_id,
                number: item.number,
            }
        })
        .filter(|m| m.similarity >= min_similarity)
        .collect();

    // Sort by similarity descending
    matches.sort_by(|a, b| b.similarity.partial_cmp(&a.similarity).unwrap());

    // Return top N
    matches.truncate(limit);

    Ok(matches)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_item_type_equality() {
        assert_eq!(ItemType::Issue, ItemType::Issue);
        assert_eq!(ItemType::PullRequest, ItemType::PullRequest);
        assert_ne!(ItemType::Issue, ItemType::PullRequest);
    }
}
