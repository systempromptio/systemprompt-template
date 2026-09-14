use std::collections::BTreeMap;
use systemprompt::evaluation::experiments::resources::{
    CaseContent, Partition, ResourceContent, RubricContent, WeightedDimension,
};
use systemprompt::evaluation::experiments::{
    BudgetRepository, ExperimentRepository, ExperimentSpec, RevisionRepository, content_digest,
};
use systemprompt::evaluation::repository::experiments::{
    AdmissionRequest, EvidenceRepository, ExecutionCapabilityRepository, ExecutionLease,
    GatewayEvaluationRepository, ManagedWorkspaceRegistration, RequestAdmission, WorkerRepository,
};
use systemprompt::identifiers::{AiRequestId, ModelId, ProviderId, SessionId, UserId};

fn managed_workspace(marker: &str) -> (serde_json::Value, String) {
    let bytes = marker.as_bytes();
    let asset_digest = match marker {
        "bundle" => "1e6ed65d77d6364eeaed5a745ba5c4985ae2b700dd85d7cf7f027bdf294a33fc",
        "candidate" => "dda18a0e21ae47c53b4309434cbc02ae8bf764fa83a6defbb719431242722aa7",
        "configuration" => "b7d64a9221007dfd5390f7df6cd5b8f3ea4f82faa1237141e35ebf161f5511a1",
        _ => unreachable!("known test projection"),
    };
    let revision = format!("revision-{marker}");
    let manifest = serde_json::json!({
        "schema_version":1,
        "assembler_version":"managed-bundle-v1",
        "root":revision,
        "revisions":{(revision.clone()):{
            "schema_version":1,"snapshot_id":format!("snapshot-{marker}"),"parent_id":null,
            "files":{"file.txt":{"digest":asset_digest,"bytes":bytes.len(),"media_type":"text/plain","executable":false}},
            "dependencies":{}
        }},
        "assets":{(asset_digest):bytes}
    });
    let digest = content_digest(&manifest).unwrap();
    (manifest, digest)
}

