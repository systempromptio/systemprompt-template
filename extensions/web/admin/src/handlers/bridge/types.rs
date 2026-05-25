use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct UserSection {
    pub id: String,
    pub name: String,
    pub email: String,
    pub display_name: Option<String>,
    pub roles: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct WhoamiResponse {
    pub user: UserSection,
    pub capabilities: Vec<&'static str>,
}

pub const COWORK_CAPABILITIES: &[&str] = &["plugins", "skills", "agents", "mcp", "user"];
