//! The `project` subject dimension.
//!
//! A user holds every project id `project_members` records for them, written
//! by the directory at sign-in from the assertion's AD groups or by an
//! operator in the dashboard. Rules at `rule_type = 'project'` confine an
//! entity to a named piece of work; a user outside it holds no matching value,
//! matches no rule, and the resolver's default closes the gated entity.
//!
//! There is no derived fallback here, unlike `group`: work attribution is
//! optional, and a user on no project simply holds no value at this dimension.

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

const PROJECT_SLUG: &str = "project";

// Why: the narrowest of the tenant dimensions — a project rule is the most
// specific statement an operator can make about who a user is, so it out-ranks
// their group (150) and every band above it.
const PROJECT_PRECEDENCE: u16 = 140;

const PROJECT_TTL: Duration = Duration::from_secs(60);

type ProjectCache = HashMap<String, (Vec<String>, Instant)>;

static PROJECT_CACHE: LazyLock<RwLock<ProjectCache>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

#[must_use]
pub fn project_rule_type() -> RuleType {
    RuleType::extension(PROJECT_SLUG)
        .unwrap_or_else(|e| unreachable!("`{PROJECT_SLUG}` is a well-formed slug: {e}"))
}

#[must_use]
pub fn project_dimension() -> SubjectDimension {
    SubjectDimension {
        rule_type: project_rule_type(),
        label: "Project",
        precedence: PROJECT_PRECEDENCE,
    }
}

// Why: Drop the cached values for one user, so a sign-in or a membership edit
// takes effect on the next request instead of after the TTL.
pub async fn invalidate(user_id: &UserId) {
    PROJECT_CACHE.write().await.remove(user_id.as_str());
}

#[derive(Debug)]
pub struct ProjectAttributeProvider {
    pool: Arc<PgPool>,
}

impl ProjectAttributeProvider {
    #[must_use]
    pub const fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    async fn cached(user_id: &UserId) -> Option<Vec<String>> {
        let cache = PROJECT_CACHE.read().await;
        cache
            .get(user_id.as_str())
            .filter(|(_, at)| at.elapsed() < PROJECT_TTL)
            .map(|(values, _)| values.clone())
    }

    async fn store(user_id: &UserId, values: &[String]) {
        let mut cache = PROJECT_CACHE.write().await;
        cache.insert(
            user_id.as_str().to_owned(),
            (values.to_vec(), Instant::now()),
        );
    }
}

#[async_trait]
impl SubjectAttributeProvider for ProjectAttributeProvider {
    fn dimension(&self) -> SubjectDimension {
        project_dimension()
    }

    async fn values_for(&self, user_id: &UserId) -> Result<Vec<String>, AuthzError> {
        if let Some(values) = Self::cached(user_id).await {
            return Ok(values);
        }
        let values = crate::repositories::projects::members::list_project_ids_for_user(
            self.pool.as_ref(),
            user_id,
        )
        .await?;
        Self::store(user_id, &values).await;
        Ok(values)
    }
}
