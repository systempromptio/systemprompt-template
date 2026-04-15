use crate::types::UserPlugin;

#[derive(Debug, Clone, serde::Serialize)]
pub struct AssociatedEntity {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct UserPluginEnriched {
    pub plugin: UserPlugin,
    pub skills: Vec<AssociatedEntity>,
    pub agents: Vec<AssociatedEntity>,
    pub mcp_servers: Vec<AssociatedEntity>,
    pub skill_count: usize,
    pub agent_count: usize,
    pub mcp_count: usize,
}
