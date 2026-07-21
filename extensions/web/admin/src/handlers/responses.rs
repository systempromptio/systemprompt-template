use serde::Serialize;

#[derive(Serialize, Debug)]
pub(crate) struct ResolutionTokenResponse {
    pub token: String,
    pub expires_in: u32,
}

#[derive(Serialize, Debug, Clone, Copy)]
pub(crate) struct ResultOkResponse {
    pub result: &'static str,
}

#[derive(Serialize, Debug)]
pub(crate) struct AuditLogEntry {
    pub id: String,
    pub var_name: String,
    pub action: String,
    // Why: opaque actor identifier (serialized Actor), no typed-ID equivalent
    pub actor_id: String,
    pub ip_address: Option<String>,
    pub created_at: String,
}

#[derive(Serialize, Debug)]
pub(crate) struct PluginEnvResponse {
    pub definitions: Vec<serde_json::Value>,

    pub stored: Vec<crate::repositories::marketplace::plugin_env::PluginEnvVar>,
    pub valid: bool,
    pub missing_required: Vec<String>,
}

macro_rules! list_response {
    ($name:ident, $field:ident) => {
        #[derive(Serialize)]
        pub(crate) struct $name<T> {
            pub(crate) $field: T,
        }
    };
}

list_response!(PluginsListResponse, plugins);
list_response!(AgentsListResponse, agents);
list_response!(SecretsListResponse, secrets);
list_response!(JobsListResponse, jobs);
list_response!(UsersListResponse, users);
list_response!(EventsListResponse, events);
list_response!(AuditLogListResponse, entries);