#[tokio::test]
async fn attested_evaluation_sessions_cannot_escape_budget_or_settle_incomplete_usage() {
    let db = template_test_common::TempDb::create()
        .await
        .expect("PostgreSQL required");
    let owner = UserId::new("gateway-evaluation-owner");
    let session = SessionId::generate();
    sqlx::query(
        "INSERT INTO users(id,name,email) VALUES($1,'evaluator','eval-test@example.invalid')",
    )
    .bind(owner.as_str())
    .execute(&*db.pool)
    .await
    .unwrap();
    sqlx::query("INSERT INTO user_sessions(session_id,user_id) VALUES($1,$2)")
        .bind(session.as_str())
        .bind(owner.as_str())
        .execute(&*db.pool)
        .await
        .unwrap();
    let workers = WorkerRepository::new((*db.pool).clone());
    let worker = workers
        .create(&owner, "local-evaluation", "gateway-test")
        .await
        .unwrap()
        .id;
    let revisions = RevisionRepository::new((*db.pool).clone());
    let case = revisions
        .create(
            &owner,
            "case",
            &ResourceContent::Case(CaseContent {
                prompt: "Draft context".into(),
                expected_behavior: vec!["Cite the source".into()],
                fixtures: BTreeMap::new(),
                partition: Partition::Development,
                assertions: vec!["response_present".into()],
            }),
        )
        .await
        .unwrap();
    let rubric = revisions
        .create(
            &owner,
            "rubric",
            &ResourceContent::Rubric(RubricContent {
                dimensions: vec![WeightedDimension {
                    name: "grounding".into(),
                    description: "Cited facts".into(),
                    weight: 1,
                }],
                pass_threshold_milli: 4000,
                hard_gates: vec![],
            }),
        )
        .await
        .unwrap();
    let dataset_content = ResourceContent::Dataset(vec![case.clone()]);
    let dataset = revisions
        .create(&owner, "dataset", &dataset_content)
        .await
        .unwrap();
    let (baseline_manifest, baseline_digest) = managed_workspace("bundle");
    let (candidate_manifest, candidate_digest) = managed_workspace("candidate");
    let (configuration_manifest, configuration_digest) = managed_workspace("configuration");
    let managed = EvidenceRepository::new((*db.pool).clone());
    for (revision, manifest, digest, bytes) in [
        ("revision-bundle", &baseline_manifest, &baseline_digest, 6),
        (
            "revision-candidate",
            &candidate_manifest,
            &candidate_digest,
            9,
        ),
        (
            "revision-configuration",
            &configuration_manifest,
            &configuration_digest,
            13,
        ),
    ] {
        managed
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
        "schema_version":1,"name":"Gateway budget", "cases":[case], "rubric":rubric, "dataset":dataset,
        "variants":[{"client":"claude-code","client_version":"pinned","model":"test-model","provider":"test-provider",
          "skill_bundle_digest":baseline_digest,"configuration_digest":configuration_digest,"worker_image_digest":"c".repeat(64)},
          {"client":"claude-code","client_version":"pinned","model":"test-model","provider":"test-provider",
          "skill_bundle_digest":candidate_digest,"configuration_digest":configuration_digest,"worker_image_digest":"c".repeat(64)}],
        "repetitions":1,"budget_microdollars":100,"execution_mode":"fixture","objective":"quality",
        "frozen":{"provider_prices_digest":"d".repeat(64),"tool_configuration_digest":"e".repeat(64),
          "fixture_clock":"2026-09-12T08:00:00Z","fixture_timezone":"UTC","permissions_digest":"f".repeat(64),
          "dataset_digest":content_digest(&dataset_content).unwrap(),
          "rubric_digest":content_digest(&revisions.get(&owner, &rubric).await.unwrap()).unwrap(),
          "cost_envelope":{"maximum_attempts_per_execution":1,"generation_microdollars_per_attempt":25,
            "judging_microdollars_per_attempt":25,"tool_microdollars_per_attempt":0,"suggestion_calls":0,
            "suggestion_microdollars_per_call":0,"auxiliary_calls":0,"auxiliary_microdollars_per_call":0}}
    })).unwrap();
    let experiments = ExperimentRepository::new((*db.pool).clone());
    let budget = BudgetRepository::new((*db.pool).clone())
        .create_shared(&owner, "gateway-budget", 100)
        .await
        .unwrap();
    let id = experiments
        .create_with_budget(&owner, "budget-case", &budget, &spec)
        .await
        .unwrap();
    let execution = experiments.claim(&owner, &worker).await.unwrap().unwrap();
    let lease = ExecutionLease {
        execution_id: execution.id,
        worker_id: worker.clone(),
        fencing_token: execution.fencing_token,
    };
    let capabilities = ExecutionCapabilityRepository::new((*db.pool).clone());
    let access = capabilities.issue(&owner, &lease).await.unwrap();
    assert!(!format!("{access:?}").contains(access.expose_token()));
    let principal = capabilities
        .authenticate(access.expose_token(), "local-evaluation")
        .await
        .unwrap();
    assert_eq!(principal.identity.owner_id, owner);
    assert_eq!(principal.identity.execution_id, lease.execution_id);
    assert!(
        capabilities
            .authenticate(access.expose_token(), "foreign-environment")
            .await
            .is_err()
    );
    assert!(
        workers
            .authenticate(access.expose_token(), "local-evaluation")
            .await
            .is_err()
    );
    sqlx::query("UPDATE eval_execution_capabilities SET expires_at=NOW()-INTERVAL '1 second' WHERE execution_id=$1")
        .bind(lease.execution_id.as_str()).execute(&*db.pool).await.unwrap();
    assert!(
        capabilities
            .authenticate(access.expose_token(), "local-evaluation")
            .await
            .is_err()
    );
    assert!(
        capabilities
            .issue(
                &owner,
                &ExecutionLease {
                    fencing_token: lease.fencing_token + 1,
                    ..lease.clone()
                }
            )
            .await
            .is_err()
    );
    let refreshed = capabilities.issue(&owner, &lease).await.unwrap();
    assert_eq!(access.session_id, refreshed.session_id);
    assert!(
        capabilities
            .authenticate(access.expose_token(), "local-evaluation")
            .await
            .is_err()
    );
    assert!(
        capabilities
            .authenticate(refreshed.expose_token(), "local-evaluation")
            .await
            .is_ok()
    );
    let gateway = GatewayEvaluationRepository::new((*db.pool).clone());
    gateway
        .bind_session(&owner, &lease, &session)
        .await
        .unwrap();
    assert_eq!(
        gateway.execution_actor(&owner, &session).await.unwrap(),
        Some(systemprompt::identifiers::Actor::job(
            owner.clone(),
            format!("evaluation:{}", lease.execution_id)
        ))
    );
    assert!(
        gateway
            .execution_actor(&UserId::new("foreign-owner"), &session)
            .await
            .unwrap()
            .is_none()
    );
    let request = AiRequestId::generate();
    sqlx::query("INSERT INTO ai_requests(id,request_id,user_id,session_id,context_id,provider,model,actor_kind,actor_id) VALUES($1,$1,$2,$3,'test-context','test-provider','test-model','user',$2)")
        .bind(request.as_str()).bind(owner.as_str()).bind(session.as_str()).execute(&*db.pool).await.unwrap();
    let model = ModelId::new("test-model");
    let provider = ProviderId::new("test-provider");
    let admission = AdmissionRequest {
        owner: &owner,
        session: &session,
        request: &request,
        model: &model,
        provider: &provider,
        bound_microdollars: 80,
    };
    let foreign_provider = ProviderId::new("foreign-provider");
    assert!(
        gateway
            .admit(&AdmissionRequest {
                provider: &foreign_provider,
                ..admission
            })
            .await
            .is_err()
    );
    assert!(matches!(
        gateway.admit(&admission).await.unwrap(),
        RequestAdmission::Reserved(_)
    ));
    assert!(gateway.admit(&admission).await.is_err());
    let second = AiRequestId::generate();
    sqlx::query("INSERT INTO ai_requests(id,request_id,user_id,session_id,context_id,provider,model,actor_kind,actor_id) VALUES($1,$1,$2,$3,'test-context','test-provider','test-model','user',$2)")
        .bind(second.as_str()).bind(owner.as_str()).bind(session.as_str()).execute(&*db.pool).await.unwrap();
    assert!(matches!(
        gateway
            .admit(&AdmissionRequest {
                request: &second,
                ..admission
            })
            .await,
        Err(systemprompt::evaluation::EvaluationError::BudgetExhausted { .. })
    ));
    let wrong = ModelId::new("other-model");
    assert!(
        gateway
            .admit(&AdmissionRequest {
                model: &wrong,
                bound_microdollars: 1,
                ..admission
            })
            .await
            .is_err()
    );
    assert!(gateway.settle_recorded(&owner, &request).await.is_err());
    assert_eq!(
        experiments
            .get(&owner, &id)
            .await
            .unwrap()
            .experiment
            .accounting
            .reserved,
        80
    );
    sqlx::query("UPDATE ai_requests SET status='completed',cost_microdollars=30,tokens_used=1,completed_at=NOW() WHERE id=$1")
        .bind(request.as_str()).execute(&*db.pool).await.unwrap();
    assert!(gateway.settle_recorded(&owner, &request).await.unwrap());
    assert!(gateway.settle_recorded(&owner, &request).await.unwrap());
    let account = experiments
        .get(&owner, &id)
        .await
        .unwrap()
        .experiment
        .accounting;
    assert_eq!((account.reserved, account.settled), (0, 30));
    let evidence_repo = systemprompt::evaluation::repository::experiments::EvidenceRepository::new(
        (*db.pool).clone(),
    );
    let artifacts = systemprompt::evaluation::experiments::execution::EvidenceArchive {
        files: BTreeMap::from([(
            "response.md".into(),
            systemprompt::evaluation::experiments::execution::ArtifactFile {
                bytes: b"hello".to_vec(),
                executable: false,
            },
        )]),
    };
    let mut evidence: systemprompt::evaluation::experiments::execution::ExecutionEvidence = serde_json::from_value(serde_json::json!({
        "execution_id": lease.execution_id, "fencing_token": lease.fencing_token,
        "capabilities": {"client":"claude-code", "client_version":"pinned", "adapter_version":"1", "image_digest":"c".repeat(64),"supports_session_resume":false},
        "installed_bundle_digest":baseline_digest,"candidate_bundle_digest":baseline_digest,
        "workspace_digest": artifacts.digest().unwrap(), "requests":[request],
        "artifacts":[{"relative_path":"response.md","sha256":"2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824","bytes":5}],
        "exit_code":0, "elapsed_milliseconds":1, "cleanup_confirmed":true
    })).unwrap();
    let mut tampered = artifacts.clone();
    tampered.files.insert(
        "response.md".into(),
        systemprompt::evaluation::experiments::execution::ArtifactFile {
            bytes: b"wrong".to_vec(),
            executable: false,
        },
    );
    assert!(
        evidence_repo
            .submit(&owner, &lease, &evidence, &tampered)
            .await
            .is_err()
    );
    evidence.requests.push(second.clone());
    assert!(
        evidence_repo
            .submit(&owner, &lease, &evidence, &artifacts)
            .await
            .is_err()
    );
    evidence.requests.pop();
    evidence_repo
        .submit(&owner, &lease, &evidence, &artifacts)
        .await
        .unwrap();
    evidence_repo
        .submit(&owner, &lease, &evidence, &artifacts)
        .await
        .unwrap();
    let stored = evidence_repo
        .get_artifacts(&owner, &lease.execution_id)
        .await
        .unwrap();
    assert_eq!(stored.files.len(), artifacts.files.len());
    for (path, expected) in &artifacts.files {
        let actual = stored.files.get(path).expect("artifact path retained");
        assert_eq!(actual.bytes, expected.bytes);
        assert_eq!(actual.executable, expected.executable);
    }
    assert!(
        evidence_repo
            .get_artifacts(&UserId::new("foreign-owner"), &lease.execution_id)
            .await
            .is_err()
    );
    evidence.elapsed_milliseconds += 1;
    assert!(
        evidence_repo
            .submit(&owner, &lease, &evidence, &artifacts)
            .await
            .is_err()
    );
    workers.revoke(&owner, &worker).await.unwrap();
    assert!(
        capabilities
            .authenticate(refreshed.expose_token(), "local-evaluation")
            .await
            .is_err()
    );
    assert!(capabilities.issue(&owner, &lease).await.is_err());
    assert!(
        gateway
            .admit(&AdmissionRequest {
                request: &second,
                bound_microdollars: 1,
                ..admission
            })
            .await
            .is_err()
    );
    experiments.cancel(&owner, &id).await.unwrap();
    assert!(
        gateway
            .admit(&AdmissionRequest {
                request: &second,
                bound_microdollars: 1,
                ..admission
            })
            .await
            .is_err()
    );
    db.cleanup().await;
}
