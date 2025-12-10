use super::column::Column;
use crate::artifacts::traits::ArtifactSchema;
use crate::artifacts::types::SortOrder;
use serde_json::{json, Value as JsonValue};

#[derive(Debug, Clone, Default)]
pub struct TableHints {
    pub columns: Vec<Column>,
    pub sortable_columns: Vec<String>,
    pub default_sort: Option<(String, SortOrder)>,
    pub filterable: bool,
    pub page_size: Option<usize>,
    pub row_click_enabled: bool,
}

impl TableHints {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_columns(mut self, columns: Vec<Column>) -> Self {
        self.columns = columns;
        self
    }

    pub fn with_sortable(mut self, columns: Vec<String>) -> Self {
        self.sortable_columns = columns;
        self
    }

    pub fn with_default_sort(mut self, column: String, order: SortOrder) -> Self {
        self.default_sort = Some((column, order));
        self
    }

    pub const fn filterable(mut self) -> Self {
        self.filterable = true;
        self
    }

    pub const fn with_page_size(mut self, size: usize) -> Self {
        self.page_size = Some(size);
        self
    }

    pub const fn with_row_click_enabled(mut self, enabled: bool) -> Self {
        self.row_click_enabled = enabled;
        self
    }
}

impl ArtifactSchema for TableHints {
    fn generate_schema(&self) -> JsonValue {
        let mut hints = json!({
            "columns": self.columns.iter().map(Column::name).collect::<Vec<_>>(),
            "sortable_columns": self.sortable_columns,
            "filterable": self.filterable,
        });

        if let Some((col, order)) = &self.default_sort {
            hints["default_sort"] = json!({
                "column": col,
                "order": order
            });
        }

        if let Some(size) = self.page_size {
            hints["page_size"] = json!(size);
        }

        if self.row_click_enabled {
            hints["row_click_enabled"] = json!(true);
        }

        let column_types: serde_json::Map<String, JsonValue> = self
            .columns
            .iter()
            .map(|c| (c.name().to_string(), json!(c.column_type())))
            .collect();
        hints["column_types"] = json!(column_types);

        hints
    }
}
