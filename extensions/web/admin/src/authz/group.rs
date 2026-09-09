//! The `group` subject dimension.
//!
//! A user holds every group id the `user_groups` view places them in —
//! `group_members` rows written by the directory at sign-in or by an operator
//! in the dashboard, plus the derived `unassigned` membership for anyone
//! carrying no row at all. Rules at `rule_type = 'group'` confine an entity to
//! members of a named group; a user outside it holds no matching value,
//! matches no rule, and the resolver's default closes the gated entity.
//!
//! Resolved by lookup rather than from the token, so a membership change binds
//! on the next request. Sign-in and dashboard membership edits are explicit
//! events, so they call [`invalidate`] and do not wait out the TTL.

use std::collections::HashMap;
use std::sync::{Arc, LazyLock};
use std::time::{Duration, Instant};

use async_trait::async_trait;
use sqlx::PgPool;
use systemprompt::identifiers::UserId;
use systemprompt_security::authz::{
    AuthzError, RuleType, SubjectAttributeProvider, SubjectDimension,
};
use tokio::sync::RwLock;

const GROUP_SLUG: &str = "group";

// Why: below core's `ROLE` (200) so a group rule out-ranks the role band that
// grants a marketplace to every user, and above `project` (140) so a project
// rule stays the narrower of the two.
const GROUP_PRECEDENCE: u16 = 150;

const GROUP_TTL: Duration = Duration::from_secs(60);

type GroupCache = HashMap<String, (Vec<String>, Instant)>;

static GROUP_CACHE: LazyLock<RwLock<GroupCache>> = LazyLock::new(|| RwLock::new(HashMap::new()));

#[must_use]
pub fn group_rule_type() -> RuleType {
    RuleType::extension(GROUP_SLUG)
        .unwrap_or_else(|e| unreachable!("`{GROUP_SLUG}` is a well-formed slug: {e}"))
}

#[must_use]
pub fn group_dimension() -> SubjectDimension {
    SubjectDimension {
        rule_type: group_rule_type(),
        label: "Group",
        precedence: GROUP_PRECEDENCE,
    }
}

// Why: Drop the cached values for one user, so a sign-in or a membership edit
// takes effect on the next request instead of after the TTL.
pub async fn invalidate(user_id: &UserId) {
    GROUP_CACHE.write().await.remove(user_id.as_str());
}

#[derive(Debug)]
pub struct GroupAttributeProvider {
    pool: Arc<PgPool>,
}

impl GroupAttributeProvider {
    #[must_use]
    pub const fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    async fn cached(user_id: &UserId) -> Option<Vec<String>> {
        let cache = GROUP_CACHE.read().await;
        cache
            .get(user_id.as_str())
            .filter(|(_, at)| at.elapsed() < GROUP_TTL)
            .map(|(values, _)| values.clone())
    }

    async fn store(user_id: &UserId, values: &[String]) {
        let mut cache = GROUP_CACHE.write().await;
        cache.insert(
            user_id.as_str().to_owned(),
            (values.to_vec(), Instant::now()),
        );
    }
}

#[async_trait]
impl SubjectAttributeProvider for GroupAttributeProvider {
    fn dimension(&self) -> SubjectDimension {
        group_dimension()
    }

    async fn values_for(&self, user_id: &UserId) -> Result<Vec<String>, AuthzError> {
        if let Some(values) = Self::cached(user_id).await {
            return Ok(values);
        }
        let values = crate::repositories::groups::members::list_group_ids_for_user(
            self.pool.as_ref(),
            user_id,
        )
        .await?;
        Self::store(user_id, &values).await;
        Ok(values)
    }
}
