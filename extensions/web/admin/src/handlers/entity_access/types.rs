//! Request/response DTOs for the generic entity-access handlers in
//! [`super`].

use serde::{Deserialize, Serialize};
use systemprompt_security::authz::AccessRule;

/// JSON body returned by [`super::list_entity_access_handler`].
#[derive(Debug, Serialize)]
pub(crate) struct EntityAccessResponse {
    pub entity_type: String,
    // Why: polymorphic entity reference (gateway_route/mcp_server), no single typed-ID equivalent
    pub entity_id: String,
    pub default_included: bool,
    pub rules: Vec<AccessRule>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct UpsertRuleBody {
    pub rule_type: String,
    pub rule_value: String,
    pub access: String,
    #[serde(default)]
    pub justification: Option<String>,
}

/// JSON body returned by [`super::upsert_entity_rule_handler`].
#[derive(Debug, Serialize)]
pub(crate) struct UpsertRuleResponse {
    pub rule: AccessRule,
}

#[derive(Debug, Deserialize)]
pub(crate) struct DefaultIncludedBody {
    pub default_included: bool,
}

/// JSON body returned by [`super::set_entity_default_handler`].
#[derive(Debug, Serialize)]
pub(crate) struct EntityDefaultResponse {
    pub entity_type: String,
    // Why: polymorphic entity reference (gateway_route/mcp_server), no single typed-ID equivalent
    pub entity_id: String,
    pub default_included: bool,
}

#[derive(Debug, Deserialize)]
pub(crate) struct AllAccessQuery {
    pub entity_type: String,
}

/// One entry in [`ListAllEntityAccessResponse::entities`].
#[derive(Debug, Serialize)]
pub(crate) struct EntityAccessEntry {
    // Why: polymorphic entity reference (gateway_route/mcp_server), no single typed-ID equivalent
    pub entity_id: String,
    pub default_included: bool,
    pub rules: Vec<AccessRule>,
}

/// JSON body returned by [`super::list_all_entity_access_handler`].
#[derive(Debug, Serialize)]
pub(crate) struct ListAllEntityAccessResponse {
    pub entity_type: String,
    pub entities: Vec<EntityAccessEntry>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ApplyTemplateBody {
    pub entity_type: String,
    pub subject_type: String,
    pub subject_value: String,
    /// One of: "allow", "deny", "clear".
    pub action: String,
}

/// JSON body returned by [`super::apply_template_handler`].
#[derive(Debug, Serialize)]
pub(crate) struct ApplyTemplateResponse {
    pub applied: usize,
    pub failed: usize,
    pub entity_count: usize,
}
