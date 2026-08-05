//! `repositories::governance::{counts, decisions, rankings}` — the allow/deny
//! rollups and the actor / policy leaderboards.
//!
//! `governance_decisions` carries no seed rows, so these tests assert on
//! absolute counts.

use chrono::{Duration, Utc};
use systemprompt_web_admin::repositories::governance::{
    get_governance_counts, get_governance_counts_windowed, list_decisions_for_policy,
    list_per_policy_counts, list_per_policy_counts_windowed, list_top_actors, list_top_policies,
};

use crate::fixtures::{DecisionSpec, insert_decision, insert_user, unclaimed_email, unique};
use crate::tempdb::TempDb;

#[tokio::test]
async fn get_governance_counts_is_zero_on_a_fresh_database() {
    let Some(db) = TempDb::create().await else {
        return;
    };

    let counts = get_governance_counts(&db.pool)
        .await
        .expect("query succeeds");

    assert_eq!(counts.total, 0);
    assert_eq!(counts.allowed, 0);
    assert_eq!(counts.denied, 0);
    assert_eq!(counts.secret_breaches, 0);
    db.cleanup().await;
}

#[tokio::test]
async fn get_governance_counts_splits_allow_from_deny() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("counts")).await;
    let session = unique("session");
    insert_decision(
        &db.pool,
        &DecisionSpec::allow(&unique("dec"), &user, &session),
    )
    .await;
    let mut denial = DecisionSpec::allow(&unique("dec"), &user, &session);
    denial.decision = "deny";
    denial.policy = "secret_scan";
    denial.reason = "AWS secret detected in the payload";
    insert_decision(&db.pool, &denial).await;

    let counts = get_governance_counts(&db.pool)
        .await
        .expect("query succeeds");

    assert_eq!(counts.total, 2);
    assert_eq!(counts.allowed, 1);
    assert_eq!(counts.denied, 1);
    assert_eq!(
        counts.secret_breaches, 1,
        "the breach count reads the reason text, not the policy name"
    );
    db.cleanup().await;
}

#[tokio::test]
async fn get_governance_counts_windowed_excludes_older_decisions() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("windowed")).await;
    let session = unique("session");
    insert_decision(
        &db.pool,
        &DecisionSpec::allow(&unique("dec"), &user, &session),
    )
    .await;
    let mut old = DecisionSpec::allow(&unique("dec"), &user, &session);
    old.created_at = Utc::now() - Duration::hours(3);
    insert_decision(&db.pool, &old).await;

    let lifetime = get_governance_counts(&db.pool)
        .await
        .expect("query succeeds");
    let recent = get_governance_counts_windowed(&db.pool, 600)
        .await
        .expect("query succeeds");

    assert_eq!(lifetime.total, 2);
    assert_eq!(
        recent.total, 1,
        "a ten-minute window drops the three-hour-old row"
    );
    db.cleanup().await;
}

#[tokio::test]
async fn list_per_policy_counts_groups_by_policy_and_reports_the_last_hit() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("policy")).await;
    let session = unique("session");
    for decision in ["allow", "allow", "deny"] {
        let id = unique("dec");
        let mut spec = DecisionSpec::allow(&id, &user, &session);
        spec.decision = decision;
        spec.policy = "rate_limit";
        insert_decision(&db.pool, &spec).await;
    }

    let rows = list_per_policy_counts(&db.pool)
        .await
        .expect("query succeeds");

    let row = rows
        .iter()
        .find(|r| r.policy == "rate_limit")
        .expect("the policy appears");
    assert_eq!(row.allowed, 2);
    assert_eq!(row.denied, 1);
    assert!(row.last_at.is_some());
    db.cleanup().await;
}

#[tokio::test]
async fn list_per_policy_counts_windowed_narrows_to_the_window() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("policywin")).await;
    let session = unique("session");
    let id = unique("dec");
    let mut old = DecisionSpec::allow(&id, &user, &session);
    old.policy = "blocklist";
    old.created_at = Utc::now() - Duration::days(2);
    insert_decision(&db.pool, &old).await;

    let recent = list_per_policy_counts_windowed(&db.pool, 600)
        .await
        .expect("query succeeds");
    let lifetime = list_per_policy_counts(&db.pool)
        .await
        .expect("query succeeds");

    assert!(recent.iter().all(|r| r.policy != "blocklist"));
    assert!(lifetime.iter().any(|r| r.policy == "blocklist"));
    db.cleanup().await;
}

