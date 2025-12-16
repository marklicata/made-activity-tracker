use super::duplicates::{find_duplicates_for_item, DuplicateMatch};
use super::hybrid::{hybrid_search as do_hybrid_search, SearchResult};
use super::vector_store::ItemType;
use crate::db::{queries, AppState};
use tauri::State;

#[derive(serde::Serialize)]
pub struct SearchResultWithDuplicates {
    #[serde(flatten)]
    pub result: SearchResult,
    pub duplicates: Option<Vec<DuplicateMatch>>,
}

/// Perform hybrid search with optional duplicate detection
#[tauri::command]
pub async fn hybrid_search(
    query: String,
    include_duplicates: bool,
    state: State<'_, AppState>,
) -> Result<Vec<SearchResultWithDuplicates>, String> {
    let conn = state.sqlite.lock().unwrap();

    let results = do_hybrid_search(&query, &conn, 20)
        .map_err(|e| e.to_string())?;

    // Optionally find duplicates for each result
    let mut results_with_duplicates: Vec<SearchResultWithDuplicates> = Vec::new();

    for result in results {
        let duplicates = if include_duplicates {
            // Parse item ID and type from result
            let (item_id, item_type) = if result.id.starts_with("issue-") {
                (result.id.trim_start_matches("issue-").parse::<i64>().ok(), ItemType::Issue)
            } else if result.id.starts_with("pr-") {
                (result.id.trim_start_matches("pr-").parse::<i64>().ok(), ItemType::PullRequest)
            } else {
                (None, ItemType::Issue)
            };

            if let Some(id) = item_id {
                // Get embedding and find duplicates
                let embedding = match item_type {
                    ItemType::Issue => queries::get_issue_embedding(&conn, id).ok().flatten(),
                    ItemType::PullRequest => queries::get_pr_embedding(&conn, id).ok().flatten(),
                };

                if let Some(emb) = embedding {
                    find_duplicates_for_item(id, item_type, &emb, &conn, false, None)
                        .ok()
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        results_with_duplicates.push(SearchResultWithDuplicates {
            result,
            duplicates,
        });
    }

    Ok(results_with_duplicates)
}

/// Find duplicates for a specific item
#[tauri::command]
pub async fn find_duplicates(
    item_id: String,
    item_type: String,
    state: State<'_, AppState>,
) -> Result<Vec<DuplicateMatch>, String> {
    let conn = state.sqlite.lock().unwrap();

    // Parse item ID and type
    let (id, typ) = if item_id.starts_with("issue-") {
        (item_id.trim_start_matches("issue-").parse::<i64>(), ItemType::Issue)
    } else if item_id.starts_with("pr-") {
        (item_id.trim_start_matches("pr-").parse::<i64>(), ItemType::PullRequest)
    } else {
        // Try parsing as raw ID with type string
        let parsed_id = item_id.parse::<i64>();
        let typ = if item_type == "pull_request" {
            ItemType::PullRequest
        } else {
            ItemType::Issue
        };
        (parsed_id, typ)
    };

    let id = id.map_err(|e| format!("Invalid item ID: {}", e))?;

    // Get embedding
    let embedding = match typ {
        ItemType::Issue => queries::get_issue_embedding(&conn, id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "No embedding found for this issue".to_string())?,
        ItemType::PullRequest => queries::get_pr_embedding(&conn, id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "No embedding found for this PR".to_string())?,
    };

    // Find duplicates
    find_duplicates_for_item(id, typ, &embedding, &conn, false, None)
        .map_err(|e| e.to_string())
}
