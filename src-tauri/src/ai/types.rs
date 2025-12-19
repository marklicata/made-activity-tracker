use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppContext {
    pub current_page: String,
    pub filters: FilterState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterState {
    pub date_range: Option<DateRange>,
    pub repositories: Vec<String>,
    pub squads: Vec<String>,
    pub users: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateRange {
    pub start: String,
    pub end: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    pub message: String,
    pub context: AppContext,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    pub response: String,
    pub context: AppContext,
}
