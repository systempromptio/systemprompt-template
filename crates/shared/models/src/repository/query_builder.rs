#[derive(Debug)]
pub struct WhereClause {
    conditions: Vec<String>,
    params: Vec<String>,
}

impl WhereClause {
    pub const fn new() -> Self {
        Self {
            conditions: vec![],
            params: vec![],
        }
    }

    pub fn eq(mut self, field: &str, value: impl Into<String>) -> Self {
        self.conditions.push(format!("{field} = ?"));
        self.params.push(value.into());
        self
    }

    pub fn not_null(mut self, field: &str) -> Self {
        self.conditions.push(format!("{field} IS NOT NULL"));
        self
    }

    pub fn null(mut self, field: &str) -> Self {
        self.conditions.push(format!("{field} IS NULL"));
        self
    }

    pub fn like(mut self, field: &str, pattern: impl Into<String>) -> Self {
        self.conditions.push(format!("{field} LIKE ?"));
        self.params.push(pattern.into());
        self
    }

    pub fn in_list(mut self, field: &str, values: Vec<String>) -> Self {
        let placeholders = values.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
        self.conditions.push(format!("{field} IN ({placeholders})"));
        self.params.extend(values);
        self
    }

    pub fn build(&self) -> (String, Vec<String>) {
        let clause = if self.conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", self.conditions.join(" AND "))
        };
        (clause, self.params.clone())
    }
}

impl Default for WhereClause {
    fn default() -> Self {
        Self::new()
    }
}
