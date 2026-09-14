use std::collections::BTreeMap;
use systemprompt::evaluation::experiments::resources::{
    CaseContent, Partition, ResourceContent, RubricContent, WeightedDimension,
};
use systemprompt::evaluation::experiments::{
    BudgetRepository, ExperimentRepository, ExperimentSpec, ReservationAdmission,
    RevisionRepository, content_digest,
};
use systemprompt::evaluation::repository::experiments::{
    EvidenceRepository, ManagedWorkspaceRegistration,
};
use systemprompt::identifiers::{AiRequestId, EvalWorkerId, UserId};

fn managed_workspace(marker: &str) -> (serde_json::Value, String) {
    let asset_digest = match marker {
        "bundle" => "1e6ed65d77d6364eeaed5a745ba5c4985ae2b700dd85d7cf7f027bdf294a33fc",
        "candidate" => "dda18a0e21ae47c53b4309434cbc02ae8bf764fa83a6defbb719431242722aa7",
        "configuration" => "b7d64a9221007dfd5390f7df6cd5b8f3ea4f82faa1237141e35ebf161f5511a1",
        _ => unreachable!("known test projection"),
    };
    let revision = format!("revision-{marker}");
    let manifest = serde_json::json!({
        "schema_version":1,"assembler_version":"managed-bundle-v1","root":revision,
        "revisions":{(revision.clone()):{
            "schema_version":1,"snapshot_id":format!("snapshot-{marker}"),"parent_id":null,
            "files":{"file.txt":{"digest":asset_digest,"bytes":marker.len(),"media_type":"text/plain","executable":false}},
            "dependencies":{}
        }},
        "assets":{(asset_digest):marker.as_bytes()}
    });
    let digest = content_digest(&manifest).unwrap();
    (manifest, digest)
}

#[tokio::test]
async fn evaluation_budget_admission_is_atomic_and_settlement_is_idempotent() {
    let db = template_test_common::TempDb::create()
        .await
        .expect("PostgreSQL required");
    let owner = UserId::new("budget-owner");
    let stranger = UserId::new("other-owner");
    for user in [&owner, &stranger] {
        sqlx::query("INSERT INTO users(id,name,email) VALUES($1,$1,$2)")
            .bind(user.as_str())
            .bind(format!("{user}@example.invalid"))
            .execute(&*db.pool)
            .await
            .unwrap();
    }
    let repository = BudgetRepository::new((*db.pool).clone());
    let account = repository
        .create_shared(&owner, "five-dollar-pilot", 5_000_000)
        .await
        .unwrap();
    assert_eq!(
        account,
        repository
            .create_shared(&owner, "five-dollar-pilot", 5_000_000)
            .await
            .unwrap()
    );
    assert!(
        repository
            .create_shared(&owner, "five-dollar-pilot", 4_000_000)
            .await
            .is_err()
    );
    let (left, right) = tokio::join!(
        repository.reserve(&owner, &account, "left", 3_000_000),
        repository.reserve(&owner, &account, "right", 3_000_000)
    );
    assert_ne!(left.is_ok(), right.is_ok());
    let ReservationAdmission::Admitted(reservation) = left.or(right).unwrap() else {
        panic!("New reservation required");
    };
    assert!(
        repository
            .reserve(&stranger, &account, "foreign", 1)
            .await
            .is_err()
    );
    let request = AiRequestId::generate();
    repository
        .settle(&owner, &reservation, &request, 1_000_000)
        .await
        .unwrap();
    repository
        .settle(&owner, &reservation, &request, 1_000_000)
        .await
        .unwrap();
    assert!(
        repository
            .settle(&owner, &reservation, &request, 2_000_000)
            .await
            .is_err()
    );
    let (reserved, settled): (i64, i64) =
        sqlx::query_as("SELECT reserved,settled FROM eval_budget_accounts WHERE id=$1")
            .bind(account.as_str())
            .fetch_one(&*db.pool)
            .await
            .unwrap();
    assert_eq!((reserved, settled), (0, 1_000_000));
    let ReservationAdmission::Admitted(overage) = repository
        .reserve(&owner, &account, "overage", 1)
        .await
        .unwrap()
    else {
        panic!("New reservation required");
    };
    assert!(matches!(
        repository
            .reserve(&owner, &account, "overage", 1)
            .await
            .unwrap(),
        ReservationAdmission::AlreadyReserved(_)
    ));
    repository
        .settle(&owner, &overage, &AiRequestId::generate(), 2)
        .await
        .unwrap();
    assert!(
        repository
            .reserve(&owner, &account, "after-overage", 1)
            .await
            .is_err()
    );
    db.cleanup().await;
}

