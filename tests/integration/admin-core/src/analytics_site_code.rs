//! `repositories::analytics::site::code` — the dashboard's code tab.
//!
//! Two measurement frames the page must never mix: hook-observed AI line
//! deltas (`loc_added_ai`, what Claude applied through Edit/Write) and git
//! commit diff totals (`commit_insertions`, AI and human lines together). The
//! daily series is spine-joined, so a day with no rollup is a zero rather than
//! a gap, and both reads honour the project and per-user scope the page is
//! filtered by.

use chrono::{Duration, Utc};
use systemprompt_web_admin::repositories::analytics::site::SiteScope;
use systemprompt_web_admin::repositories::analytics::site::code::{
    get_code_totals, list_daily_code_series,
};
use systemprompt_web_admin::repositories::scope::{Attribution, SubjectScope};
use systemprompt_web_admin::util::time_range::{TimeRange, TimeRangePreset};

use crate::fixtures::{insert_user, unclaimed_email, unique};
use crate::tempdb::TempDb;

fn window(days_back: i64) -> TimeRange {
    let now = Utc::now();
    TimeRange {
        from: now - Duration::days(days_back),
        to: now + Duration::minutes(1),
        preset: TimeRangePreset::Custom,
        rejected_bounds: false,
    }
}

// One rollup row, `days_back` days ago. The two frames are given deliberately
// different values so a query reading the wrong column is visible.
struct Rollup {
    days_back: i64,
    loc_added_ai: i64,
    loc_removed_ai: i64,
    commits: i64,
    insertions: i64,
    deletions: i64,
}

async fn insert_rollup(pool: &sqlx::PgPool, user: &systemprompt::identifiers::UserId, r: &Rollup) {
    sqlx::query(
        "INSERT INTO admin_usage_daily_rollups
             (user_id, date, sessions_count, prompts, tool_uses, errors,
              loc_added_ai, loc_removed_ai, commits_count, commit_insertions,
              commit_deletions, ai_requests_count, input_tokens, output_tokens,
              cost_microdollars)
         VALUES ($1, (NOW() - ($2 || ' days')::interval)::date,
                 1, 2, 3, 0, $3, $4, $5, $6, $7, 4, 400, 100, 2000)",
    )
    .bind(user.as_str())
    .bind(r.days_back.to_string())
    .bind(r.loc_added_ai)
    .bind(r.loc_removed_ai)
    .bind(r.commits)
    .bind(r.insertions)
    .bind(r.deletions)
    .execute(pool)
    .await
    .expect("insert a daily rollup");
}

async fn insert_edit_events(
    pool: &sqlx::PgPool,
    user: &systemprompt::identifiers::UserId,
    tool: &str,
    event_type: &str,
    count: i64,
) {
    sqlx::query(
        "INSERT INTO plugin_usage_daily
             (id, user_id, date, event_type, tool_name, event_count)
         VALUES ($5, $1, CURRENT_DATE, $2, $3, $4)",
    )
    .bind(user.as_str())
    .bind(event_type)
    .bind(tool)
    .bind(count)
    .bind(unique("pud"))
    .execute(pool)
    .await
    .expect("insert a plugin usage row");
}

fn scope_for(user: &systemprompt::identifiers::UserId) -> SiteScope {
    SiteScope {
        user_id: Some(user.clone()),
        scope: SubjectScope::All,
        attribution: Attribution::Exclusive,
    }
}

#[tokio::test]
async fn get_code_totals_sums_both_measurement_frames_separately() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("code")).await;
    insert_rollup(
        &db.pool,
        &user,
        &Rollup {
            days_back: 0,
            loc_added_ai: 40,
            loc_removed_ai: 5,
            commits: 2,
            insertions: 90,
            deletions: 12,
        },
    )
    .await;
    insert_rollup(
        &db.pool,
        &user,
        &Rollup {
            days_back: 1,
            loc_added_ai: 10,
            loc_removed_ai: 1,
            commits: 1,
            insertions: 30,
            deletions: 3,
        },
    )
    .await;

    let totals = get_code_totals(&db.pool, window(7), &scope_for(&user))
        .await
        .expect("the totals read succeeds");

    // Why: hook-observed AI lines and git diff totals are different frames of
    // the same work — a query that summed them together, or read one column
    // for both, would report a number the page cannot explain.
    assert_eq!(totals.loc_added_ai, 50);
    assert_eq!(totals.loc_removed_ai, 6);
    assert_eq!(totals.commits, 3);
    assert_eq!(totals.commit_insertions, 120);
    assert_eq!(totals.commit_deletions, 15);

    db.cleanup().await;
}

