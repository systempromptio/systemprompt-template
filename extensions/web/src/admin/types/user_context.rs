use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct UserContext {
    pub user_id: String,
    pub username: String,
    pub email: String,
    pub roles: Vec<String>,
    pub department: String,
    pub is_admin: bool,
}
