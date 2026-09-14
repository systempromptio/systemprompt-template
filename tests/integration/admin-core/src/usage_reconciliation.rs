//! The profile pane and the analytics dashboard must report the same usage.
//!
//! They did not. `/admin/profile` summed `tokens_used`; `/admin/analytics`
//! summed `input_tokens + output_tokens`. Both are defensible in isolation, and
//! on live data they differed by two orders of magnitude, because `tokens_used`
//! is the provider's own total and counts cache reads — which dominate a Claude
//! Code session. A user's month read 276,421 tokens on one page and 1,644 on
//! the other, with nothing on either page to suggest one of them was wrong.
//!
//! Nothing caught it because nothing compared them. Each side had tests, and
//! each side passed its own. These tests are the comparison: they seed one
//! known set of requests and assert the two read models agree, so the next
//! divergence fails here rather than being discovered in a screenshot.
//!
//! The scope is deliberately pinned to a single user, because that is the only
//! window in which the two are answering the same question — the profile pane
//! is always per-user, the dashboard is per-scope.

use chrono::{Duration, Utc};
use systemprompt_web_admin::repositories::analytics::site::SiteScope;
use systemprompt_web_admin::repositories::analytics::site::kpis::get_site_kpis;
use systemprompt_web_admin::repositories::scope::{Attribution, SubjectScope};
use systemprompt_web_admin::repositories::users::usage::get_usage_window;
use systemprompt_web_admin::util::time_range::{TimeRange, TimeRangePreset};

use crate::fixtures::{RequestSpec, insert_request, insert_user, unclaimed_email, unique};
use crate::tempdb::TempDb;

// Why: the dashboard takes an explicit range and the profile pane takes a
// trailing day count, so the two are only comparable when the range is exactly
// the trailing window. One day, ending a moment in the future so a row written
// during the test cannot fall outside it.
const WINDOW_DAYS: i32 = 1;

fn matching_window() -> TimeRange {
    let now = Utc::now();
    TimeRange {
        from: now - Duration::days(i64::from(WINDOW_DAYS)),
        to: now + Duration::minutes(1),
        preset: TimeRangePreset::Custom,
        rejected_bounds: false,
    }
}

#[tokio::test]
async fn the_profile_and_the_dashboard_agree_when_cache_tokens_dominate() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("recon")).await;

    // The shape that broke them: a provider total far larger than the visible
    // input and output, because the rest was read from cache.
    let mut request = RequestSpec::completed(&unique("req"), &user);
    request.input_tokens = 1_000;
    request.output_tokens = 644;
    request.tokens_used = Some(276_421);
    request.created_at = Utc::now() - Duration::hours(1);
    insert_request(&db.pool, &request).await;

    let profile = get_usage_window(&db.pool, &user, WINDOW_DAYS)
        .await
        .expect("profile usage window");
    let dashboard = get_site_kpis(
        &db.pool,
        matching_window(),
        &SiteScope {
            scope: SubjectScope::All,
            attribution: Attribution::Exclusive,
            user_id: Some(user.clone()),
        },
    )
    .await
    .expect("site kpis");

    assert_eq!(
        profile.tokens, dashboard.total_tokens,
        "the two pages must count the same tokens for the same user and window"
    );
    assert_eq!(
        profile.tokens, 276_421,
        "and both must count the provider's own total, cache reads included"
    );
    assert_eq!(
        profile.requests, dashboard.total_requests,
        "request counts must agree too"
    );
    assert_eq!(
        profile.cost_microdollars, dashboard.total_cost_microdollars,
        "spend must agree — a page whose tokens and cost come from different \
         definitions cannot state a cost per token"
    );
    db.cleanup().await;
}

