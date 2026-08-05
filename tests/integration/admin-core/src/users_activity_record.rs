//! `activity::record` and the near-duplicate suppression in front of it.
//!
//! Hook events arrive far faster than a human-readable timeline can absorb, so
//! four categories collapse repeats inside a window rather than writing every
//! one. That suppression is the whole reason the writer exists as more than an
//! `INSERT`, and each rule keys on something different — the user for logins,
//! the user *and* tool name for tool usage, the server name alone for MCP
//! rejections, and the session id forever for session starts. A rule that
//! deduplicated on the wrong key would either flood the feed or silently drop
//! a distinct event, and neither shows up in a test that writes one row.
//!
//! `record` returns `()` and swallows its own failures by design, so every
//! assertion here reads the table back.

use serde_json::json;
use sqlx::PgPool;
use systemprompt::identifiers::UserId;
use systemprompt_web_admin::activity::{
    ActivityAction, ActivityCategory, ActivityEntity, ActivityEntityRef, NewActivity, record,
};

use crate::fixtures::{insert_user, unclaimed_email, unique};
use crate::tempdb::TempDb;

struct Event<'a> {
    user_id: &'a UserId,
    category: ActivityCategory,
    action: ActivityAction,
    entity: Option<(ActivityEntity, Option<&'a str>, Option<&'a str>)>,
    description: &'a str,
}

fn new_activity(event: Event<'_>) -> NewActivity {
    NewActivity {
        user_id: event.user_id.clone(),
        category: event.category,
        action: event.action,
        entity: event.entity.map(|(kind, id, name)| ActivityEntityRef {
            kind,
            id: id.map(ToOwned::to_owned),
            name: name.map(ToOwned::to_owned),
        }),
        description: event.description.to_owned(),
        metadata: json!({ "source": "integration-suite" }),
    }
}

async fn rows_for(pool: &PgPool, user_id: &UserId) -> i64 {
    sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM user_activity WHERE user_id = $1")
        .bind(user_id.as_str())
        .fetch_one(pool)
        .await
        .expect("count activity rows")
}

async fn use_tool(pool: &PgPool, user_id: &UserId, tool: Option<&str>) {
    record(
        pool,
        new_activity(Event {
            user_id,
            category: ActivityCategory::ToolUsage,
            action: ActivityAction::Used,
            entity: Some((ActivityEntity::Tool, None, tool)),
            description: "Used a tool",
        }),
    )
    .await;
}

async fn reject_mcp(pool: &PgPool, user_id: &UserId, server: Option<&str>) {
    record(
        pool,
        new_activity(Event {
            user_id,
            category: ActivityCategory::McpAccess,
            action: ActivityAction::Rejected,
            entity: Some((ActivityEntity::McpServer, None, server)),
            description: "Access denied",
        }),
    )
    .await;
}

async fn start_session(pool: &PgPool, user_id: &UserId, session_id: Option<&str>) {
    record(
        pool,
        new_activity(Event {
            user_id,
            category: ActivityCategory::Session,
            action: ActivityAction::Started,
            entity: Some((ActivityEntity::Session, session_id, None)),
            description: "Session started",
        }),
    )
    .await;
}

async fn seed_user(pool: &PgPool, label: &str) -> UserId {
    insert_user(pool, &unique(label), &unclaimed_email(label)).await
}

#[tokio::test]
async fn a_recorded_event_lands_with_its_entity_and_metadata() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = seed_user(&db.pool, "activity-write").await;

    record(
        &db.pool,
        new_activity(Event {
            user_id: &user,
            category: ActivityCategory::MarketplaceEdit,
            action: ActivityAction::Updated,
            entity: Some((ActivityEntity::Plugin, Some("plug-1"), Some("Billing"))),
            description: "Updated the billing plugin",
        }),
    )
    .await;

    let row = sqlx::query_as::<
        _,
        (
            String,
            String,
            Option<String>,
            Option<String>,
            Option<String>,
            String,
        ),
    >(
        "SELECT category, action, entity_type, entity_id, entity_name, description
         FROM user_activity WHERE user_id = $1",
    )
    .bind(user.as_str())
    .fetch_one(db.pool.as_ref())
    .await
    .expect("the event was written");

    assert_eq!(row.0, "marketplace_edit");
    assert_eq!(row.1, "updated");
    assert_eq!(row.2.as_deref(), Some("plugin"));
    assert_eq!(row.3.as_deref(), Some("plug-1"));
    assert_eq!(row.4.as_deref(), Some("Billing"));
    assert_eq!(row.5, "Updated the billing plugin");

    db.cleanup().await;
}

#[tokio::test]
async fn an_event_with_no_entity_writes_null_entity_columns() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = seed_user(&db.pool, "activity-bare").await;

    record(
        &db.pool,
        new_activity(Event {
            user_id: &user,
            category: ActivityCategory::Prompt,
            action: ActivityAction::Submitted,
            entity: None,
            description: "Submitted a prompt",
        }),
    )
    .await;

    let row = sqlx::query_as::<_, (Option<String>, Option<String>, Option<String>)>(
        "SELECT entity_type, entity_id, entity_name FROM user_activity WHERE user_id = $1",
    )
    .bind(user.as_str())
    .fetch_one(db.pool.as_ref())
    .await
    .expect("the event was written");

    assert_eq!(row, (None, None, None), "no entity means three NULLs");

    db.cleanup().await;
}

