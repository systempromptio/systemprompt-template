//! `repositories::analytics::site` — the site dashboard's read models:
//! period-over-period KPIs, cost-by-model series, the latency split, the
//! client-reported session snapshots, and the per-user daily rollups.

use chrono::{Duration, Utc};
use systemprompt_web_admin::repositories::analytics::site::SiteScope;
use systemprompt_web_admin::repositories::analytics::site::kpis::get_site_kpis;
use systemprompt_web_admin::repositories::analytics::site::latency::{
    FAST_THRESHOLD_MS, get_latency_split,
};
use systemprompt_web_admin::repositories::analytics::site::model_series::list_model_cost_series;
use systemprompt_web_admin::repositories::analytics::site::series::SeriesBucket;
use systemprompt_web_admin::repositories::analytics::site::session_costs::{
    get_session_cost_stats, list_user_session_costs,
};
use systemprompt_web_admin::repositories::analytics::site::user_rollups::list_user_daily_rollups;
use systemprompt_web_admin::util::time_range::{TimeRange, TimeRangePreset};

use crate::fixtures::{RequestSpec, insert_request, insert_user, unclaimed_email, unique};
use crate::tempdb::TempDb;

fn window(hours_back: i64) -> TimeRange {
    let now = Utc::now();
    TimeRange {
        from: now - Duration::hours(hours_back),
        to: now + Duration::minutes(1),
        preset: TimeRangePreset::Custom,
    }
}

#[tokio::test]
async fn get_site_kpis_reports_the_previous_window_separately() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("kpi")).await;
    let scope = SiteScope {
        user_id: Some(user.clone()),
        ..SiteScope::default()
    };

    // Current window is the last hour; the previous window is the hour before.
    let now = Utc::now();
    let mut current = RequestSpec::completed(&unique("req"), &user);
    current.created_at = now - Duration::minutes(10);
    current.cost_microdollars = 500;
    insert_request(&db.pool, &current).await;

    let mut previous = RequestSpec::completed(&unique("req"), &user);
    previous.created_at = now - Duration::minutes(90);
    previous.cost_microdollars = 1500;
    insert_request(&db.pool, &previous).await;

    let range = TimeRange {
        from: now - Duration::hours(1),
        to: now + Duration::minutes(1),
        preset: TimeRangePreset::Custom,
    };
    let kpis = get_site_kpis(&db.pool, range, &scope)
        .await
        .expect("kpi query succeeds");

    assert_eq!(kpis.total_requests, 1, "only the current window's request");
    assert_eq!(kpis.total_cost_microdollars, 500);
    assert_eq!(kpis.prev_total_requests, 1, "the prior hour's request");
    assert_eq!(kpis.prev_total_cost_microdollars, 1500);
    db.cleanup().await;
}

#[tokio::test]
async fn get_site_kpis_scope_excludes_other_users() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let mine = insert_user(&db.pool, &unique("user"), &unclaimed_email("mine")).await;
    let theirs = insert_user(&db.pool, &unique("user"), &unclaimed_email("theirs")).await;
    insert_request(&db.pool, &RequestSpec::completed(&unique("req"), &mine)).await;
    insert_request(&db.pool, &RequestSpec::completed(&unique("req"), &theirs)).await;

    let scope = SiteScope {
        user_id: Some(mine.clone()),
        ..SiteScope::default()
    };
    let kpis = get_site_kpis(&db.pool, window(24), &scope)
        .await
        .expect("kpi query succeeds");
    assert_eq!(kpis.total_requests, 1);
    db.cleanup().await;
}

#[tokio::test]
async fn list_model_cost_series_folds_everything_past_the_sixth_model() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("models")).await;
    let scope = SiteScope {
        user_id: Some(user.clone()),
        ..SiteScope::default()
    };

    // Eight models with strictly descending cost: the top six keep their own
    // series, the last two must arrive folded into a single "Other".
    for (i, cost) in (0..8).map(|i| (i, 8_000 - i64::from(i) * 500)) {
        let mut spec = RequestSpec::completed(&unique("req"), &user);
        spec.model = Box::leak(format!("model-{i}").into_boxed_str());
        spec.cost_microdollars = cost;
        insert_request(&db.pool, &spec).await;
    }

    let rows = list_model_cost_series(&db.pool, window(24), &scope, SeriesBucket::Day)
        .await
        .expect("model cost series succeeds");

    let mut labels: Vec<String> = rows.iter().map(|r| r.model.clone()).collect();
    labels.sort();
    labels.dedup();
    assert_eq!(
        labels.len(),
        7,
        "six models plus the Other fold: {labels:?}"
    );
    assert!(labels.iter().any(|m| m == "Other"));

    let other_cost: i64 = rows
        .iter()
        .filter(|r| r.model == "Other")
        .map(|r| r.cost_microdollars)
        .sum();
    // The two cheapest models: 8000 - 6*500 = 5000 and 8000 - 7*500 = 4500.
    assert_eq!(other_cost, 9_500);
    db.cleanup().await;
}

