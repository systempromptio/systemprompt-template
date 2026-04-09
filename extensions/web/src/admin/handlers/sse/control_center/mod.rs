mod activity;

use std::convert::Infallible;
use std::sync::Arc;
use std::time::Duration;

use axum::{
    extract::{Extension, State},
    response::sse::{Event, KeepAlive, Sse},
};
use serde::Serialize;
use sqlx::PgPool;
use tokio_stream::Stream;

use crate::admin::repositories::control_center;
use crate::admin::types::control_center::RecentSession;
use crate::admin::types::UserContext;
use crate::admin::{event_hub::EventHub, gamification};

use super::{cc_analytics, cc_types};

use activity::ActivityWrapper;

#[derive(Serialize)]
struct ApmBlock {
    current: f32,
    peak: f32,
    avg: f32,
}

#[derive(Serialize)]
struct ConcurrencyBlock {
    current: i32,
    peak: i32,
    avg: f32,
}

#[derive(Serialize)]
struct ThroughputBlock {
    total_display: String,
    rate_display: String,
}

#[derive(Serialize)]
struct ApmMetricsBlock {
    apm: ApmBlock,
    concurrency: ConcurrencyBlock,
    throughput: ThroughputBlock,
    tool_diversity: i32,
    multitasking_score: f32,
}

#[derive(Serialize)]
struct TodayStats {
    active_now: usize,
    completed: i64,
    success_rate: i64,
    has_success_rate: bool,
    sessions: i64,
    prompts: i64,
    tool_calls: i64,
    errors: i64,
    total_content_display: String,
    gamification: cc_types::GamificationBlock,
    apm_metrics: ApmMetricsBlock,
}

type SessionGroup = crate::admin::handlers::ssr::ssr_control_center::types::SessionGroup;

pub async fn control_center_sse(
    Extension(user_ctx): Extension<UserContext>,
    Extension(event_hub): Extension<EventHub>,
    Extension(tier_cache): Extension<crate::admin::tier_enforcement::TierEnforcementCache>,
    State(pool): State<Arc<PgPool>>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let stream = async_stream::stream! {
        let user_id = user_ctx.user_id.clone();
        let mut rx = event_hub.subscribe(&user_id).await;
        let timeout_dur = Duration::from_secs(30);

        loop {
            tokio::select! {
                () = async { let _ = rx.recv().await; } => {}
                () = tokio::time::sleep(timeout_dur) => {}
            }
            while rx.try_recv().is_ok() {}

            let today_event = build_today_stats_event(&pool, &user_id).await;
            if let Ok(data) = serde_json::to_string(&today_event.today_stats) {
                yield Ok(Event::default().event("today-stats").data(data));
            }

            let usage_summary = crate::admin::tier_enforcement::get_usage_summary(
                &tier_cache, &pool, &user_id
            ).await;
            if let Ok(data) = serde_json::to_string(&usage_summary) {
                yield Ok(Event::default().event("usage-limits").data(data));
            }

            let activity_data = activity::build_activity_event(
                &pool, &user_id, &today_event.recent_sessions,
            ).await;
            let wrapper = ActivityWrapper { session_groups: activity_data.session_groups };
            if let Ok(data) = serde_json::to_string(&wrapper) {
                yield Ok(Event::default().event("activity").data(data));
            }

            let analytics_input = cc_analytics::AnalyticsInput {
                entity_links: &activity_data.entity_links,
                session_ratings: &activity_data.session_ratings,
                gam: today_event.gam.as_ref(),
                apm_live: &today_event.apm_live,
            };
            if let Some(data) = cc_analytics::build_analytics_event(
                &pool, &user_id, &analytics_input,
            ).await {
                yield Ok(Event::default().event("analytics").data(data));
            }
        }
    };

    Sse::new(stream).keep_alive(KeepAlive::default())
}

struct TodayStatsEvent {
    today_stats: TodayStats,
    recent_sessions: Vec<RecentSession>,
    gam: Option<crate::admin::types::UserGamificationProfile>,
    apm_live: crate::admin::repositories::apm_metrics::TodayApmLive,
}

async fn build_today_stats_event(
    pool: &PgPool,
    user_id: &systemprompt::identifiers::UserId,
) -> TodayStatsEvent {
    let (today_res, outcome, sessions_res, apm_live) = tokio::join!(
        control_center::fetch_today_stats(pool, user_id),
        control_center::fetch_today_outcome_stats(pool, user_id),
        control_center::fetch_recent_sessions(pool, user_id, 50),
        crate::admin::repositories::apm_metrics::fetch_today_apm_live(pool, user_id.as_str()),
    );

    let recent_sessions = sessions_res.unwrap_or_else(|_| Vec::new());
    let active_now = recent_sessions
        .iter()
        .filter(|s| s.ended_at.is_none() && s.status == "active")
        .count();

    let success_rate = if outcome.rated_count > 0 {
        outcome.positive_count.saturating_mul(100) / outcome.rated_count
    } else {
        0
    };

    let gam = gamification::queries::find_user_gamification(pool, user_id.as_str())
        .await
        .unwrap_or(None);
    let gam_block = cc_types::build_gamification_block(gam.as_ref());

    let today_stats = TodayStats {
        active_now,
        completed: outcome.completed_today,
        success_rate,
        has_success_rate: outcome.rated_count > 0,
        sessions: today_res.sessions_started,
        prompts: today_res.total_prompts,
        tool_calls: today_res.total_tool_calls,
        errors: today_res.total_errors,
        total_content_display: control_center::format_bytes(
            today_res.content_input_bytes + today_res.content_output_bytes,
        ),
        gamification: gam_block,
        apm_metrics: ApmMetricsBlock {
            apm: ApmBlock {
                current: apm_live.current_apm,
                peak: apm_live.peak_apm,
                avg: apm_live.avg_apm,
            },
            concurrency: ConcurrencyBlock {
                current: apm_live.current_concurrency,
                peak: apm_live.peak_concurrency,
                avg: apm_live.avg_concurrency,
            },
            throughput: ThroughputBlock {
                total_display: apm_live.total_throughput_display.clone(),
                rate_display: apm_live.throughput_rate_display.clone(),
            },
            tool_diversity: apm_live.tool_diversity,
            multitasking_score: apm_live.multitasking_score,
        },
    };
    TodayStatsEvent {
        today_stats,
        recent_sessions,
        gam,
        apm_live,
    }
}
