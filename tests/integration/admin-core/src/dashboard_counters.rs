//! `repositories::dashboard` counters, leaderboards, and the event feed.
//!
//! `user_activity` and `mcp_tool_executions` carry no seed rows, so counters
//! over them are asserted absolutely; the `plugin_usage_events` feed is seeded
//! by migration 011, so those tests assert on their own rows.

use systemprompt_web_admin::repositories::dashboard::aggregates::{
    get_active_users_24h, get_activity_stats, list_usage_timeseries,
};
use systemprompt_web_admin::repositories::dashboard::queries::{
    list_hourly_activity, list_popular_skills, list_recent_mcp_errors, list_tool_success_rates,
    list_top_users,
};
use systemprompt_web_admin::repositories::dashboard::{get_dashboard_data, list_events};
use systemprompt_web_admin::types::EventsQuery;

use crate::dashboard_sessions::insert_mcp_execution;
use crate::fixtures::{
    EventSpec, insert_activity, insert_event, insert_user, insert_user_full, unclaimed_email,
    unique,
};
use crate::tempdb::TempDb;

#[tokio::test]
async fn get_activity_stats_is_zero_before_any_activity() {
    let Some(db) = TempDb::create().await else {
        return;
    };

    let stats = get_activity_stats(&db.pool).await.expect("query succeeds");

    assert_eq!(stats.events_today, 0);
    assert_eq!(stats.total_logins, 0);
    assert_eq!(stats.total_edits, 0);
    assert_eq!(stats.mcp_tool_calls, 0);
    db.cleanup().await;
}

#[tokio::test]
async fn get_activity_stats_counts_logins_and_edits_by_category() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("stats")).await;
    insert_activity(&db.pool, &unique("act"), &user, "login", "signed_in").await;
    insert_activity(&db.pool, &unique("act"), &user, "login", "signed_in").await;
    insert_activity(
        &db.pool,
        &unique("act"),
        &user,
        "marketplace_edit",
        "updated",
    )
    .await;

    let stats = get_activity_stats(&db.pool).await.expect("query succeeds");

    assert_eq!(stats.total_logins, 2);
    assert_eq!(stats.total_edits, 1);
    assert_eq!(stats.events_today, 3);
    db.cleanup().await;
}

#[tokio::test]
async fn get_activity_stats_counts_mcp_calls_and_failures() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("mcp")).await;
    insert_mcp_execution(&db.pool, user.as_str(), "search", "success").await;
    insert_mcp_execution(&db.pool, user.as_str(), "search", "failed").await;

    let stats = get_activity_stats(&db.pool).await.expect("query succeeds");

    assert_eq!(stats.mcp_tool_calls, 2);
    assert_eq!(stats.mcp_errors, 1);
    db.cleanup().await;
}

#[tokio::test]
async fn get_active_users_24h_unions_activity_and_mcp_traffic() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let by_activity = insert_user(&db.pool, &unique("user"), &unclaimed_email("act")).await;
    let by_mcp = insert_user(&db.pool, &unique("user"), &unclaimed_email("mcp")).await;
    insert_activity(&db.pool, &unique("act"), &by_activity, "login", "signed_in").await;
    insert_mcp_execution(&db.pool, by_mcp.as_str(), "search", "success").await;

    let active = get_active_users_24h(&db.pool)
        .await
        .expect("query succeeds");

    assert_eq!(active, 2, "a user is active if either table saw them");
    db.cleanup().await;
}

#[tokio::test]
async fn get_active_users_24h_excludes_anonymous_identities() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let robot = insert_user_full(
        &db.pool,
        &unique("anon"),
        &unclaimed_email("robot"),
        None,
        &["anonymous".to_owned()],
    )
    .await;
    insert_activity(&db.pool, &unique("act"), &robot, "login", "signed_in").await;

    let active = get_active_users_24h(&db.pool)
        .await
        .expect("query succeeds");

    assert_eq!(active, 0);
    db.cleanup().await;
}

#[tokio::test]
async fn list_usage_timeseries_returns_ordered_buckets() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("series")).await;
    insert_activity(&db.pool, &unique("act"), &user, "login", "signed_in").await;

    let series = list_usage_timeseries(&db.pool, "24 hours", "1 hour")
        .await
        .expect("query succeeds");

    assert!(
        !series.is_empty(),
        "the bucket spine is generated, not derived from rows"
    );
    assert!(series.windows(2).all(|w| w[0].bucket < w[1].bucket));
    let logins: i64 = series.iter().map(|b| b.sessions).sum();
    assert_eq!(logins, 1, "the login lands in exactly one bucket");
    db.cleanup().await;
}

#[tokio::test]
async fn list_top_users_ranks_by_edits_and_mcp_calls() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let busy = insert_user(&db.pool, &unique("user"), &unclaimed_email("busy")).await;
    let idle = insert_user(&db.pool, &unique("user"), &unclaimed_email("idle")).await;
    insert_activity(
        &db.pool,
        &unique("act"),
        &busy,
        "marketplace_edit",
        "updated",
    )
    .await;
    insert_activity(&db.pool, &unique("act"), &busy, "login", "signed_in").await;
    insert_mcp_execution(&db.pool, busy.as_str(), "search", "success").await;
    insert_activity(&db.pool, &unique("act"), &idle, "login", "signed_in").await;

    let top = list_top_users(&db.pool).await.expect("query succeeds");

    assert_eq!(top[0].user_id, busy);
    assert_eq!(top[0].edits, 1);
    assert_eq!(top[0].logins, 1);
    assert_eq!(top[0].mcp_calls, 1);
    assert!(top.iter().any(|u| u.user_id == idle));
    db.cleanup().await;
}

