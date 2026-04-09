use serde::Serialize;

#[derive(Serialize)]
pub struct RulesResponse<T: Serialize> {
    pub rules: T,
}

#[derive(Serialize)]
pub struct ResolutionTokenResponse {
    pub token: String,
    pub expires_in: u32,
}

#[derive(Serialize)]
pub struct ResultOkResponse {
    pub result: &'static str,
}

#[derive(Serialize)]
pub struct AuditLogEntry {
    pub id: String,
    pub var_name: String,
    pub action: String,
    pub actor_id: String,
    pub ip_address: Option<String>,
    pub created_at: String,
}

#[derive(Serialize)]
pub struct PluginEnvResponse {
    pub definitions: Vec<serde_json::Value>,

    pub stored: Vec<crate::admin::repositories::plugin_env::PluginEnvVar>,
    pub valid: bool,
    pub missing_required: Vec<String>,
}

#[derive(Serialize)]
pub struct ImportUserBundleResponse {
    pub message: String,
    pub imported_count: u32,
}

macro_rules! list_response {
    ($name:ident, $field:ident) => {
        #[derive(Serialize)]
        pub struct $name<T: Serialize> {
            pub $field: T,
        }
    };
}

list_response!(PluginsListResponse, plugins);
list_response!(SkillsListResponse, skills);
list_response!(AgentsListResponse, agents);
list_response!(SecretsListResponse, secrets);
list_response!(McpServersListResponse, mcp_servers);
list_response!(HooksListResponse, hooks);

#[derive(Serialize)]
pub struct ForkablePluginItem {
    pub id: String,
    pub name: String,
    pub description: String,
    pub skill_count: usize,
    pub agent_count: usize,
    pub mcp_count: usize,
    pub already_forked: bool,
}

#[derive(Serialize)]
pub struct ForkableSkillItem {
    pub id: String,
    pub name: String,
    pub description: String,
    pub plugin_id: String,
    pub plugin_name: String,
    pub already_forked: bool,
}

#[derive(Serialize)]
pub struct ForkableAgentItem {
    pub id: String,
    pub name: String,
    pub description: String,
    pub plugin_id: String,
    pub plugin_name: String,
    pub already_forked: bool,
}

#[derive(Serialize)]
pub struct ForkPluginResponse<T: Serialize> {
    pub plugin: T,
    pub forked_skills: usize,
    pub forked_agents: usize,
}

#[derive(Serialize)]
pub struct BaseSkillContentResponse {
    pub skill_id: String,
    pub name: String,
    pub description: String,
    pub content: String,
    pub config: String,
}

list_response!(JobsListResponse, jobs);
list_response!(UsersListResponse, users);
list_response!(EventsListResponse, events);
list_response!(MarketplaceListResponse, plugins);
list_response!(FilesListResponse, files);
list_response!(SkillIdsListResponse, skill_ids);
list_response!(AuditLogListResponse, entries);
list_response!(VersionsListResponse, versions);
list_response!(ChangelogListResponse, entries);
