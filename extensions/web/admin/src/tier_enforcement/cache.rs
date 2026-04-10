use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use std::time::Instant;

use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use systemprompt::identifiers::UserId;
use tokio::sync::RwLock;

use super::super::tier_limits::{TierLimits, UsageSnapshot};
use super::usage::fetch_usage_from_db;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SubscriptionStatus {
    Active,
    PastDue,
    Cancelled,
    Free,
}

impl SubscriptionStatus {
    fn from_db(s: &str) -> Self {
        match s {
            "active" => Self::Active,
            "past_due" => Self::PastDue,
            "cancelled" => Self::Cancelled,
            _ => Self::Free,
        }
    }
}

impl fmt::Display for SubscriptionStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Active => f.write_str("active"),
            Self::PastDue => f.write_str("past_due"),
            Self::Cancelled => f.write_str("cancelled"),
            Self::Free => f.write_str("free"),
        }
    }
}

#[derive(Debug)]
struct CachedTierContext {
    limits: Arc<TierLimits>,
    plan_name: Arc<str>,
    subscription_status: SubscriptionStatus,
    _period_end: Option<chrono::DateTime<Utc>>,
    cached_at: Instant,
}

#[derive(Debug)]
struct CachedUsageSnapshot {
    snapshot: Arc<UsageSnapshot>,
    cached_at: Instant,
}

const TIER_CACHE_TTL_SECS: u64 = 60;
const USAGE_CACHE_TTL_SECS: u64 = 10;

#[derive(Clone, Debug)]
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

    pub async fn get_plan_name(&self, user_id: &str) -> Arc<str> {
        let guard = self.tier_cache.read().await;
        guard
            .get(user_id)
            .map_or_else(|| Arc::from("Free"), |c| Arc::clone(&c.plan_name))
    }
}

pub async fn load_tier_context(
    cache: &TierEnforcementCache,
    pool: &PgPool,
    user_id: &UserId,
) -> (Arc<TierLimits>, SubscriptionStatus) {
    {
        let guard = cache.tier_cache.read().await;
        if let Some(cached) = guard.get(user_id.as_str()) {
            if cached.cached_at.elapsed().as_secs() < TIER_CACHE_TTL_SECS {
                return (
                    Arc::clone(&cached.limits),
                    cached.subscription_status,
                );
            }
        }
    }

    let (limits, plan_name, status, period_end) = resolve_tier_for_user(pool, user_id).await;
    let limits = Arc::new(limits);

    {
        let mut guard = cache.tier_cache.write().await;
        guard.insert(
            user_id.as_str().to_string(),
            CachedTierContext {
                limits: Arc::clone(&limits),
                plan_name: Arc::from(plan_name),
                subscription_status: status,
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
) -> (TierLimits, String, SubscriptionStatus, Option<chrono::DateTime<Utc>>) {
    if let Some((limits, plan_name)) = fetch_role_based_plan(pool, user_id).await {
        return (limits, plan_name, SubscriptionStatus::Active, None);
    }

    let (limits, plan_name, status, period_end) = fetch_subscription_tier(pool, user_id).await;
    if status != SubscriptionStatus::Free {
        return (limits, plan_name, status, period_end);
    }

    let (free_limits, free_name) = fetch_free_plan(pool).await;
    (free_limits, free_name, SubscriptionStatus::Free, None)
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
    .map_err(|e| {
        tracing::warn!(error = %e, user_id = %user_id.as_str(), "Failed to fetch role-based plan");
    })
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
    .map_err(|e| {
        tracing::warn!(error = %e, "Failed to fetch free plan from DB");
    })
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
) -> (TierLimits, String, SubscriptionStatus, Option<chrono::DateTime<Utc>>) {
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
    .map_err(|e| {
        tracing::warn!(error = %e, user_id = %user_id.as_str(), "Failed to fetch subscription tier");
    })
    .ok()
    .flatten();

    if let Some(r) = row {
        let status = SubscriptionStatus::from_db(&r.status);
        let period_end = r.current_period_end;
        let plan_name = r.plan_name;

        let active = match status {
            SubscriptionStatus::Active | SubscriptionStatus::PastDue => true,
            SubscriptionStatus::Cancelled => period_end.is_some_and(|end| end > Utc::now()),
            SubscriptionStatus::Free => false,
        };

        if active {
            (parse_limits(r.limits), plan_name, status, period_end)
        } else {
            let (free_limits, free_name) = fetch_free_plan(pool).await;
            (free_limits, free_name, status, period_end)
        }
    } else {
        let (free_limits, free_name) = fetch_free_plan(pool).await;
        (free_limits, free_name, SubscriptionStatus::Free, None)
    }
}

fn parse_limits(limits_json: Option<serde_json::Value>) -> TierLimits {
    limits_json
        .and_then(|v| {
            serde_json::from_value(v).map_err(|e| {
                tracing::warn!(error = %e, "Failed to parse tier limits JSON, falling back to free defaults");
            }).ok()
        })
        .unwrap_or_else(TierLimits::free_default)
}

pub async fn load_usage_snapshot(
    cache: &TierEnforcementCache,
    pool: &PgPool,
    user_id: &UserId,
) -> Result<Arc<UsageSnapshot>, sqlx::Error> {
    {
        let guard = cache.usage_cache.read().await;
        if let Some(cached) = guard.get(user_id.as_str()) {
            if cached.cached_at.elapsed().as_secs() < USAGE_CACHE_TTL_SECS {
                return Ok(Arc::clone(&cached.snapshot));
            }
        }
    }

    let snapshot = Arc::new(fetch_usage_from_db(pool, user_id).await?);

    {
        let mut guard = cache.usage_cache.write().await;
        guard.insert(
            user_id.as_str().to_string(),
            CachedUsageSnapshot {
                snapshot: Arc::clone(&snapshot),
                cached_at: Instant::now(),
            },
        );
    }

    Ok(snapshot)
}
