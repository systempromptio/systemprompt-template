//! The shipped gateway policy under warn mode, through the real ingestion and
//! resolution path: `services/gateway/policies.yaml` is parsed by the loader
//! the boot job uses, written to `ai_gateway_policies` in a throwaway
//! database, and read back through the gateway's `PolicyResolver` — the merge
//! that decides what the dispatch path actually enforces. Both `quota_mode`
//! and `safety.mode` have to come out the far side as `warn`, or the file
//! says one thing and the gateway does another.
//!
//! The quota plane gets its own proof because it is the one that had no warn
//! mode until now: the resolved windows are driven to exhaustion with the
//! real reservation code, which must still report the breach (the report
//! needs it) while the mode says it is not a refusal.

use std::sync::Arc;

use systemprompt::ai::repository::{AiGatewayPolicyRepository, AiQuotaBucketRepository};
use systemprompt::ai::{
    GatewayPolicyConfig, GatewayPolicyIngestOptions, GatewayPolicyIngestionService, QuotaMode,
    QuotaWindow, SafetyMode,
};
use systemprompt::api::services::gateway::policy::PolicyResolver;
use systemprompt::api::services::gateway::quota::precheck_and_reserve;
use systemprompt::database::{Database, DbPool};
use systemprompt::identifiers::UserId;

use crate::fixtures::unique;
use crate::tempdb::TempDb;

fn shipped_policies() -> GatewayPolicyConfig {
    let path = template_test_common::repo_path("services/gateway/policies.yaml");
    let yaml =
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    serde_yaml::from_str(&yaml).unwrap_or_else(|e| panic!("parse {}: {e}", path.display()))
}

fn db_pool(db: &TempDb) -> DbPool {
    Arc::new(Database::from_pools(
        Arc::clone(&db.pool),
        Some(Arc::clone(&db.pool)),
    ))
}

async fn ingest_shipped(db: &TempDb) -> DbPool {
    let pool = db_pool(db);
    let repo = AiGatewayPolicyRepository::new(&pool).expect("policy repository");
    let report = GatewayPolicyIngestionService::from_repository(repo)
        .ingest_config(
            &shipped_policies(),
            GatewayPolicyIngestOptions {
                override_existing: true,
                delete_orphans: false,
            },
        )
        .await
        .expect("the shipped policy ingests");
    assert!(
        report.inserted + report.updated >= 1,
        "at least the default-quotas row landed: {report:?}"
    );
    pool
}

#[test]
fn the_shipped_file_declares_warn_on_both_gateway_planes() {
    let cfg = shipped_policies();
    cfg.validate().expect("the shipped file validates");
    let spec = &cfg
        .policies
        .iter()
        .find(|p| p.enabled)
        .expect("an enabled policy")
        .spec;
    assert_eq!(spec.quota_mode, QuotaMode::Warn);
    assert_eq!(spec.safety.mode, SafetyMode::Warn);
    assert!(
        !spec.quota_windows.is_empty(),
        "the windows stay declared under warn — they are the hypothesis being measured"
    );
    assert!(
        !spec.safety.block_categories.is_empty()
            && !spec.safety.block_response_categories.is_empty(),
        "the block lists stay declared under warn for the same reason"
    );
}

#[tokio::test]
async fn the_resolver_keeps_both_warn_switches_after_ingestion() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let pool = ingest_shipped(&db).await;

    let repo = AiGatewayPolicyRepository::new(&pool).expect("policy repository");
    let resolved = PolicyResolver::from_repository(repo)
        .resolve(systemprompt::models::services::QuotaFaultMode::default())
        .await
        .expect("policy row resolves");

    assert!(
        resolved.quota_mode.is_warn(),
        "quota_mode survived the row merge; the quota stage would otherwise answer 429"
    );
    assert!(
        resolved.safety.mode.is_warn(),
        "safety.mode survived the row merge; the scanners would otherwise refuse"
    );
    assert!(
        !resolved.quota_windows.is_empty() && !resolved.safety.scanners.is_empty(),
        "warn did not empty the windows or the scanner list: {resolved:?}"
    );

    db.cleanup().await;
}

// Why: the quota stage is `precheck_and_reserve` followed by a refusal the
// dispatch path now skips under `quota_mode: warn`. The reservation must
// still happen and the breach must still be reported — a warn row with no
// message would be useless to the report — so the exhausted state is driven
// with the real windows and the real bucket table.
#[tokio::test]
async fn an_exhausted_window_is_still_detected_so_the_warn_carries_a_reason() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let pool = ingest_shipped(&db).await;
    let repo = AiGatewayPolicyRepository::new(&pool).expect("policy repository");
    let resolved = PolicyResolver::from_repository(repo)
        .resolve(systemprompt::models::services::QuotaFaultMode::default())
        .await
        .expect("policy row resolves");
    assert!(resolved.quota_mode.is_warn());

    let user_window = resolved
        .quota_windows
        .iter()
        .find(|w| w.subject == "user")
        .expect("the shipped per-user window");
    let max_requests = user_window
        .max_requests
        .expect("the per-user window caps requests");
    // Why: a one-request window on the same subject kind and seconds, so the
    // test exhausts a real user bucket in two calls rather than six hundred.
    let tiny = QuotaWindow {
        max_requests: Some(1),
        ..user_window.clone()
    };
    let buckets = AiQuotaBucketRepository::new(&pool).expect("bucket repository");
    let user = UserId::new(unique("quota-user"));

    let first = precheck_and_reserve(
        &pool,
        &buckets,
        &user,
        std::slice::from_ref(&tiny),
        systemprompt::models::services::QuotaFaultMode::default(),
    )
    .await
    .expect("first reservation");
    assert!(first.is_none(), "the first request is within the window");

    let second = precheck_and_reserve(
        &pool,
        &buckets,
        &user,
        std::slice::from_ref(&tiny),
        systemprompt::models::services::QuotaFaultMode::default(),
    )
    .await
    .expect("second reservation")
    .expect("the second request breaches a one-request window");
    assert!(
        !second.allow,
        "the breach is detected exactly as under enforce"
    );
    // Why: core words this "<label> ceiling exceeded for <subject> window <n>s
    // (used x/y)" — `request`, `input token` or `output token` for the label —
    // so the assertion pins the subject and the ceiling clause rather than the
    // label, which varies with whichever limit the window tripped.
    assert!(
        second.message.contains("ceiling exceeded") && second.message.contains("user"),
        "the reason names the subject and the ceiling: {}",
        second.message
    );
    assert_eq!(second.window_seconds, user_window.window_seconds);
    assert!(
        max_requests > 1,
        "the shipped ceiling is a real backstop, not this test's 1"
    );

    db.cleanup().await;
}
