//! `repositories::governance::{filter_options, hook_events, resolve}` — the
//! identity filter ribbon, the merged hook-event feed, and the id resolver
//! behind the global search bar.

use chrono::{Duration, Utc};
use systemprompt_web_admin::repositories::governance::{filter_options, hook_events, resolve};

use crate::fixtures::{
    DecisionSpec, RequestSpec, insert_decision, insert_request, insert_session, insert_user,
    unclaimed_email, unique, wide_window,
};
use crate::tempdb::TempDb;

#[tokio::test]
async fn get_filter_options_is_empty_before_any_decision() {
    let Some(db) = TempDb::create().await else {
        return;
    };

    let options = filter_options::get_filter_options(&db.pool, wide_window())
        .await
        .expect("query succeeds");

    assert!(options.users.is_empty());
    assert!(options.policies.is_empty());
    assert!(options.decisions.is_empty());
    db.cleanup().await;
}

#[tokio::test]
async fn get_filter_options_only_offers_facets_present_in_the_window() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("facets")).await;
    let session = unique("session");
    let id = unique("dec");
    let agent = unique("agent");
    let mut spec = DecisionSpec::allow(&id, &user, &session);
    spec.agent_id = Some(&agent);
    spec.agent_scope = Some("subagent");
    insert_decision(&db.pool, &spec).await;
    let stale = unique("dec");
    let mut old = DecisionSpec::allow(&stale, &user, &session);
    old.policy = "ancient_policy";
    old.created_at = Utc::now() - Duration::days(3);
    insert_decision(&db.pool, &old).await;

    let options = filter_options::get_filter_options(&db.pool, wide_window())
        .await
        .expect("query succeeds");

    assert_eq!(options.users.len(), 1);
    assert_eq!(options.users[0].id, user.as_str());
    assert_eq!(
        options.users[0].count, 1,
        "the out-of-window row is not counted"
    );
    assert_eq!(options.agents.len(), 1);
    assert_eq!(options.agents[0].id, agent);
    assert_eq!(options.agent_scopes[0].id, "subagent");
    assert!(options.policies.iter().all(|p| p.id != "ancient_policy"));
    assert_eq!(options.decisions[0].id, "allow");
    db.cleanup().await;
}

#[tokio::test]
async fn hook_event_counters_read_the_two_hook_tables() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("hooks")).await;
    let session = unique("session");
    insert_decision(
        &db.pool,
        &DecisionSpec::allow(&unique("dec"), &user, &session),
    )
    .await;

    let pre = hook_events::count_pretool_fired_24h(&db.pool)
        .await
        .expect("query succeeds");

    assert_eq!(pre, 1, "PreToolUse is counted from governance_decisions");
    db.cleanup().await;
}

#[tokio::test]
async fn recent_hook_events_merges_both_sources_newest_first() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("merge")).await;
    let session = unique("session");
    let older = unique("dec");
    let mut decision = DecisionSpec::allow(&older, &user, &session);
    decision.created_at = Utc::now() - Duration::minutes(10);
    insert_decision(&db.pool, &decision).await;

    let events = hook_events::recent_hook_events(&db.pool, 50)
        .await
        .expect("query succeeds");

    let mine = events
        .iter()
        .find(|e| e.kind == "PreToolUse" && e.user_id == user)
        .expect("the decision surfaces as a PreToolUse event");
    assert_eq!(mine.status.as_deref(), Some("allow"));
    assert!(
        events
            .windows(2)
            .all(|w| w[0].created_at >= w[1].created_at),
        "the merge must stay newest-first"
    );
    db.cleanup().await;
}

#[tokio::test]
async fn resolve_id_returns_none_for_an_unknown_identifier() {
    let Some(db) = TempDb::create().await else {
        return;
    };

    let found = resolve::resolve_id(&db.pool, &unique("mystery"))
        .await
        .expect("lookup succeeds");

    assert!(found.is_none());
    db.cleanup().await;
}

#[tokio::test]
async fn resolve_id_recognises_a_request_id() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("resolve")).await;
    let request = unique("req");
    insert_request(&db.pool, &RequestSpec::completed(&request, &user)).await;

    let found = resolve::resolve_id(&db.pool, &request)
        .await
        .expect("lookup succeeds")
        .expect("the request resolves");

    assert!(matches!(found.kind, resolve::ResolvedKind::Request));
    assert_eq!(found.id, request);
    db.cleanup().await;
}

#[tokio::test]
async fn resolve_id_falls_through_to_a_governance_only_session() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("govsess")).await;
    let session = unique("session");
    insert_decision(
        &db.pool,
        &DecisionSpec::allow(&unique("dec"), &user, &session),
    )
    .await;

    let found = resolve::resolve_id(&db.pool, &session)
        .await
        .expect("lookup succeeds")
        .expect("the session resolves");

    assert!(matches!(found.kind, resolve::ResolvedKind::Session));
    assert_eq!(found.id, session);
    db.cleanup().await;
}

#[tokio::test]
async fn resolve_id_prefers_the_request_over_the_session_that_carries_it() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("precedence")).await;
    let session = unique("session");
    insert_session(&db.pool, &session, &user).await;
    let request = unique("req");
    let mut spec = RequestSpec::completed(&request, &user);
    spec.session_id = Some(&session);
    insert_request(&db.pool, &spec).await;

    let by_request = resolve::resolve_id(&db.pool, &request)
        .await
        .expect("lookup succeeds")
        .expect("the request resolves");
    let by_session = resolve::resolve_id(&db.pool, &session)
        .await
        .expect("lookup succeeds")
        .expect("the session resolves");

    assert!(matches!(by_request.kind, resolve::ResolvedKind::Request));
    assert!(matches!(by_session.kind, resolve::ResolvedKind::Session));
    db.cleanup().await;
}
