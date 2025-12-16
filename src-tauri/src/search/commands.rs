use super::duplicates::{find_duplicates_for_item, DuplicateMatch};
use super::hybrid::{hybrid_search as do_hybrid_search, SearchResult};
use crate::db::AppState;
use tauri::State;

#[derive(serde::Serialize)]
pub struct SearchResultWithDuplicates {
    #[serde(flatten)]
    pub result: SearchResult,
    pub duplicates: Option<Vec<DuplicateMatch>>,
}

/// Perform hybrid search with optional duplicate detection (stub for Phase 1)
#[tauri::command]
pub async fn hybrid_search(
    query: String,
    _include_duplicates: bool,
    state: State<'_, AppState>,
) -> Result<Vec<SearchResultWithDuplicates>, String> {
    let results = do_hybrid_search(&query, &state.lancedb_path, 20)
        .await
        .map_err(|e| e.to_string())?;
    
    // For Phase 1, return results without duplicates
    let results_with_duplicates: Vec<SearchResultWithDuplicates> = results
        .into_iter()
        .map(|result| SearchResultWithDuplicates {
            result,
            duplicates: None,
        })
        .collect();
    
    Ok(results_with_duplicates)
}

/// Find duplicates for a specific item (stub for Phase 1)
#[tauri::command]
pub async fn find_duplicates(
    _item_id: String,
    _item_type: String,
    state: State<'_, AppState>,
) -> Result<Vec<DuplicateMatch>, String> {
    // Stub: Return empty for Phase 1
    let embedding = vec![0.0f32; 384];
    
    find_duplicates_for_item(&_item_id, &embedding, &state.lancedb_path, false, None)
        .await
        .map_err(|e| e.to_string())
}
