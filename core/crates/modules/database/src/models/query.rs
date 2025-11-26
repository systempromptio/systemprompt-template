use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    pub columns: Vec<String>,
    pub rows: Vec<QueryRow>,
    pub row_count: usize,
    pub execution_time_ms: u64,
}

pub type QueryRow = HashMap<String, Value>;

impl QueryResult {
    pub const fn new() -> Self {
        Self {
            columns: vec![],
            rows: vec![],
            row_count: 0,
            execution_time_ms: 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    pub fn first(&self) -> Option<&QueryRow> {
        self.rows.first()
    }
}

impl Default for QueryResult {
    fn default() -> Self {
        Self::new()
    }
}
