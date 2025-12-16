use anyhow::Result;
use serde::{Deserialize, Serialize};

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

/// Perform hybrid search (stub for Phase 1)
/// Phase 3 will implement actual LanceDB vector search
pub async fn hybrid_search(
    _query: &str,
    _lancedb_path: &std::path::Path,
    _limit: usize,
) -> Result<Vec<SearchResult>> {
    // Stub: Return empty results
    // Real implementation will use LanceDB vector search + keyword matching
    Ok(vec![])
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
