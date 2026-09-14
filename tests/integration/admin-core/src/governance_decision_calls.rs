//! `repositories::governance::decision_calls` — the log folded from one row per
//! policy evaluation to one row per governed call.
//!
//! What these cover is the fold itself: that a trace's evaluations arrive as
//! one row carrying all of them, that a row without a trace is still a call,
//! that the row speaks for the worst evaluation in its group, and that the
//! page's total counts calls. The last is load-bearing beyond its size — the
//! page query and the count query repeat their filter text and nothing but this
//! test proves the two still agree.

use systemprompt::identifiers::UserId;

use systemprompt_web_admin::repositories::governance::decision_calls::list_decision_calls_paged;
use systemprompt_web_admin::repositories::governance::decision_log::{
    DecisionFilter, DecisionSort,
};
use systemprompt_web_admin::repositories::governance::{DecisionPage, PageSlice};
use systemprompt_web_admin::repositories::scope::SubjectScope;

use crate::fixtures::{
    DecisionSpec, insert_decision, insert_user, unclaimed_email, unique, wide_window,
};
use crate::tempdb::TempDb;

// Why: the scope every one of these runs under. The fixtures insert for a
// single user, so scoping to that user isolates the assertions from the
// schema's own seed rows without any test needing to know what those are.
fn only(user: &UserId) -> SubjectScope {
    SubjectScope::Users(vec![user.as_str().to_owned()])
}

async fn calls(
    db: &TempDb,
    user: &UserId,
    filter: &DecisionFilter,
) -> (
    Vec<systemprompt_web_admin::repositories::governance::decision_calls::DecisionCallRow>,
    i64,
) {
    list_decision_calls_paged(
        &db.pool,
        wide_window(),
        &only(user),
        filter,
        DecisionPage {
            sort: DecisionSort::default(),
            slice: PageSlice::first(50),
        },
    )
    .await
    .expect("query succeeds")
}

// Why: the shape this whole change exists for — the gateway gate, the authz
// webhook and core's audit sink all writing against one trace.
async fn insert_triplet(db: &TempDb, user: &UserId, session: &str, trace: &str, worst: &str) {
    let mut gate = DecisionSpec::allow(&unique("dec"), user, session);
    gate.policy = "default_allow";
    gate.tool_name = "user_prompt";
    gate.trace_id = Some(trace);
    insert_decision(&db.pool, &gate).await;

    let mut authz = DecisionSpec::allow(&unique("dec"), user, session);
    authz.policy = "authz";
    authz.tool_name = "claude-star-4203d1";
    authz.entity_type = Some("gateway_route");
    authz.trace_id = Some(trace);
    insert_decision(&db.pool, &authz).await;

    let mut rules = DecisionSpec::allow(&unique("dec"), user, session);
    rules.policy = "authz_rule_based";
    rules.tool_name = "claude-star-4203d1";
    rules.entity_type = Some("gateway_route");
    rules.decision = worst;
    rules.reason = "the rule refused it";
    rules.trace_id = Some(trace);
    insert_decision(&db.pool, &rules).await;
}

#[tokio::test]
async fn three_evaluations_sharing_a_trace_are_one_call() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("fold")).await;
    let session = unique("session");
    let trace = unique("trace");
    insert_triplet(&db, &user, &session, &trace, "allow").await;

    let (rows, total) = calls(&db, &user, &DecisionFilter::default()).await;

    assert_eq!(rows.len(), 1, "three evaluations, one call");
    assert_eq!(total, 1, "the total counts calls, not evaluations");
    let row = &rows[0];
    assert_eq!(row.eval_count, 3);
    assert_eq!(row.chain.0.len(), 3, "every evaluation is still on the row");
    assert_eq!(row.call_key, trace);
    db.cleanup().await;
}

#[tokio::test]
async fn a_decision_without_a_trace_is_still_a_call_of_its_own() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("lone")).await;
    let session = unique("session");
    let id = unique("dec");
    let mut lone = DecisionSpec::allow(&id, &user, &session);
    lone.policy = "authentication";
    lone.decision = "deny";
    insert_decision(&db.pool, &lone).await;

    let (rows, total) = calls(&db, &user, &DecisionFilter::default()).await;

    assert_eq!(total, 1);
    assert_eq!(rows[0].call_key, id, "the row's own id is its group key");
    assert_eq!(rows[0].eval_count, 1);
    assert!(rows[0].trace_id.is_none());
    db.cleanup().await;
}