#[tokio::test]
async fn list_decisions_for_policy_returns_only_that_policy_newest_first() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("detail")).await;
    let session = unique("session");
    let older = unique("dec");
    let mut first = DecisionSpec::allow(&older, &user, &session);
    first.policy = "scope_check";
    first.created_at = Utc::now() - Duration::minutes(5);
    insert_decision(&db.pool, &first).await;
    let newer = unique("dec");
    let mut second = DecisionSpec::allow(&newer, &user, &session);
    second.policy = "scope_check";
    insert_decision(&db.pool, &second).await;
    let unrelated = unique("dec");
    let mut other = DecisionSpec::allow(&unrelated, &user, &session);
    other.policy = "blocklist";
    insert_decision(&db.pool, &other).await;

    let rows = list_decisions_for_policy(&db.pool, "scope_check", 10)
        .await
        .expect("query succeeds");

    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].id, newer, "newest first");
    assert_eq!(rows[1].id, older);
    db.cleanup().await;
}

#[tokio::test]
async fn list_decisions_for_policy_honours_the_limit() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("limited")).await;
    let session = unique("session");
    for _ in 0..3 {
        insert_decision(
            &db.pool,
            &DecisionSpec::allow(&unique("dec"), &user, &session),
        )
        .await;
    }

    let rows = list_decisions_for_policy(&db.pool, "scope_check", 2)
        .await
        .expect("query succeeds");

    assert_eq!(rows.len(), 2);
    db.cleanup().await;
}

#[tokio::test]
async fn list_decisions_for_policy_is_empty_for_an_unknown_policy() {
    let Some(db) = TempDb::create().await else {
        return;
    };

    let rows = list_decisions_for_policy(&db.pool, &unique("nope"), 10)
        .await
        .expect("query succeeds");

    assert!(rows.is_empty());
    db.cleanup().await;
}

#[tokio::test]
async fn list_top_actors_ranks_by_denials_and_counts_secret_hits() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let noisy = insert_user(&db.pool, &unique("user"), &unclaimed_email("noisy")).await;
    let quiet = insert_user(&db.pool, &unique("user"), &unclaimed_email("quiet")).await;
    let session = unique("session");
    for _ in 0..2 {
        let id = unique("dec");
        let mut spec = DecisionSpec::allow(&id, &noisy, &session);
        spec.decision = "deny";
        spec.policy = "secret_scan";
        insert_decision(&db.pool, &spec).await;
    }
    insert_decision(
        &db.pool,
        &DecisionSpec::allow(&unique("dec"), &quiet, &session),
    )
    .await;

    let actors = list_top_actors(&db.pool, 3600, 10)
        .await
        .expect("query succeeds");

    assert_eq!(actors[0].user_id, noisy.as_str());
    assert_eq!(actors[0].deny_count, 2);
    assert_eq!(actors[0].secret_count, 2);
    assert_eq!(actors[0].total, 2);
    let quiet_row = actors
        .iter()
        .find(|a| a.user_id == quiet.as_str())
        .expect("the quiet actor is still listed");
    assert_eq!(quiet_row.deny_count, 0);
    db.cleanup().await;
}

#[tokio::test]
async fn list_top_policies_only_counts_denials() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("toppol")).await;
    let session = unique("session");
    let id = unique("dec");
    let mut denial = DecisionSpec::allow(&id, &user, &session);
    denial.decision = "deny";
    denial.policy = "blocklist";
    insert_decision(&db.pool, &denial).await;
    insert_decision(
        &db.pool,
        &DecisionSpec::allow(&unique("dec"), &user, &session),
    )
    .await;

    let policies = list_top_policies(&db.pool, 3600, 10)
        .await
        .expect("query succeeds");

    assert_eq!(
        policies.len(),
        1,
        "an allow contributes nothing to this ranking"
    );
    assert_eq!(policies[0].policy, "blocklist");
    assert_eq!(policies[0].hits, 1);
    assert_eq!(policies[0].distinct_actors, 1);
    db.cleanup().await;
}
