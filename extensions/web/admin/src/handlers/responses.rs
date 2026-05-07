use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct ResolutionTokenResponse {
    pub token: String,
    pub expires_in: u32,
}

#[derive(Serialize, Debug, Clone, Copy)]
pub struct ResultOkResponse {
    pub result: &'static str,
}

#[derive(Serialize, Debug)]
pub struct AuditLogEntry {
    pub id: String,
    pub var_name: String,
    pub action: String,
    pub actor_id: String,
    pub ip_address: Option<String>,
    pub created_at: String,
}

#[derive(Serialize, Debug)]
pub struct PluginEnvResponse {
    pub definitions: Vec<serde_json::Value>,

    pub stored: Vec<crate::repositories::plugin_env::PluginEnvVar>,
    pub valid: bool,
    pub missing_required: Vec<String>,
}

macro_rules! list_response {
    ($name:ident, $field:ident) => {
        #[derive(Serialize)]
        pub struct $name<T> {
            pub $field: T,
        }
    };
}

list_response!(PluginsListResponse, plugins);
list_response!(SkillsListResponse, skills);
list_response!(AgentsListResponse, agents);
list_response!(SecretsListResponse, secrets);
list_response!(JobsListResponse, jobs);
list_response!(UsersListResponse, users);
list_response!(EventsListResponse, events);
list_response!(AuditLogListResponse, entries);
