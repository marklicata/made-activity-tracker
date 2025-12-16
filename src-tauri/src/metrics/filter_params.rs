use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MetricsFilters {
    pub date_range: Option<DateRange>,
    pub repository_ids: Option<Vec<i64>>,
    pub squad_id: Option<String>,
    pub user_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DateRange {
    pub start: String,  // ISO 8601 format
    pub end: String,
}

impl Default for MetricsFilters {
    fn default() -> Self {
        Self {
            date_range: None,
            repository_ids: None,
            squad_id: None,
            user_id: None,
        }
    }
}