#[tokio::test]
async fn list_top_users_needs_an_activity_row_to_list_anyone() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("silent")).await;
    insert_mcp_execution(&db.pool, user.as_str(), "search", "success").await;

    let top = list_top_users(&db.pool).await.expect("query succeeds");

    assert!(
        !top.iter().any(|u| u.user_id == user),
        "the leaderboard is driven from user_activity; MCP traffic alone does not enter it"
    );
    db.cleanup().await;
}

#[tokio::test]
async fn list_popular_skills_counts_mcp_tool_calls() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("skills")).await;
    let tool = unique("tool");
    for _ in 0..3 {
        insert_mcp_execution(&db.pool, user.as_str(), &tool, "success").await;
    }

    let skills = list_popular_skills(&db.pool).await.expect("query succeeds");

    let row = skills
        .iter()
        .find(|s| s.tool_name == tool)
        .expect("the tool appears");
    assert_eq!(row.count, 3);
    db.cleanup().await;
}

#[tokio::test]
async fn list_hourly_activity_buckets_the_last_day() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("hourly")).await;
    insert_activity(&db.pool, &unique("act"), &user, "login", "signed_in").await;
    insert_mcp_execution(&db.pool, user.as_str(), "search", "success").await;

    let hours = list_hourly_activity(&db.pool)
        .await
        .expect("query succeeds");

    let total: i64 = hours.iter().map(|h| h.count).sum();
    assert_eq!(
        total, 2,
        "both sources contribute to the same hour histogram"
    );
    assert!(hours.windows(2).all(|w| w[0].hour < w[1].hour));
    db.cleanup().await;
}

#[tokio::test]
async fn list_recent_mcp_errors_reports_only_failures() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("errors")).await;
    let broken = unique("tool");
    insert_mcp_execution(&db.pool, user.as_str(), &broken, "failed").await;
    insert_mcp_execution(&db.pool, user.as_str(), "healthy", "success").await;

    let errors = list_recent_mcp_errors(&db.pool)
        .await
        .expect("query succeeds");

    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].tool_name, broken);
    db.cleanup().await;
}

#[tokio::test]
async fn list_tool_success_rates_needs_three_calls_before_reporting() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("rates")).await;
    let sparse = unique("tool");
    let busy = unique("tool");
    insert_mcp_execution(&db.pool, user.as_str(), &sparse, "success").await;
    insert_mcp_execution(&db.pool, user.as_str(), &busy, "success").await;
    insert_mcp_execution(&db.pool, user.as_str(), &busy, "success").await;
    insert_mcp_execution(&db.pool, user.as_str(), &busy, "failed").await;

    let rates = list_tool_success_rates(&db.pool)
        .await
        .expect("query succeeds");

    assert!(
        rates.iter().all(|r| r.tool_name != sparse),
        "a tool under the HAVING threshold is not rated"
    );
    let row = rates
        .iter()
        .find(|r| r.tool_name == busy)
        .expect("the busy tool is rated");
    assert_eq!(row.total, 3);
    assert_eq!(row.successes, 2);
    assert_eq!(row.failures, 1);
    assert!((row.success_pct - 66.666_666).abs() < 0.01);
    db.cleanup().await;
}

#[tokio::test]
async fn list_events_pages_the_feed_and_reports_the_total() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("feed")).await;
    let session = unique("session");
    let marker = unique("MarkerTool");
    for _ in 0..3 {
        let id = unique("evt");
        let mut spec = EventSpec::tool_use(&id, &user, &session);
        spec.tool_name = Some(&marker);
        insert_event(&db.pool, &spec).await;
    }
    let query = EventsQuery {
        search: Some(marker.clone()),
        event_type: None,
        limit: 2,
        offset: 0,
    };

    let response = list_events(&db.pool, &query).await.expect("query succeeds");

    assert_eq!(response.total, 3, "the total ignores the page window");
    assert_eq!(response.events.len(), 2);
    assert!(response.events.iter().all(|e| e.user_id == user));
    db.cleanup().await;
}

#[tokio::test]
async fn list_events_filters_by_event_type() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("typed")).await;
    let session = unique("session");
    let wanted = unique("SpecialEvent");
    let id = unique("evt");
    let mut spec = EventSpec::tool_use(&id, &user, &session);
    spec.event_type = &wanted;
    insert_event(&db.pool, &spec).await;
    insert_event(
        &db.pool,
        &EventSpec::tool_use(&unique("evt"), &user, &session),
    )
    .await;
    let query = EventsQuery {
        search: None,
        event_type: Some(wanted.clone()),
        limit: 50,
        offset: 0,
    };

    let response = list_events(&db.pool, &query).await.expect("query succeeds");

    assert_eq!(response.total, 1);
    assert_eq!(response.events[0].event_type, wanted);
    db.cleanup().await;
}

#[tokio::test]
async fn get_dashboard_data_assembles_without_error_on_a_quiet_instance() {
    let Some(db) = TempDb::create().await else {
        return;
    };

    let data = get_dashboard_data(&db.pool, "24 hours", "1 hour", "24h", "7d")
        .await
        .expect("the landing page must render on an instance with no traffic");

    assert_eq!(data.stats.total_logins, 0);
    assert_eq!(data.active_users_24h, 0);
    assert!(!data.usage_timeseries.is_empty());
    db.cleanup().await;
}
