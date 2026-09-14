use axum::Router;
use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode};
use systemprompt::evaluation::experiments::{BudgetRepository, ExperimentSpec, content_digest};
use systemprompt::evaluation::repository::experiments::{
    EvidenceRepository, ExperimentRepository, ManagedWorkspaceRegistration, RevisionRepository,
    WorkerRepository,
};
use systemprompt::identifiers::UserId;
use systemprompt_api::routes::evaluation::{EvaluationWorkerState, router};
use tower::ServiceExt;

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

async fn post(
    app: &Router,
    token: &str,
    path: &str,
    value: serde_json::Value,
) -> axum::response::Response {
    app.clone()
        .oneshot(
            Request::post(path)
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(value.to_string()))
                .unwrap(),
        )
        .await
        .unwrap()
}

#[tokio::test]
async fn evaluation_assignments_and_events_are_fenced_owner_scoped_and_immutable() {
    let db = template_test_common::TempDb::create()
        .await
        .expect("PostgreSQL required");
    let owner = UserId::new("assignment-owner");
    let stranger = UserId::new("assignment-stranger");
    for user in [&owner, &stranger] {
        sqlx::query("INSERT INTO users(id,name,email) VALUES($1,$1,$2)")
            .bind(user.as_str())
            .bind(format!("{user}@example.invalid"))
            .execute(&*db.pool)
            .await
            .unwrap();
    }
    let workers = WorkerRepository::new((*db.pool).clone());
    let worker = workers
        .create(&owner, "assignment-test", "worker")
        .await
        .unwrap();
    let foreign = workers
        .create(&stranger, "assignment-test", "foreign")
        .await
        .unwrap();
    let peer = workers
        .create(&owner, "assignment-test", "peer")
        .await
        .unwrap();
    let revisions = RevisionRepository::new((*db.pool).clone());
    let case_content = serde_json::from_value(serde_json::json!({
        "kind":"case", "content":{"prompt":"Draft grounded context", "expected_behavior":["Cite sources"],"fixtures":{},"partition":"development","assertions":["response_present"]}
    })).unwrap();
    let case = revisions
        .create(&owner, "case", &case_content)
        .await
        .unwrap();
    let rubric = revisions.create(&owner, "rubric", &serde_json::from_value(serde_json::json!({
        "kind":"rubric", "content":{"dimensions":[{"name":"grounding","description":"Cited facts","weight":1}],"pass_threshold_milli":4000,"hard_gates":[]}
    })).unwrap()).await.unwrap();
    let dataset_content: systemprompt::evaluation::experiments::resources::ResourceContent =
        serde_json::from_value(serde_json::json!({
            "kind":"dataset", "content":[case]
        }))
        .unwrap();
    let dataset = revisions
        .create(&owner, "dataset", &dataset_content)
        .await
        .unwrap();
    let (bundle, bundle_digest) = managed_workspace("bundle");
    let (candidate, candidate_digest) = managed_workspace("candidate");
    let (config, config_digest) = managed_workspace("configuration");
    let evidence = EvidenceRepository::new((*db.pool).clone());
    for (revision, manifest, digest, bytes) in [
        ("revision-bundle", &bundle, &bundle_digest, 6),
        ("revision-candidate", &candidate, &candidate_digest, 9),
        ("revision-configuration", &config, &config_digest, 13),
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
        "schema_version":1,"name":"Assignments", "cases":[case],"rubric":rubric,"dataset":dataset,
        "variants":[{"client":"claude-code","client_version":"pinned","model":"test-model","provider":"test-provider",
        "skill_bundle_digest":bundle_digest,"configuration_digest":config_digest,"worker_image_digest":"c".repeat(64)},
        {"client":"claude-code","client_version":"pinned","model":"test-model","provider":"test-provider",
        "skill_bundle_digest":candidate_digest,"configuration_digest":config_digest,"worker_image_digest":"c".repeat(64)}],
        "repetitions":1,"budget_microdollars":100,"execution_mode":"fixture","objective":"quality",
        "frozen":{"provider_prices_digest":"d".repeat(64),"tool_configuration_digest":"e".repeat(64),
          "fixture_clock":"2026-09-12T08:00:00Z","fixture_timezone":"UTC","permissions_digest":"f".repeat(64),
          "dataset_digest":content_digest(&dataset_content).unwrap(),"rubric_digest":content_digest(&revisions.get(&owner, &rubric).await.unwrap()).unwrap(),
          "cost_envelope":{"maximum_attempts_per_execution":1,"generation_microdollars_per_attempt":25,
            "judging_microdollars_per_attempt":25,"tool_microdollars_per_attempt":0,"suggestion_calls":0,
            "suggestion_microdollars_per_call":0,"auxiliary_calls":0,"auxiliary_microdollars_per_call":0}}
    })).unwrap();
    let experiments = ExperimentRepository::new((*db.pool).clone());
    let budget = BudgetRepository::new((*db.pool).clone())
        .create_shared(&owner, "assignment-budget", 100)
        .await
        .unwrap();
    let experiment = experiments
        .create_with_budget(&owner, "assignment", &budget, &spec)
        .await
        .unwrap();
    let execution = experiments
        .claim(&owner, &worker.id)
        .await
        .unwrap()
        .unwrap();
    let lease = serde_json::json!({"execution_id":execution.id,"worker_id":worker.id,"fencing_token":execution.fencing_token});
    let app = router(
        EvaluationWorkerState::builder((*db.pool).clone())
            .environment("assignment-test".into())
            .build()
            .unwrap(),
    );
    let token = worker.expose_token();
    let response = post(&app, token, "/assignment", lease.clone()).await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()["cache-control"], "no-store");
    let body: serde_json::Value =
        serde_json::from_slice(&to_bytes(response.into_body(), 1_000_000).await.unwrap()).unwrap();
    assert_eq!(body["skill_bundle"]["manifest"], bundle);
    assert_eq!(body["configuration"]["manifest"], config);
    assert_eq!(body["case"], serde_json::to_value(&case_content).unwrap());
    for unauthorized in [foreign.expose_token(), peer.expose_token(), "invalid"] {
        assert_eq!(
            post(&app, unauthorized, "/assignment", lease.clone())
                .await
                .status(),
            StatusCode::UNAUTHORIZED
        );
    }
    let mut forged = lease.clone();
    forged["worker_id"] = serde_json::json!(foreign.id);
    assert_eq!(
        post(&app, foreign.expose_token(), "/assignment", forged)
            .await
            .status(),
        StatusCode::CONFLICT
    );
    let mut stale = lease.clone();
    stale["fencing_token"] = serde_json::json!(execution.fencing_token + 1);
    assert_eq!(
        post(&app, token, "/assignment", stale.clone())
            .await
            .status(),
        StatusCode::CONFLICT
    );
    let event = serde_json::json!({"lease":lease,"event":{"sequence":0,"stage":"context","summary":"Sources retrieved"}});
    assert_eq!(
        post(&app, token, "/events", event.clone()).await.status(),
        StatusCode::NO_CONTENT
    );
    assert_eq!(
        post(&app, token, "/events", event.clone()).await.status(),
        StatusCode::NO_CONTENT
    );
    let mut changed = event.clone();
    changed["event"]["summary"] = "Changed".into();
    assert_eq!(
        post(&app, token, "/events", changed).await.status(),
        StatusCode::CONFLICT
    );
    let mut skipped = event.clone();
    skipped["event"]["sequence"] = 2.into();
    assert_eq!(
        post(&app, token, "/events", skipped).await.status(),
        StatusCode::CONFLICT
    );
    let mut stale_event = event.clone();
    stale_event["lease"] = stale;
    assert_eq!(
        post(&app, token, "/events", stale_event).await.status(),
        StatusCode::CONFLICT
    );
    let mut approval = event.clone();
    approval["event"]["approved"] = true.into();
    assert_eq!(
        post(&app, token, "/events", approval).await.status(),
        StatusCode::UNPROCESSABLE_ENTITY
    );
    sqlx::query("UPDATE eval_resource_revisions SET digest=$2 WHERE id=$1")
        .bind(case.as_str())
        .bind("0".repeat(64))
        .execute(&*db.pool)
        .await
        .unwrap();
    assert_eq!(
        post(&app, token, "/assignment", lease.clone())
            .await
            .status(),
        StatusCode::BAD_REQUEST
    );
    sqlx::query("UPDATE eval_resource_revisions SET digest=$2 WHERE id=$1")
        .bind(case.as_str())
        .bind(body["case_digest"].as_str().unwrap())
        .execute(&*db.pool)
        .await
        .unwrap();
    sqlx::query!("ALTER TABLE eval_managed_workspace_assets DISABLE TRIGGER eval_managed_workspace_assets_immutable").execute(&*db.pool).await.unwrap();
    sqlx::query!("UPDATE eval_managed_workspace_assets SET content=$3 WHERE owner_id=$1 AND workspace_digest=$2", owner.as_str(), config_digest, b"tampered".as_slice()).execute(&*db.pool).await.unwrap();
    sqlx::query!("ALTER TABLE eval_managed_workspace_assets ENABLE TRIGGER eval_managed_workspace_assets_immutable").execute(&*db.pool).await.unwrap();
    assert_eq!(
        post(&app, token, "/assignment", lease.clone())
            .await
            .status(),
        StatusCode::BAD_REQUEST
    );
    experiments.cancel(&owner, &experiment).await.unwrap();
    assert_eq!(
        post(&app, token, "/assignment", lease.clone())
            .await
            .status(),
        StatusCode::CONFLICT
    );
    assert_eq!(
        post(&app, token, "/events", event.clone()).await.status(),
        StatusCode::CONFLICT
    );
    workers.revoke(&owner, &worker.id).await.unwrap();
    assert_eq!(
        post(&app, token, "/assignment", lease).await.status(),
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        post(&app, token, "/events", event).await.status(),
        StatusCode::UNAUTHORIZED
    );
    let count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM eval_execution_events WHERE execution_id=$1")
            .bind(execution.id.as_str())
            .fetch_one(&*db.pool)
            .await
            .unwrap();
    assert_eq!(count, 1);
    db.cleanup().await;
}