#[tokio::test]
async fn the_row_speaks_for_the_worst_evaluation_in_its_group() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("worst")).await;
    let session = unique("session");
    let trace = unique("trace");
    insert_triplet(&db, &user, &session, &trace, "deny").await;

    let (rows, _) = calls(&db, &user, &DecisionFilter::default()).await;

    let row = &rows[0];
    assert_eq!(
        row.worst_decision, "deny",
        "two allows do not outvote a deny"
    );
    assert_eq!(row.worst_policy, "authz_rule_based");
    assert_eq!(row.worst_reason, "the rule refused it");
    assert_eq!(row.deny_count, 1);
    assert_eq!(row.warn_count, 0);
    db.cleanup().await;
}

#[tokio::test]
async fn a_policy_filter_returns_the_whole_call_not_the_matching_fragment() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("frag")).await;
    let session = unique("session");
    let trace = unique("trace");
    insert_triplet(&db, &user, &session, &trace, "allow").await;

    let filter = DecisionFilter {
        policy: Some("authz".to_owned()),
        ..DecisionFilter::default()
    };
    let (rows, total) = calls(&db, &user, &filter).await;

    assert_eq!(total, 1);
    assert_eq!(
        rows[0].chain.0.len(),
        3,
        "narrowing to one policy shows the call it ran in, not one line of it"
    );
    db.cleanup().await;
}

#[tokio::test]
async fn attention_selects_calls_that_contain_a_deny_or_a_warn() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("attn")).await;
    let session = unique("session");
    insert_triplet(&db, &user, &session, &unique("trace"), "allow").await;
    insert_triplet(&db, &user, &session, &unique("trace"), "warn").await;

    let (_, all) = calls(&db, &user, &DecisionFilter::default()).await;
    let filter = DecisionFilter {
        attention: true,
        ..DecisionFilter::default()
    };
    let (rows, attention) = calls(&db, &user, &filter).await;

    assert_eq!(all, 2, "both calls are in the window");
    assert_eq!(attention, 1, "only one of them objected");
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].warn_count, 1);
    assert_eq!(
        rows[0].chain.0.len(),
        1,
        "review groups contain warning evaluations; the full call remains in the unfiltered log"
    );
    db.cleanup().await;
}

// Why: the page query and the count query repeat their filter predicate, and
// `query_file!` cannot prove the two agree because a sort key or a filter
// spliced into the text would be unverified and an injection site. This is the
// guard the doc comment on both statements points at.
#[tokio::test]
async fn the_count_agrees_with_the_rows_under_every_filter() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("agree")).await;
    let session = unique("session");
    insert_triplet(&db, &user, &session, &unique("trace"), "allow").await;
    insert_triplet(&db, &user, &session, &unique("trace"), "deny").await;
    let mut lone = DecisionSpec::allow(&unique("dec"), &user, &session);
    lone.policy = "authentication";
    lone.decision = "deny";
    insert_decision(&db.pool, &lone).await;

    for filter in [
        DecisionFilter::default(),
        DecisionFilter {
            attention: true,
            ..DecisionFilter::default()
        },
        DecisionFilter {
            policy: Some("authz_rule_based".to_owned()),
            ..DecisionFilter::default()
        },
        DecisionFilter {
            decision: Some("deny".to_owned()),
            ..DecisionFilter::default()
        },
        DecisionFilter {
            search: Some("refused".to_owned()),
            ..DecisionFilter::default()
        },
    ] {
        let (rows, total) = calls(&db, &user, &filter).await;
        assert_eq!(
            i64::try_from(rows.len()).unwrap_or(-1),
            total,
            "page and count disagree for {filter:?}"
        );
    }
    db.cleanup().await;
}

#[tokio::test]
async fn attention_groups_recurrences_but_excludes_entropy_observations() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("calibration")).await;
    let session = unique("session");
    for _ in 0..3 {
        let id = unique("dec");
        let trace = unique("trace");
        let mut spec = DecisionSpec::allow(&id, &user, &session);
        spec.decision = "warn";
        spec.policy = "secret_scan";
        spec.reason = "confirmed credential fingerprint:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
        spec.trace_id = Some(&trace);
        insert_decision(&db.pool, &spec).await;
    }
    let id = unique("dec");
    let mut observation = DecisionSpec::allow(&id, &user, &session);
    observation.decision = "warn";
    observation.policy = "secret_scan";
    observation.reason = "secret detected: High-entropy token (possible credential)";
    insert_decision(&db.pool, &observation).await;
    let filter = DecisionFilter {
        attention: true,
        ..Default::default()
    };
    let (rows, total) = calls(&db, &user, &filter).await;
    assert_eq!(total, 1);
    assert_eq!(rows[0].eval_count, 3);
    let stats = systemprompt_web_admin::repositories::governance::decision_log::get_decision_stats(
        &db.pool,
        wide_window(),
        &only(&user),
    )
    .await
    .unwrap();
    assert_eq!(stats.attention_calls, total);
    assert_eq!(stats.warned, 4);
    db.cleanup().await;
}
