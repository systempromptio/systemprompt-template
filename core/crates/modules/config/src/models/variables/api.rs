use super::{ConfigVariable, VariableCategory};
use serde::{Deserialize, Serialize};
use systemprompt_models::api::ApiQuery;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize)]
pub struct VariableResponse {
    pub id: String,
    pub name: String,
    pub value: Option<String>,
    pub variable_type: String,
    pub category: String,
    pub description: Option<String>,
    pub is_secret: Option<bool>,
    pub is_required: bool,
    pub default_value: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<ConfigVariable> for VariableResponse {
    fn from(v: ConfigVariable) -> Self {
        Self {
            id: v.id,
            name: v.name,
            value: v.value,
            variable_type: v.variable_type,
            category: v.category,
            description: v.description,
            is_secret: v.is_secret,
            is_required: v.is_required,
            default_value: v.default_value,
            created_at: v.created_at.to_rfc3339(),
            updated_at: v.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateVariableRequest {
    pub name: String,
    pub value: Option<String>,
    #[serde(rename = "type")]
    pub variable_type: String,
    pub category: Option<String>,
    pub description: Option<String>,
    pub is_secret: Option<bool>,
    pub is_required: Option<bool>,
    pub default_value: Option<String>,
}

impl CreateVariableRequest {
    pub fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("Variable name cannot be empty".to_string());
        }

        if !self
            .name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
        {
            return Err(format!(
                "Variable name '{}' contains invalid characters",
                self.name
            ));
        }

        if let Some(ref cat) = self.category {
            if VariableCategory::try_from_str(cat).is_none() {
                return Err(format!("Invalid category: {cat}"));
            }
        }

        Ok(())
    }
}

#[derive(Debug, Deserialize)]
pub struct UpdateVariableRequest {
    pub value: Option<String>,
    pub category: Option<String>,
    pub description: Option<String>,
    pub is_secret: Option<bool>,
    pub is_required: Option<bool>,
    pub default_value: Option<String>,
}

impl UpdateVariableRequest {
    pub fn validate(&self) -> Result<(), String> {
        if let Some(ref cat) = self.category {
            if VariableCategory::try_from_str(cat).is_none() {
                return Err(format!("Invalid category: {cat}"));
            }
        }

        Ok(())
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct ListVariablesQuery {
    #[serde(flatten)]
    pub common: ApiQuery,

    #[validate(length(min = 1, max = 50))]
    pub category: Option<String>,
}

impl ListVariablesQuery {
    pub const fn offset(&self) -> i32 {
        self.common.pagination.offset()
    }

    pub const fn page(&self) -> i32 {
        self.common.pagination.page
    }

    pub const fn per_page(&self) -> i32 {
        self.common.pagination.per_page
    }
}
