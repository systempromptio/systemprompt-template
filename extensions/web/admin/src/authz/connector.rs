//! Current database-backed connector attributes for authorization.
//!
//! The dimension's values are the MCP server ids whose connection is ready
//! for the user — configured, entitled, connected and verified, or a server
//! that needs no sign-in at all — so a rule written at
//! `rule_value = <server id>` opens an entity only to the people whose calls
//! to that server will actually work.

use async_trait::async_trait;
use sqlx::PgPool;
use std::sync::Arc;
use systemprompt::identifiers::UserId;
use systemprompt_security::authz::{
    AuthzError, RuleType, SubjectAttributeProvider, SubjectDimension,
};

use crate::services::connector_accounts::{Connection, get_connections};

const CONNECTOR_SLUG: &str = "connector";

// Why: above `group` (150) and below core's `role` (200). A connection rule
// is narrower than group membership — it names the people holding a grant,
// not a team — so it out-ranks the group band and still yields to a person
// rule.
const CONNECTOR_PRECEDENCE: u16 = 160;

#[must_use]
pub fn connector_rule_type() -> RuleType {
    RuleType::extension(CONNECTOR_SLUG)
        .unwrap_or_else(|e| unreachable!("`{CONNECTOR_SLUG}` is a well-formed slug: {e}"))
}

#[must_use]
pub fn connector_dimension() -> SubjectDimension {
    SubjectDimension {
        rule_type: connector_rule_type(),
        label: "Connected server",
        precedence: CONNECTOR_PRECEDENCE,
    }
}

// Why: the one answer to "will this person's calls to the server work",
// read off the same connection snapshot the Connections page renders.
fn is_ready(connection: &Connection) -> bool {
    if !connection.configured {
        return false;
    }
    if !connection.requires_auth {
        return true;
    }
    connection.entitled
        && matches!(
            connection.status.as_str(),
            "connected" | "temporarily_unavailable"
        )
        && connection.verified_at.is_some()
}

#[derive(Debug)]
pub struct ConnectorAttributeProvider {
    pool: Arc<PgPool>,
}

impl ConnectorAttributeProvider {
    #[must_use]
    pub const fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SubjectAttributeProvider for ConnectorAttributeProvider {
    fn dimension(&self) -> SubjectDimension {
        connector_dimension()
    }

    async fn values_for(&self, user_id: &UserId) -> Result<Vec<String>, AuthzError> {
        let snapshot = get_connections(&self.pool, user_id)
            .await
            .map_err(|e| AuthzError::Validation(e.to_string()))?;
        Ok(snapshot
            .connections
            .into_iter()
            .filter(is_ready)
            .map(|c| c.provider)
            .collect())
    }
}
