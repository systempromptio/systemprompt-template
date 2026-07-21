//! The `department` subject dimension.
//!
//! Core's resolver ships two subject dimensions, `user` and `role`. Department
//! is ours: `access_control_rules` rows with `rule_type = 'department'` are
//! written by the access matrix and the department screens, and this provider
//! is what makes them bind at enforcement time rather than render as decoration.
//!
//! The value is resolved by lookup against `user_profile_ext`, not read from a
//! JWT claim, so moving a user between departments or revoking their profile
//! takes effect on the next request instead of lingering until the token
//! refreshes. A short TTL cache keeps that from costing a query per
//! enforcement decision.

use std::collections::HashMap;
use std::sync::{Arc, LazyLock};
use std::time::{Duration, Instant};

use async_trait::async_trait;
use sqlx::PgPool;
use systemprompt::identifiers::UserId;
use systemprompt_security::authz::{RuleType, SubjectAttributeProvider, SubjectDimension};
use tokio::sync::RwLock;

/// Slug bound to `access_control_rules.rule_type`.
const DEPARTMENT_SLUG: &str = "department";

/// Between core's `USER` (0) and `ROLE` (200): a department rule outranks a
/// role rule and yields to a rule naming the user directly. That is the
/// precedence the access matrix has always displayed.
const DEPARTMENT_PRECEDENCE: u16 = 100;

/// Same shape as the marketplace-parent cache in the authz webhook: a short
/// TTL that bounds staleness after a department change without turning every
/// decision into a query.
const DEPARTMENT_TTL: Duration = Duration::from_secs(60);

/// User id to (values, fetched-at). Values is a `Vec` because the provider
/// contract is multi-valued, even though a user has at most one department.
type DepartmentCache = HashMap<String, (Vec<String>, Instant)>;

static DEPARTMENT_CACHE: LazyLock<RwLock<DepartmentCache>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

/// The slug this dimension owns, as core's open rule-type vocabulary sees it.
#[must_use]
pub fn department_rule_type() -> RuleType {
    RuleType::extension(DEPARTMENT_SLUG)
        .unwrap_or_else(|e| unreachable!("`{DEPARTMENT_SLUG}` is a well-formed slug: {e}"))
}

/// The dimension's descriptor, also used by the access matrix to label the
/// layer that decided a cell.
#[must_use]
pub fn department_dimension() -> SubjectDimension {
    SubjectDimension {
        rule_type: department_rule_type(),
        label: "Department",
        precedence: DEPARTMENT_PRECEDENCE,
    }
}

#[derive(Debug)]
pub struct DepartmentAttributeProvider {
    pool: Arc<PgPool>,
}

impl DepartmentAttributeProvider {
    #[must_use]
    pub const fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    async fn cached(user_id: &UserId) -> Option<Vec<String>> {
        let cache = DEPARTMENT_CACHE.read().await;
        cache
            .get(user_id.as_str())
            .filter(|(_, at)| at.elapsed() < DEPARTMENT_TTL)
            .map(|(values, _)| values.clone())
    }

    async fn store(user_id: &UserId, values: &[String]) {
        let mut cache = DEPARTMENT_CACHE.write().await;
        cache.insert(
            user_id.as_str().to_owned(),
            (values.to_vec(), Instant::now()),
        );
    }
}

#[async_trait]
impl SubjectAttributeProvider for DepartmentAttributeProvider {
    fn dimension(&self) -> SubjectDimension {
        department_dimension()
    }

    /// A user has at most one department, so this yields zero or one value.
    ///
    /// Fails soft: a lookup error means "no department", which makes every
    /// department rule unmatchable for this request and hands the decision to
    /// the role band. Denying instead would turn a transient database blip
    /// into a site-wide outage, and the resolver's own default already closes
    /// the unmatched case.
    async fn values_for(&self, user_id: &UserId) -> Vec<String> {
        if let Some(values) = Self::cached(user_id).await {
            return values;
        }
        let looked_up = sqlx::query_scalar!(
            r#"SELECT department FROM user_profile_ext WHERE user_id = $1"#,
            user_id.as_str()
        )
        .fetch_optional(self.pool.as_ref())
        .await;

        let values = match looked_up {
            Ok(row) => row
                .map(|d| d.trim().to_owned())
                .filter(|d| !d.is_empty())
                .map_or_else(Vec::new, |d| vec![d]),
            Err(e) => {
                tracing::warn!(
                    error = %e, user_id = %user_id,
                    "department lookup failed; resolving with no department attribute",
                );
                Vec::new()
            },
        };
        Self::store(user_id, &values).await;
        values
    }
}
