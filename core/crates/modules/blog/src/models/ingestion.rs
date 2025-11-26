use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestionReport {
    pub files_found: usize,
    pub files_processed: usize,
    pub errors: Vec<String>,
}

impl IngestionReport {
    pub const fn new() -> Self {
        Self {
            files_found: 0,
            files_processed: 0,
            errors: Vec::new(),
        }
    }

    pub fn is_success(&self) -> bool {
        self.errors.is_empty()
    }
}

impl Default for IngestionReport {
    fn default() -> Self {
        Self::new()
    }
}