#[tokio::test]
async fn a_rollup_outside_the_window_is_not_counted() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("window")).await;
    insert_rollup(
        &db.pool,
        &user,
        &Rollup {
            days_back: 40,
            loc_added_ai: 999,
            loc_removed_ai: 999,
            commits: 99,
            insertions: 999,
            deletions: 999,
        },
    )
    .await;

    let totals = get_code_totals(&db.pool, window(7), &scope_for(&user))
        .await
        .expect("the totals read succeeds");

    assert_eq!(totals.loc_added_ai, 0, "the 40-day-old row is out of range");
    assert_eq!(totals.commits, 0);

    db.cleanup().await;
}

#[tokio::test]
async fn only_applied_edit_tools_count_as_ai_edit_operations() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("edits")).await;
    insert_edit_events(&db.pool, &user, "Edit", "PostToolUse", 7).await;
    insert_edit_events(&db.pool, &user, "Write", "PostToolUse", 3).await;
    // A read is not an edit, and a PreToolUse event is an intention rather
    // than an applied change — neither belongs in the count.
    insert_edit_events(&db.pool, &user, "Read", "PostToolUse", 100).await;
    insert_edit_events(&db.pool, &user, "Edit", "PreToolUse", 100).await;

    let totals = get_code_totals(&db.pool, window(1), &scope_for(&user))
        .await
        .expect("the totals read succeeds");

    assert_eq!(totals.ai_edit_operations, 10);

    db.cleanup().await;
}

#[tokio::test]
async fn another_users_work_is_excluded_by_the_scope() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let mine = insert_user(&db.pool, &unique("user"), &unclaimed_email("mine")).await;
    let theirs = insert_user(&db.pool, &unique("user"), &unclaimed_email("theirs")).await;
    for user in [&mine, &theirs] {
        insert_rollup(
            &db.pool,
            user,
            &Rollup {
                days_back: 0,
                loc_added_ai: 25,
                loc_removed_ai: 0,
                commits: 1,
                insertions: 10,
                deletions: 0,
            },
        )
        .await;
    }

    let scoped = get_code_totals(&db.pool, window(7), &scope_for(&mine))
        .await
        .expect("the scoped read succeeds");
    let everyone = get_code_totals(&db.pool, window(7), &SiteScope::new(SubjectScope::All))
        .await
        .expect("the unscoped read succeeds");

    assert_eq!(scoped.loc_added_ai, 25, "one user's own work");
    assert_eq!(everyone.loc_added_ai, 50, "and both users' together");

    db.cleanup().await;
}

#[tokio::test]
async fn the_daily_series_fills_a_day_with_no_rollup_with_zeroes() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("series")).await;
    insert_rollup(
        &db.pool,
        &user,
        &Rollup {
            days_back: 0,
            loc_added_ai: 40,
            loc_removed_ai: 5,
            commits: 2,
            insertions: 90,
            deletions: 12,
        },
    )
    .await;

    let series = list_daily_code_series(&db.pool, window(3), &scope_for(&user))
        .await
        .expect("the series read succeeds");

    // Why: the chart is drawn off this spine, so a quiet day has to arrive as
    // a zero. A gap would make the line jump between the days either side of
    // it and misreport a pause as continuous work.
    assert_eq!(
        series.len(),
        4,
        "one bucket per day in the window: {series:?}"
    );
    let today = series.last().expect("the window ends today");
    assert_eq!(today.loc_added_ai, 40);
    assert_eq!(today.commit_insertions, 90);
    assert!(
        series[..3]
            .iter()
            .all(|b| b.commits == 0 && b.loc_added_ai == 0),
        "the quiet days are zeroes, not gaps: {series:?}"
    );
    assert!(
        series.windows(2).all(|w| w[0].date < w[1].date),
        "the series is ordered oldest first"
    );

    db.cleanup().await;
}
