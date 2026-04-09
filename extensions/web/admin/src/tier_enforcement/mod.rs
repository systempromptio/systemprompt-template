mod cache;
mod usage;

use std::sync::Arc;

use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use super::tier_limits::{
    Feature, LimitCheck, LimitCheckResult, TierLimits, UsageSnapshot, UsageSummary,
};

pub use cache::TierEnforcementCache;
use cache::{load_tier_context, load_usage_snapshot};

pub async fn check_limit(
    cache: &TierEnforcementCache,
    pool: &PgPool,
    user_id: &UserId,
    check: LimitCheck,
) -> LimitCheckResult {
    let (limits, _status) = load_tier_context(cache, pool, user_id).await;

    match check {
        LimitCheck::FeatureAccess(feature) => check_feature(&limits, feature),
        LimitCheck::IngestEvent
        | LimitCheck::IngestContentBytes(_)
        | LimitCheck::IngestSession
        | LimitCheck::CreateSkill
        | LimitCheck::CreateAgent
        | LimitCheck::CreatePlugin
        | LimitCheck::CreateMcpServer
        | LimitCheck::CreateHook => {
            let usage = load_usage_snapshot(cache, pool, user_id).await;
            check_usage(&limits, &usage, check)
        }
    }
}

pub async fn get_usage_summary(
    cache: &TierEnforcementCache,
    pool: &PgPool,
    user_id: &UserId,
) -> UsageSummary {
    let (limits, _status) = load_tier_context(cache, pool, user_id).await;
    let usage = load_usage_snapshot(cache, pool, user_id).await;
    let plan_name = cache.get_plan_name(user_id.as_str()).await;
    UsageSummary::build(
        Arc::unwrap_or_clone(limits),
        Arc::unwrap_or_clone(usage),
        plan_name,
    )
}

fn check_feature(limits: &TierLimits, feature: Feature) -> LimitCheckResult {
    let allowed = match feature {
        Feature::AiSessionAnalysis => limits.features.ai.session_analysis,
        Feature::AiDailySummaries => limits.features.ai.daily_summaries,
        Feature::ApmMetrics => limits.features.apm_metrics,
        Feature::ExportZip => limits.features.export_zip,
    };
    if allowed {
        LimitCheckResult::allowed()
    } else {
        let name = match feature {
            Feature::AiSessionAnalysis => "AI Session Analysis",
            Feature::AiDailySummaries => "AI Daily Summaries",
            Feature::ApmMetrics => "APM Metrics",
            Feature::ExportZip => "Export ZIP",
        };
        LimitCheckResult::feature_denied(name)
    }
}

fn check_usage(limits: &TierLimits, usage: &UsageSnapshot, check: LimitCheck) -> LimitCheckResult {
    match check {
        LimitCheck::IngestEvent => {
            LimitCheckResult::with_usage(limits.ingestion.events, usage.events_today)
        }
        LimitCheck::IngestContentBytes(additional) => LimitCheckResult::with_usage(
            limits.ingestion.content_bytes,
            usage.content_bytes_today + additional,
        ),
        LimitCheck::IngestSession => {
            LimitCheckResult::with_usage(limits.ingestion.sessions, usage.sessions_today)
        }
        LimitCheck::CreateSkill => {
            LimitCheckResult::with_usage(limits.entities.skills, usage.skills_count)
        }
        LimitCheck::CreateAgent => {
            LimitCheckResult::with_usage(limits.entities.agents, usage.agents_count)
        }
        LimitCheck::CreatePlugin => {
            LimitCheckResult::with_usage(limits.entities.plugins, usage.plugins_count)
        }
        LimitCheck::CreateMcpServer => {
            LimitCheckResult::with_usage(limits.entities.mcp_servers, usage.mcp_servers_count)
        }
        LimitCheck::CreateHook => {
            LimitCheckResult::with_usage(limits.entities.hooks, usage.hooks_count)
        }
        LimitCheck::FeatureAccess(_) => unreachable!(),
    }
}