#[tokio::test]
async fn uncollapsed_categories_write_every_repeat() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = seed_user(&db.pool, "activity-repeat").await;

    // `prompt` has no suppression rule. Three identical submissions must
    // produce three rows — suppression must not be the default.
    for _ in 0..3 {
        record(
            &db.pool,
            new_activity(Event {
                user_id: &user,
                category: ActivityCategory::Prompt,
                action: ActivityAction::Submitted,
                entity: None,
                description: "Submitted a prompt",
            }),
        )
        .await;
    }

    assert_eq!(rows_for(&db.pool, &user).await, 3);

    db.cleanup().await;
}

#[tokio::test]
async fn repeat_logins_within_the_hour_collapse_to_one_row() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = seed_user(&db.pool, "activity-login").await;
    let other = seed_user(&db.pool, "activity-login-other").await;

    for _ in 0..3 {
        record(
            &db.pool,
            new_activity(Event {
                user_id: &user,
                category: ActivityCategory::Login,
                action: ActivityAction::LoggedIn,
                entity: None,
                description: "Signed in",
            }),
        )
        .await;
    }
    assert_eq!(rows_for(&db.pool, &user).await, 1, "one row per hour");

    // The rule keys on the user: a second person signing in is a distinct
    // event, not a duplicate of the first.
    record(
        &db.pool,
        new_activity(Event {
            user_id: &other,
            category: ActivityCategory::Login,
            action: ActivityAction::LoggedIn,
            entity: None,
            description: "Signed in",
        }),
    )
    .await;
    assert_eq!(rows_for(&db.pool, &other).await, 1);

    // Aging the existing row past the window re-opens it.
    sqlx::query(
        "UPDATE user_activity SET created_at = NOW() - INTERVAL '2 hours' WHERE user_id = $1",
    )
    .bind(user.as_str())
    .execute(db.pool.as_ref())
    .await
    .expect("age the login row");

    record(
        &db.pool,
        new_activity(Event {
            user_id: &user,
            category: ActivityCategory::Login,
            action: ActivityAction::LoggedIn,
            entity: None,
            description: "Signed in",
        }),
    )
    .await;
    assert_eq!(
        rows_for(&db.pool, &user).await,
        2,
        "a login after the window is a new entry"
    );

    db.cleanup().await;
}

#[tokio::test]
async fn tool_usage_collapses_per_tool_not_per_user() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = seed_user(&db.pool, "activity-tool").await;

    let write = async |tool: Option<&str>| {
        use_tool(&db.pool, &user, tool).await;
    };

    write(Some("Read")).await;
    write(Some("Read")).await;
    assert_eq!(rows_for(&db.pool, &user).await, 1, "the repeat collapses");

    // A different tool inside the same window is a different event.
    write(Some("Write")).await;
    assert_eq!(rows_for(&db.pool, &user).await, 2);

    // No tool name means nothing to compare on, so the rule cannot fire and
    // the event must be written rather than dropped.
    write(None).await;
    write(None).await;
    assert_eq!(
        rows_for(&db.pool, &user).await,
        4,
        "an unnamed tool is never treated as a duplicate"
    );

    db.cleanup().await;
}

#[tokio::test]
async fn mcp_rejections_collapse_across_users() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let first = seed_user(&db.pool, "activity-mcp-a").await;
    let second = seed_user(&db.pool, "activity-mcp-b").await;
    let server = unique("mcp-server");

    let reject = async |user: &UserId, name: Option<&str>| {
        reject_mcp(&db.pool, user, name).await;
    };

    reject(&first, Some(&server)).await;
    // Deliberately a *different* user. This rule keys on the server name
    // alone, because a misconfigured server rejecting everyone should read as
    // one incident rather than one line per employee.
    reject(&second, Some(&server)).await;

    assert_eq!(rows_for(&db.pool, &first).await, 1);
    assert_eq!(
        rows_for(&db.pool, &second).await,
        0,
        "the second rejection is suppressed by the first, across users"
    );

    // A rejection naming no server cannot be matched, so it is always written.
    reject(&second, None).await;
    assert_eq!(rows_for(&db.pool, &second).await, 1);

    db.cleanup().await;
}

#[tokio::test]
async fn a_session_start_is_recorded_once_for_all_time() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = seed_user(&db.pool, "activity-session").await;
    let session = unique("session");

    let start = async |id: Option<&str>| {
        start_session(&db.pool, &user, id).await;
    };

    start(Some(&session)).await;
    start(Some(&session)).await;
    assert_eq!(rows_for(&db.pool, &user).await, 1);

    // Unlike the windowed rules, this one has no expiry — a session starts
    // exactly once, so age must not re-open it.
    sqlx::query("UPDATE user_activity SET created_at = NOW() - INTERVAL '30 days'")
        .execute(db.pool.as_ref())
        .await
        .expect("age the session row");
    start(Some(&session)).await;
    assert_eq!(
        rows_for(&db.pool, &user).await,
        1,
        "a session start never becomes recordable again"
    );

    // A different session, and a start with no id at all, both still record.
    start(Some(&unique("session"))).await;
    start(None).await;
    assert_eq!(rows_for(&db.pool, &user).await, 3);

    db.cleanup().await;
}

#[tokio::test]
async fn a_session_ending_is_not_governed_by_the_start_rule() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = seed_user(&db.pool, "activity-session-end").await;
    let session = unique("session");

    for action in [ActivityAction::Started, ActivityAction::Ended] {
        record(
            &db.pool,
            new_activity(Event {
                user_id: &user,
                category: ActivityCategory::Session,
                action,
                entity: Some((ActivityEntity::Session, Some(&session), None)),
                description: "Session lifecycle",
            }),
        )
        .await;
    }

    // The rule matches `("session", "started")` exactly. An end sharing the
    // same entity id must not be swallowed by the start already on record.
    assert_eq!(rows_for(&db.pool, &user).await, 2);

    db.cleanup().await;
}
