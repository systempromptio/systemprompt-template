use crate::repositories::{self, conversation_analytics};
use sqlx::PgPool;
use systemprompt::identifiers::UserId;

pub(super) async fn fetch_hooks_and_plugins(
    pool: &PgPool,
    user_id: &UserId,
) -> (Vec<crate::types::UserHook>, Vec<crate::types::UserPlugin>) {
    let hooks = repositories::user_hooks::list_user_hooks(pool, user_id)
        .await
        .unwrap_or_else(|e| {
            tracing::error!(error = %e, "Failed to list user hooks");
            vec![]
        });
    let user_plugins = repositories::list_user_plugins(pool, user_id)
        .await
        .unwrap_or_else(|e| {
            tracing::error!(error = %e, "Failed to list user plugins");
            vec![]
        });
    (hooks, user_plugins)
}

pub(super) async fn fetch_hook_analytics(
    pool: &PgPool,
    user_id: &UserId,
    range: &str,
) -> (
    Vec<crate::types::HookEventTypeStat>,
    Vec<crate::types::HookTimeSeriesBucket>,
    crate::types::HookSummaryStats,
    Vec<crate::types::conversation_analytics::HookSessionQuality>,
) {
    let (event_breakdown, timeseries, summary, hook_quality) = tokio::join!(
        repositories::user_hooks::get_hook_event_breakdown(pool, user_id),
        repositories::user_hooks::get_hook_timeseries(pool, user_id, range),
        repositories::user_hooks::get_hook_summary_stats(pool, user_id, range),
        async {
            conversation_analytics::fetch_hook_session_quality(pool, user_id)
                .await
                .unwrap_or_else(|e| {
                    tracing::error!(error = %e, "Failed to fetch hook session quality");
                    vec![]
                })
        },
    );
    let event_breakdown = event_breakdown.unwrap_or_else(|e| {
        tracing::error!(error = %e, "Failed to fetch hook event breakdown");
        vec![]
    });
    let timeseries = timeseries.unwrap_or_else(|e| {
        tracing::error!(error = %e, "Failed to fetch hook timeseries");
        vec![]
    });
    (event_breakdown, timeseries, summary, hook_quality)
}

pub(super) fn compute_avg_session_quality(
    hook_quality: &[crate::types::conversation_analytics::HookSessionQuality],
) -> f64 {
    if hook_quality.is_empty() {
        return 0.0;
    }
    let count = hook_quality.len();
    hook_quality.iter().map(|q| q.avg_quality).sum::<f64>()
        / f64::from(u32::try_from(count).unwrap_or(1))
}