#[tokio::test]
async fn the_profile_and_the_dashboard_agree_when_the_provider_reported_no_total() {
    let db = astound_test_common::db_or_skip!();
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("recon-null")).await;

    // Why: `tokens_used` is nullable, and both readers now aggregate that one
    // column rather than re-summing the components. The components no longer
    // partition the total -- cache reads are disjoint from input on every wire
    // -- so a fallback sum would over-count exactly the rows it was meant to
    // rescue. A row with no provider total contributes nothing, on both pages
    // identically, and a visible gap beats a wrong number.
    let mut request = RequestSpec::completed(&unique("req"), &user);
    request.input_tokens = 300;
    request.output_tokens = 45;
    request.tokens_used = None;
    request.created_at = Utc::now() - Duration::hours(1);
    insert_request(&db.pool, &request).await;

    let profile = get_usage_window(&db.pool, &user, WINDOW_DAYS)
        .await
        .expect("profile usage window");
    let dashboard = get_site_kpis(
        &db.pool,
        matching_window(),
        &SiteScope {
            scope: SubjectScope::All,
            attribution: Attribution::Exclusive,
            user_id: Some(user.clone()),
        },
    )
    .await
    .expect("site kpis");

    assert_eq!(
        profile.tokens, dashboard.total_tokens,
        "both pages read the same column, so a row without a total is absent \
         from both or from neither"
    );
    assert_eq!(
        profile.tokens, 0,
        "a row with no provider total is not re-summed from its components"
    );
    assert_eq!(
        profile.requests, dashboard.total_requests,
        "the request is still counted; only its tokens are unknown"
    );
    db.cleanup().await;
}

#[tokio::test]
async fn reasoning_tokens_are_reported_beside_the_total_not_added_into_it() {
    let db = astound_test_common::db_or_skip!();
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("recon-think")).await;

    // Why: a total no re-summation of the components could produce. A reader
    // that recomputes instead of aggregating `tokens_used` fails here.
    let mut request = RequestSpec::completed(&unique("req"), &user);
    request.input_tokens = 100;
    request.output_tokens = 40;
    request.reasoning_tokens = Some(25);
    request.cache_read_tokens = Some(70);
    request.cache_creation_tokens = Some(30);
    request.tokens_used = Some(9_999);
    request.created_at = Utc::now() - Duration::hours(1);
    insert_request(&db.pool, &request).await;

    let profile = get_usage_window(&db.pool, &user, WINDOW_DAYS)
        .await
        .expect("profile usage window");
    let dashboard = get_site_kpis(
        &db.pool,
        matching_window(),
        &SiteScope {
            scope: SubjectScope::All,
            attribution: Attribution::Exclusive,
            user_id: Some(user.clone()),
        },
    )
    .await
    .expect("site kpis");

    assert_eq!(profile.tokens, 9_999, "the stored total is read as stored");
    assert_eq!(dashboard.total_tokens, 9_999);
    assert_eq!(
        dashboard.reasoning_tokens, 25,
        "reasoning is reported so the dashboard can state a thinking share"
    );
    assert_eq!(
        dashboard.output_tokens, 40,
        "and the output it is a share of is reported beside it"
    );
    assert!(
        dashboard.reasoning_tokens <= dashboard.output_tokens,
        "reasoning is billed inside output, never alongside it"
    );
    db.cleanup().await;
}

#[tokio::test]
async fn a_user_with_no_project_still_counts_in_the_instance_wide_view() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    // Why: this was a reported bug, at the read-model level. The user has no
    // project row, and the admin's `All` scope must still account for their
    // traffic rather than silently dropping the unattached.
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("unattached")).await;
    let mut request = RequestSpec::completed(&unique("req"), &user);
    request.created_at = Utc::now() - Duration::hours(1);
    insert_request(&db.pool, &request).await;

    let dashboard = get_site_kpis(
        &db.pool,
        matching_window(),
        &SiteScope::new(SubjectScope::All),
    )
    .await
    .expect("site kpis");

    assert!(
        dashboard.total_requests >= 1,
        "a request from a user in no project must still be counted in the \
         instance-wide view every admin sees"
    );
    db.cleanup().await;
}
