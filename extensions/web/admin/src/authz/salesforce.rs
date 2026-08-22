//! The `salesforce` subject dimension.
//!
//! The value is derived, not assigned: a user holds `linked` exactly while a
//! `salesforce_user_identities` row exists for them — written by the SSO
//! callback, removed by Disconnect. Rules at `rule_type = 'salesforce'` are
//! what confine the Salesforce MCP server and the Salesforce marketplace
//! plugins to linked users; a passkey-only account holds no value, matches no
//! rule, and the resolver's default closes the gated entities.
//!
//! Resolved by lookup, like the department dimension, so link state binds on
//! the next request rather than at token issue time. Unlike department, link
//! and unlink are explicit events with handlers of their own, so they call
//! [`invalidate`] and revocation does not wait out the TTL.

use std::collections::HashMap;
use std::sync::{Arc, LazyLock};
use std::time::{Duration, Instant};

use async_trait::async_trait;
use sqlx::PgPool;
use systemprompt::identifiers::UserId;
use systemprompt_security::authz::{RuleType, SubjectAttributeProvider, SubjectDimension};
use tokio::sync::RwLock;

const SALESFORCE_SLUG: &str = "salesforce";

// Why: between `department` (100) and core's `ROLE` (200) — a Salesforce rule
// must out-rank the role band that grants the marketplace to every user, while
// an operator's department rule can still override it.
const SALESFORCE_PRECEDENCE: u16 = 150;

const SALESFORCE_TTL: Duration = Duration::from_secs(60);

// Why: The value a linked user holds. The rule rows seeded from
// `services/access-control/salesforce.yaml` are written against it.
pub const SALESFORCE_LINKED_VALUE: &str = "linked";

type SalesforceCache = HashMap<String, (Vec<String>, Instant)>;

static SALESFORCE_CACHE: LazyLock<RwLock<SalesforceCache>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

#[must_use]
pub fn salesforce_rule_type() -> RuleType {
    RuleType::extension(SALESFORCE_SLUG)
        .unwrap_or_else(|e| unreachable!("`{SALESFORCE_SLUG}` is a well-formed slug: {e}"))
}

#[must_use]
pub fn salesforce_dimension() -> SubjectDimension {
    SubjectDimension {
        rule_type: salesforce_rule_type(),
        label: "Salesforce",
        precedence: SALESFORCE_PRECEDENCE,
    }
}

// Why: Drop the cached value for one user, so a Salesforce link change takes
// effect on the next request instead of after the TTL.
pub async fn invalidate(user_id: &UserId) {
    let mut cache = SALESFORCE_CACHE.write().await;
    cache.remove(user_id.as_str());
}

#[derive(Debug)]
pub struct SalesforceAttributeProvider {
    pool: Arc<PgPool>,
}

impl SalesforceAttributeProvider {
    #[must_use]
    pub const fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    async fn cached(user_id: &UserId) -> Option<Vec<String>> {
        let cache = SALESFORCE_CACHE.read().await;
        cache
            .get(user_id.as_str())
            .filter(|(_, at)| at.elapsed() < SALESFORCE_TTL)
            .map(|(values, _)| values.clone())
    }

    async fn store(user_id: &UserId, values: &[String]) {
        let mut cache = SALESFORCE_CACHE.write().await;
        cache.insert(
            user_id.as_str().to_owned(),
            (values.to_vec(), Instant::now()),
        );
    }
}

#[async_trait]
impl SubjectAttributeProvider for SalesforceAttributeProvider {
    fn dimension(&self) -> SubjectDimension {
        salesforce_dimension()
    }

    // Why: `linked` while an identity row exists, otherwise nothing.
    //
    // Fails soft: a lookup error means "not linked", which hides the
    // Salesforce entities for this request rather than denying everything the
    // user can otherwise reach; the resolver's default already closes the
    // unmatched case.
    async fn values_for(&self, user_id: &UserId) -> Vec<String> {
        if let Some(values) = Self::cached(user_id).await {
            return values;
        }
        let looked_up = sqlx::query_scalar!(
            r#"SELECT 1 AS "one" FROM salesforce_user_identities WHERE user_id = $1"#,
            user_id.as_str()
        )
        .fetch_optional(self.pool.as_ref())
        .await;

        let values = match looked_up {
            Ok(Some(_)) => vec![SALESFORCE_LINKED_VALUE.to_owned()],
            Ok(None) => Vec::new(),
            Err(e) => {
                tracing::warn!(
                    error = %e, user_id = %user_id,
                    "salesforce identity lookup failed; resolving as not linked",
                );
                Vec::new()
            },
        };
        Self::store(user_id, &values).await;
        values
    }
}
