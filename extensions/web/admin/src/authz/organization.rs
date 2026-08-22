//! The `organization` subject dimension.
//!
//! An organization is a paying customer on a pooled instance. Rules written at
//! `rule_type = 'organization'` are what a plan projects into
//! `access_control_rules`, and this provider is what makes them bind at
//! enforcement time rather than render as decoration — the same relationship
//! [`super::department`] has to the department screens.
//!
//! Precedence sits *below* role and department: an organization grant is the
//! outermost envelope, and a department or role rule inside a customer refines
//! what the customer's plan already allows. Because the resolver is
//! deny-overrides, a customer admin can narrow their plan but never widen it.
//!
//! The value is resolved by lookup against `organization_members`, not read
//! from a JWT claim, so suspending a customer or moving a user between
//! organizations takes effect on the next request rather than lingering until
//! the token refreshes.

use std::collections::HashMap;
use std::sync::{Arc, LazyLock};
use std::time::{Duration, Instant};

use async_trait::async_trait;
use sqlx::PgPool;
use systemprompt::identifiers::UserId;
use systemprompt_security::authz::{RuleType, SubjectAttributeProvider, SubjectDimension};
use tokio::sync::RwLock;

const ORGANIZATION_SLUG: &str = "organization";

// Why: below core's `ROLE` (200) and `DEPARTMENT` (100) — the broadest scope
// yields to every narrower one.
const ORGANIZATION_PRECEDENCE: u16 = 300;

const ORGANIZATION_TTL: Duration = Duration::from_secs(60);

type OrganizationCache = HashMap<String, (Vec<String>, Instant)>;

static ORGANIZATION_CACHE: LazyLock<RwLock<OrganizationCache>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

#[must_use]
pub fn organization_rule_type() -> RuleType {
    RuleType::extension(ORGANIZATION_SLUG)
        .unwrap_or_else(|e| unreachable!("`{ORGANIZATION_SLUG}` is a well-formed slug: {e}"))
}

#[must_use]
pub fn organization_dimension() -> SubjectDimension {
    SubjectDimension {
        rule_type: organization_rule_type(),
        label: "Organization",
        precedence: ORGANIZATION_PRECEDENCE,
    }
}

#[derive(Debug)]
pub struct OrganizationAttributeProvider {
    pool: Arc<PgPool>,
}

impl OrganizationAttributeProvider {
    #[must_use]
    pub const fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    async fn cached(user_id: &UserId) -> Option<Vec<String>> {
        let cache = ORGANIZATION_CACHE.read().await;
        cache
            .get(user_id.as_str())
            .filter(|(_, at)| at.elapsed() < ORGANIZATION_TTL)
            .map(|(values, _)| values.clone())
    }

    async fn store(user_id: &UserId, values: &[String]) {
        let mut cache = ORGANIZATION_CACHE.write().await;
        cache.insert(
            user_id.as_str().to_owned(),
            (values.to_vec(), Instant::now()),
        );
    }
}

// Why: Drop a user's cached organization, so a seat move or a suspension binds
// on the next request rather than at the end of the TTL.
pub async fn invalidate(user_id: &UserId) {
    ORGANIZATION_CACHE.write().await.remove(user_id.as_str());
}

#[async_trait]
impl SubjectAttributeProvider for OrganizationAttributeProvider {
    fn dimension(&self) -> SubjectDimension {
        organization_dimension()
    }

    // Why: A user belongs to exactly one organization, so this yields zero or one
    // value — zero when the org is suspended or cancelled, which is what
    // makes suspension revoke every plan grant at once without touching a
    // single rule row.
    //
    // Fails soft for the same reason the department provider does: a lookup
    // error means "no organization", making organization rules unmatchable
    // and handing the decision to the narrower bands, rather than turning a
    // transient database blip into a site-wide outage.
    async fn values_for(&self, user_id: &UserId) -> Vec<String> {
        if let Some(values) = Self::cached(user_id).await {
            return values;
        }
        let looked_up = sqlx::query_scalar!(
            r#"
            SELECT o.slug
            FROM organization_members m
            JOIN organizations o ON o.id = m.org_id
            WHERE m.user_id = $1 AND o.status = 'active'
            "#,
            user_id.as_str()
        )
        .fetch_optional(self.pool.as_ref())
        .await;

        let values = match looked_up {
            Ok(row) => row.map_or_else(Vec::new, |slug| vec![slug]),
            Err(e) => {
                tracing::warn!(
                    error = %e, user_id = %user_id,
                    "organization lookup failed; resolving with no organization attribute",
                );
                Vec::new()
            },
        };
        Self::store(user_id, &values).await;
        values
    }
}
