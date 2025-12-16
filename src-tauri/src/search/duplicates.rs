use anyhow::Result;
use serde::{Deserialize, Serialize};

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

/// Find potential duplicates for a given item (stub for Phase 1)
/// Phase 3 will implement actual vector similarity search
pub async fn find_duplicates_for_item(
    _item_id: &str,
    _item_embedding: &[f32],
    _lancedb_path: &std::path::Path,
    _exclude_same_repo: bool,
    _item_repo_id: Option<i64>,
) -> Result<Vec<DuplicateMatch>> {
    // Stub: Return empty results
    // Real implementation will use LanceDB to find similar vectors
    Ok(vec![])
}

/// Batch find duplicates for all open issues (stub)
pub async fn find_all_duplicates(
    _lancedb_path: &std::path::Path,
) -> Result<Vec<(String, Vec<DuplicateMatch>)>> {
    // Stub for Phase 3
    Ok(vec![])
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
