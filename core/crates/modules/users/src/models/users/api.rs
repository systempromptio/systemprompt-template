use serde::{Deserialize, Serialize};
use systemprompt_models::api::ApiQuery;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateUserRequest {
    #[validate(length(min = 3, max = 50))]
    pub name: String,
    #[validate(email)]
    pub email: String,
    #[validate(length(max = 100))]
    pub full_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpdateUserRequest {
    #[validate(email)]
    pub email: Option<String>,
    #[validate(length(max = 100))]
    pub full_name: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ListUsersQuery {
    #[serde(flatten)]
    pub common: ApiQuery,

    #[validate(length(min = 1, max = 50))]
    pub status: Option<String>,

    #[validate(length(min = 1, max = 50))]
    pub role: Option<String>,
}