#[tokio::test]
async fn get_latency_split_buckets_on_the_fixed_threshold() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("latency")).await;
    let scope = SiteScope {
        user_id: Some(user.clone()),
        ..SiteScope::default()
    };

    let mut fast = RequestSpec::completed(&unique("req"), &user);
    fast.latency_ms = i32::try_from(FAST_THRESHOLD_MS - 1).expect("threshold fits i32");
    insert_request(&db.pool, &fast).await;

    let mut slow = RequestSpec::completed(&unique("req"), &user);
    slow.latency_ms = i32::try_from(FAST_THRESHOLD_MS).expect("threshold fits i32");
    insert_request(&db.pool, &slow).await;

    let default_ms = i32::try_from(FAST_THRESHOLD_MS).expect("threshold fits i32");
    let split = get_latency_split(&db.pool, window(24), &scope, default_ms)
        .await
        .expect("latency split succeeds");

    assert_eq!(split.fast, 1, "4999ms is fast at the default threshold");
    assert_eq!(
        split.slow, 1,
        "5000ms is slow — the boundary is inclusive up"
    );
    assert!(split.p95_ms >= split.p50_ms);

    // The threshold is caller-configurable (REQ-029): at 2s both seeded
    // requests land on the slow side of the same query.
    let tight = get_latency_split(&db.pool, window(24), &scope, 2_000)
        .await
        .expect("latency split succeeds");
    assert_eq!(tight.fast, 0, "nothing beats a 2s SLO here");
    assert_eq!(tight.slow, 2);
    assert_eq!(tight.threshold_ms, 2_000);

    db.cleanup().await;
}

#[tokio::test]
async fn session_cost_snapshots_aggregate_and_list_per_user() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("snap")).await;
    let scope = SiteScope {
        user_id: Some(user.clone()),
        ..SiteScope::default()
    };

    for (i, cache_read) in [(0, 3_000i64), (1, 1_000)] {
        sqlx::query(
            "INSERT INTO session_cost_snapshots
                 (session_id, user_id, model, total_cost_microdollars, context_window_size,
                  input_tokens, output_tokens, cache_creation_input_tokens,
                  cache_read_input_tokens)
             VALUES ($1, $2, 'claude-opus-5', 1000, $3, 1000, 200, 500, $4)",
        )
        .bind(format!("snap-session-{i}"))
        .bind(user.as_str())
        .bind(100_000i64 + i64::from(i) * 50_000)
        .bind(cache_read)
        .execute(&*db.pool)
        .await
        .expect("insert snapshot");
    }

    let stats = get_session_cost_stats(&db.pool, window(24), &scope)
        .await
        .expect("session cost stats succeed");
    assert_eq!(stats.sessions, 2);
    assert_eq!(stats.cache_read_tokens, 4_000);
    // cache_read / (cache_read + input) = 4000 / (4000 + 2000).
    assert!(
        (stats.cache_hit_pct - 66.6).abs() < 0.2,
        "{}",
        stats.cache_hit_pct
    );
    assert_eq!(stats.max_context_window, 150_000);

    let rows = list_user_session_costs(&db.pool, &user, 1)
        .await
        .expect("session list succeeds");
    assert_eq!(rows.len(), 1, "limit is honoured");
    db.cleanup().await;
}

#[tokio::test]
async fn list_user_daily_rollups_reads_only_the_requested_range() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("rollup")).await;

    for days_back in [0i64, 40] {
        sqlx::query(
            "INSERT INTO admin_usage_daily_rollups
                 (user_id, date, sessions_count, prompts, tool_uses, errors,
                  loc_added_ai, loc_removed_ai, commits_count, commit_insertions,
                  commit_deletions, ai_requests_count, input_tokens, output_tokens,
                  cost_microdollars)
             VALUES ($1, (NOW() - ($2 || ' days')::interval)::date,
                     1, 2, 3, 0, 40, 5, 1, 60, 8, 4, 400, 100, 2000)",
        )
        .bind(user.as_str())
        .bind(days_back.to_string())
        .execute(&*db.pool)
        .await
        .expect("insert rollup");
    }

    let rows = list_user_daily_rollups(&db.pool, &user, window(24 * 7))
        .await
        .expect("rollup read succeeds");
    assert_eq!(rows.len(), 1, "the 40-day-old row is outside the window");
    assert_eq!(rows[0].loc_added_ai, 40);
    assert_eq!(rows[0].commits_count, 1);
    db.cleanup().await;
}
