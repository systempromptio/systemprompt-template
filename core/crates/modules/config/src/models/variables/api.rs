use super::{Variable, VariableCategory, VariableType};
use serde::{Deserialize, Serialize};
use systemprompt_models::api::ApiQuery;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize)]
pub struct VariableResponse {
    pub id: i32,
    pub name: String,
    pub value: Option<String>,
    pub r#type: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub is_secret: bool,
    pub is_required: bool,
    pub default_value: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<Variable> for VariableResponse {
    fn from(v: Variable) -> Self {
        Self {
            id: v.id,
            name: v.name,
            // Hide secret values in API responses
            value: if v.is_secret { None } else { v.value },
            r#type: v.r#type,
            description: v.description,
            category: v.category,
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
    pub r#type: String,
    pub description: Option<String>,
    pub category: Option<String>,
    #[serde(default)]
    pub is_secret: bool,
    #[serde(default = "default_true")]
    pub is_required: bool,
    pub default_value: Option<String>,
}

const fn default_true() -> bool {
    true
}

impl CreateVariableRequest {
    pub fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("Variable name cannot be empty".to_string());
        }

        if VariableType::try_from_str(&self.r#type).is_none() {
            return Err(format!("Invalid variable type: {}", self.r#type));
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
    pub description: Option<String>,
    pub category: Option<String>,
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
