use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use chrono::Utc;
use sqlx::PgPool;
use systemprompt::identifiers::UserId;
use tokio::sync::RwLock;

use super::super::tier_limits::{TierLimits, UsageSnapshot};
use super::usage::fetch_usage_from_db;

struct CachedTierContext {
    limits: TierLimits,
    plan_name: String,
    subscription_status: String,
    _period_end: Option<chrono::DateTime<Utc>>,
    cached_at: Instant,
}

struct CachedUsageSnapshot {
    snapshot: UsageSnapshot,
    cached_at: Instant,
}

const TIER_CACHE_TTL_SECS: u64 = 60;
const USAGE_CACHE_TTL_SECS: u64 = 10;

#[derive(Clone)]
pub struct TierEnforcementCache {
    tier_cache: Arc<RwLock<HashMap<String, CachedTierContext>>>,
    usage_cache: Arc<RwLock<HashMap<String, CachedUsageSnapshot>>>,
}

impl Default for TierEnforcementCache {
    fn default() -> Self {
        Self::new()
    }
}

impl TierEnforcementCache {
    #[must_use]
    pub fn new() -> Self {
        Self {
            tier_cache: Arc::new(RwLock::new(HashMap::new())),
            usage_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn invalidate_tier(&self, user_id: &str) {
        self.tier_cache.write().await.remove(user_id);
        self.usage_cache.write().await.remove(user_id);
    }

    pub async fn get_plan_name(&self, user_id: &str) -> String {
        let guard = self.tier_cache.read().await;
        guard
            .get(user_id)
            .map_or_else(|| "Free".to_string(), |c| c.plan_name.clone())
    }
}

pub async fn load_tier_context(
    cache: &TierEnforcementCache,
    pool: &PgPool,
    user_id: &UserId,
) -> (TierLimits, String) {
    {
        let guard = cache.tier_cache.read().await;
        if let Some(cached) = guard.get(user_id.as_str()) {
            if cached.cached_at.elapsed().as_secs() < TIER_CACHE_TTL_SECS {
                return (cached.limits.clone(), cached.subscription_status.clone());
            }
        }
    }

    let (limits, plan_name, status, period_end) = resolve_tier_for_user(pool, user_id).await;

    {
        let mut guard = cache.tier_cache.write().await;
        guard.insert(
            user_id.as_str().to_string(),
            CachedTierContext {
                limits: limits.clone(),
                plan_name,
                subscription_status: status.clone(),
                _period_end: period_end,
                cached_at: Instant::now(),
            },
        );
    }

    (limits, status)
}

async fn resolve_tier_for_user(
    pool: &PgPool,
    user_id: &UserId,
) -> (TierLimits, String, String, Option<chrono::DateTime<Utc>>) {
    if let Some((limits, plan_name)) = fetch_role_based_plan(pool, user_id).await {
        return (limits, plan_name, "active".to_string(), None);
    }

    let (limits, plan_name, status, period_end) = fetch_subscription_tier(pool, user_id).await;
    if status != "free" {
        return (limits, plan_name, status, period_end);
    }

    let (free_limits, free_name) = fetch_free_plan(pool).await;
    (free_limits, free_name, "free".to_string(), None)
}

async fn fetch_role_based_plan(pool: &PgPool, user_id: &UserId) -> Option<(TierLimits, String)> {
    #[derive(sqlx::FromRow)]
    struct RolePlanRow {
        plan_name: String,
        limits: Option<serde_json::Value>,
    }

    let row: Option<RolePlanRow> = sqlx::query_as(
        r"SELECT p.display_name AS plan_name, p.limits
          FROM marketplace.plans p
          WHERE p.role_name = ANY(
              SELECT UNNEST(roles) FROM users WHERE id = $1
          )
          AND p.is_active = true
          ORDER BY p.sort_order DESC
          LIMIT 1",
    )
    .bind(user_id.as_str())
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();

    row.map(|r| (parse_limits(r.limits), r.plan_name))
}

async fn fetch_free_plan(pool: &PgPool) -> (TierLimits, String) {
    #[derive(sqlx::FromRow)]
    struct FreePlanRow {
        plan_name: String,
        limits: Option<serde_json::Value>,
    }

    let row: Option<FreePlanRow> = sqlx::query_as(
        r"SELECT p.display_name AS plan_name, p.limits
          FROM marketplace.plans p
          WHERE p.name = 'free' AND p.is_active = true
          LIMIT 1",
    )
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();

    if let Some(r) = row {
        (parse_limits(r.limits), r.plan_name)
    } else {
        tracing::warn!("No 'free' plan found in DB, using hardcoded fallback");
        (TierLimits::free_default(), "Free".to_string())
    }
}

#[derive(sqlx::FromRow)]
struct TierRow {
    plan_name: String,
    status: String,
    current_period_end: Option<chrono::DateTime<Utc>>,
    limits: Option<serde_json::Value>,
}

async fn fetch_subscription_tier(
    pool: &PgPool,
    user_id: &UserId,
) -> (TierLimits, String, String, Option<chrono::DateTime<Utc>>) {
    let row: Option<TierRow> = sqlx::query_as(
        r"SELECT
            COALESCE(p.display_name, 'Free') AS plan_name,
            s.status,
            s.current_period_end,
            p.limits
           FROM marketplace.subscriptions s
           LEFT JOIN marketplace.plans p ON p.id = s.plan_id
           WHERE s.user_id = $1
           ORDER BY s.created_at DESC
           LIMIT 1",
    )
    .bind(user_id.as_str())
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();

    if let Some(r) = row {
        let status = r.status;
        let period_end = r.current_period_end;
        let plan_name = r.plan_name;

        let active = match status.as_str() {
            "active" | "past_due" => true,
            "cancelled" => period_end.is_some_and(|end| end > Utc::now()),
            _ => false,
        };

        if active {
            (parse_limits(r.limits), plan_name, status, period_end)
        } else {
            let (free_limits, free_name) = fetch_free_plan(pool).await;
            (free_limits, free_name, status, period_end)
        }
    } else {
        let (free_limits, free_name) = fetch_free_plan(pool).await;
        (free_limits, free_name, "free".to_string(), None)
    }
}

fn parse_limits(limits_json: Option<serde_json::Value>) -> TierLimits {
    limits_json
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_else(TierLimits::free_default)
}

pub async fn load_usage_snapshot(
    cache: &TierEnforcementCache,
    pool: &PgPool,
    user_id: &UserId,
) -> UsageSnapshot {
    {
        let guard = cache.usage_cache.read().await;
        if let Some(cached) = guard.get(user_id.as_str()) {
            if cached.cached_at.elapsed().as_secs() < USAGE_CACHE_TTL_SECS {
                return cached.snapshot.clone();
            }
        }
    }

    let snapshot = fetch_usage_from_db(pool, user_id).await;

    {
        let mut guard = cache.usage_cache.write().await;
        guard.insert(
            user_id.as_str().to_string(),
            CachedUsageSnapshot {
                snapshot: snapshot.clone(),
                cached_at: Instant::now(),
            },
        );
    }

    snapshot
}
