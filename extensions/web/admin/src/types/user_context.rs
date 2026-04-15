use serde::Serialize;
use systemprompt::identifiers::{Email, UserId};

#[derive(Debug, Clone, Serialize)]
pub struct UserContext {
    pub user_id: UserId,
    pub username: String,
    pub email: Email,
    pub department: String,
    pub roles: Vec<String>,
    pub is_admin: bool,
    pub email_verified: bool,
}