#[tokio::test]
async fn evaluation_experiments_pin_revisions_and_reject_cross_owner_access() {
    let db = template_test_common::TempDb::create()
        .await
        .expect("PostgreSQL required");
    let owner = UserId::new("experiment-owner");
    let worker = EvalWorkerId::generate();
    let stranger = UserId::new("other-owner");
    for user in [&owner, &stranger] {
        sqlx::query("INSERT INTO users(id,name,email) VALUES($1,$1,$2)")
            .bind(user.as_str())
            .bind(format!("{user}@example.invalid"))
            .execute(&*db.pool)
            .await
            .unwrap();
    }
    let revisions = RevisionRepository::new((*db.pool).clone());
    let case = ResourceContent::Case(CaseContent {
        prompt: "Write a specification".into(),
        expected_behavior: vec!["Cite source".into()],
        fixtures: BTreeMap::new(),
        partition: Partition::Development,
        assertions: vec!["response_present".into()],
    });
    let case_id = revisions
        .create(&owner, "requirements-context", &case)
        .await
        .unwrap();
    assert_eq!(
        case_id,
        revisions
            .create(&owner, "requirements-context", &case)
            .await
            .unwrap()
    );
    assert!(revisions.get(&stranger, &case_id).await.is_err());
    let rubric = ResourceContent::Rubric(RubricContent {
        dimensions: vec![WeightedDimension {
            name: "grounding".into(),
            description: "Source-supported claims".into(),
            weight: 1,
        }],
        pass_threshold_milli: 4000,
        hard_gates: vec![],
    });
    let rubric_id = revisions
        .create(&owner, "requirements-quality", &rubric)
        .await
        .unwrap();
    let dataset_content = ResourceContent::Dataset(vec![case_id.clone()]);
    let dataset_id = revisions
        .create(&owner, "dataset", &dataset_content)
        .await
        .unwrap();
    let (baseline, baseline_digest) = managed_workspace("bundle");
    let (candidate, candidate_digest) = managed_workspace("candidate");
    let (configuration, configuration_digest) = managed_workspace("configuration");
    let evidence = EvidenceRepository::new((*db.pool).clone());
    for (revision, manifest, digest, bytes) in [
        ("revision-bundle", &baseline, &baseline_digest, 6),
        ("revision-candidate", &candidate, &candidate_digest, 9),
        (
            "revision-configuration",
            &configuration,
            &configuration_digest,
            13,
        ),
    ] {
        evidence
            .register_managed_workspace(
                &owner,
                &ManagedWorkspaceRegistration {
                    managed_revision_id: revision,
                    publication_generation: Some(1),
                    manifest,
                    expected_digest: digest,
                    file_count: 1,
                    byte_count: bytes,
                },
            )
            .await
            .unwrap();
    }
    let spec: ExperimentSpec = serde_json::from_value(serde_json::json!({
        "schema_version":1,"name":"UK POC","cases":[case_id],"rubric":rubric_id,"dataset":dataset_id,
        "variants":[
          {"client":"claude-code","client_version":"pinned","model":"claude-haiku-4-5","provider":"anthropic","skill_bundle_digest":baseline_digest,"configuration_digest":configuration_digest,"worker_image_digest":"c".repeat(64)},
          {"client":"claude-code","client_version":"pinned","model":"claude-haiku-4-5","provider":"anthropic","skill_bundle_digest":candidate_digest,"configuration_digest":configuration_digest,"worker_image_digest":"c".repeat(64)}],
        "repetitions":1,"budget_microdollars":100,"execution_mode":"fixture","objective":"quality",
        "frozen":{"provider_prices_digest":"d".repeat(64),"tool_configuration_digest":"e".repeat(64),"fixture_clock":"2026-09-12T08:00:00Z","fixture_timezone":"UTC","permissions_digest":"f".repeat(64),
          "dataset_digest":content_digest(&dataset_content).unwrap(),"rubric_digest":content_digest(&rubric).unwrap(),
          "cost_envelope":{"maximum_attempts_per_execution":1,"generation_microdollars_per_attempt":25,"judging_microdollars_per_attempt":25,"tool_microdollars_per_attempt":0,"suggestion_calls":0,"suggestion_microdollars_per_call":0,"auxiliary_calls":0,"auxiliary_microdollars_per_call":0}}
    })).unwrap();
    let experiments = ExperimentRepository::new((*db.pool).clone());
    let budgets = BudgetRepository::new((*db.pool).clone());
    let shared = budgets
        .create_shared(&owner, "shared-pilot", 5_000_000)
        .await
        .unwrap();
    let id = experiments
        .create_with_budget(&owner, "poc", &shared, &spec)
        .await
        .unwrap();
    assert_eq!(
        id,
        experiments
            .create_with_budget(&owner, "poc", &shared, &spec)
            .await
            .unwrap()
    );
    assert_eq!(
        experiments.get(&owner, &id).await.unwrap().executions.len(),
        2
    );
    assert!(experiments.get(&stranger, &id).await.is_err());
    assert!(
        experiments
            .create_with_budget(&stranger, "poc", &shared, &spec)
            .await
            .is_err()
    );
    let mut changed = spec.clone();
    changed.name = "Changed".into();
    assert!(
        experiments
            .create_with_budget(&owner, "poc", &shared, &changed)
            .await
            .is_err()
    );
    let claimed = experiments.claim(&owner, &worker).await.unwrap().unwrap();
    use systemprompt::evaluation::repository::experiments::{
        ExecutionCompletion, ExecutionLease, TerminalOutcome,
    };
    let lease = ExecutionLease {
        execution_id: claimed.id,
        worker_id: worker.clone(),
        fencing_token: claimed.fencing_token,
    };
    experiments.heartbeat(&owner, &lease).await.unwrap();
    let completion = ExecutionCompletion {
        outcome: TerminalOutcome::Completed,
        summary: "Fixture completed; semantic judgment remains separate".into(),
    };
    let mut stale = lease.clone();
    stale.fencing_token += 1;
    assert!(
        experiments
            .complete(&owner, &stale, &completion)
            .await
            .is_err()
    );
    experiments
        .complete(&owner, &lease, &completion)
        .await
        .unwrap();
    experiments
        .complete(&owner, &lease, &completion)
        .await
        .unwrap();
    assert!(experiments.heartbeat(&owner, &lease).await.is_err());
    let paired = experiments.claim(&owner, &worker).await.unwrap().unwrap();
    let paired_lease = ExecutionLease {
        execution_id: paired.id,
        worker_id: worker.clone(),
        fencing_token: paired.fencing_token,
    };
    experiments
        .complete(&owner, &paired_lease, &completion)
        .await
        .unwrap();
    crate::optimization_reports::assert_retained_measurement_report(&db.pool, &owner, &id, &shared)
        .await;
    let another = experiments
        .create_with_budget(&owner, "cancel-test", &shared, &spec)
        .await
        .unwrap();
    experiments.cancel(&owner, &another).await.unwrap();
    assert!(!budgets.get(&owner, &shared).await.unwrap().frozen);
    budgets
        .reserve(&owner, &shared, "unrelated-after-cancel", 1)
        .await
        .unwrap();
    assert!(experiments.claim(&owner, &worker).await.unwrap().is_none());
    let expired_id = experiments
        .create_with_budget(&owner, "expiry-test", &shared, &spec)
        .await
        .unwrap();
    let expired = experiments.claim(&owner, &worker).await.unwrap().unwrap();
    let expired_pair = experiments.claim(&owner, &worker).await.unwrap().unwrap();
    sqlx::query(
        "UPDATE eval_executions SET lease_expires_at=NOW()-INTERVAL '1 second' WHERE id=$1",
    )
    .bind(expired.id.as_str())
    .execute(&*db.pool)
    .await
    .unwrap();
    sqlx::query!(
        "UPDATE eval_executions SET lease_expires_at=NOW()-INTERVAL '1 second' WHERE id=$1",
        expired_pair.id.as_str()
    )
    .execute(&*db.pool)
    .await
    .unwrap();
    assert!(experiments.claim(&owner, &worker).await.unwrap().is_none());
    let expired_detail = experiments.get(&owner, &expired_id).await.unwrap();
    assert_eq!(
        serde_json::to_value(expired_detail.experiment.status).unwrap(),
        "completed"
    );
    assert_eq!(
        serde_json::to_value(expired_detail.executions[0].status).unwrap(),
        "error"
    );
    let mut concurrent = spec.clone();
    concurrent.repetitions = 2;
    concurrent.budget_microdollars = 200;
    let concurrent_id = experiments
        .create_with_budget(&owner, "concurrent-completion", &shared, &concurrent)
        .await
        .unwrap();
    let left = experiments.claim(&owner, &worker).await.unwrap().unwrap();
    let right = experiments.claim(&owner, &worker).await.unwrap().unwrap();
    experiments
        .create_with_budget(&owner, "waiting-for-capacity", &shared, &spec)
        .await
        .unwrap();
    assert!(experiments.claim(&owner, &worker).await.unwrap().is_none());
    let left_lease = ExecutionLease {
        execution_id: left.id,
        worker_id: worker.clone(),
        fencing_token: left.fencing_token,
    };
    let right_lease = ExecutionLease {
        execution_id: right.id,
        worker_id: worker.clone(),
        fencing_token: right.fencing_token,
    };
    let (left_result, right_result) = tokio::join!(
        experiments.complete(&owner, &left_lease, &completion),
        experiments.complete(&owner, &right_lease, &completion)
    );
    left_result.unwrap();
    right_result.unwrap();
    for _ in 0..2 {
        let remaining = experiments.claim(&owner, &worker).await.unwrap().unwrap();
        let remaining_lease = ExecutionLease {
            execution_id: remaining.id,
            worker_id: worker.clone(),
            fencing_token: remaining.fencing_token,
        };
        experiments
            .complete(&owner, &remaining_lease, &completion)
            .await
            .unwrap();
    }
    assert_eq!(
        serde_json::to_value(
            experiments
                .get(&owner, &concurrent_id)
                .await
                .unwrap()
                .experiment
                .status
        )
        .unwrap(),
        "completed"
    );
    let timed = experiments.claim(&owner, &worker).await.unwrap().unwrap();
    let timed_lease = ExecutionLease {
        execution_id: timed.id,
        worker_id: worker,
        fencing_token: timed.fencing_token,
    };
    sqlx::query("UPDATE eval_executions SET deadline_at=NOW()-INTERVAL '1 second' WHERE id=$1")
        .bind(timed_lease.execution_id.as_str())
        .execute(&*db.pool)
        .await
        .unwrap();
    assert!(experiments.heartbeat(&owner, &timed_lease).await.is_err());
    assert!(
        experiments
            .complete(&owner, &timed_lease, &completion)
            .await
            .is_err()
    );
    db.cleanup().await;
}
