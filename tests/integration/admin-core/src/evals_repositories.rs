//! `repositories::evals::{runs, results, cases, sampling}` — the eval spine
//! CRUD against core's superset schema, and the sampling rule that judge
//! traffic (`actor_kind = 'job'`) is never a candidate.

use sqlx::PgPool;
use sqlx::types::Json;
use systemprompt::identifiers::UserId;
use systemprompt_web_admin::repositories::evals::cases::{
    InsertCaseParams, delete_case, insert_case, list_cases,
};
use systemprompt_web_admin::repositories::evals::results::{
    DimensionScores, InsertResultParams, insert_result, list_results_for_run,
};
use systemprompt_web_admin::repositories::evals::runs::{
    CompleteRunParams, EvalRunFilterSnapshot, InsertRunParams, find_run, insert_run,
    list_recent_runs, update_run_completion,
};
use systemprompt_web_admin::repositories::evals::sampling::{
    CandidateFilter, list_eval_candidates,
};
use systemprompt_web_admin::repositories::evals::{EvalRunKind, EvalRunStatus, EvalVerdict};

use crate::fixtures::{narrow_window, unclaimed_email, unique, insert_user};
use crate::tempdb::TempDb;

fn filter_snapshot() -> Json<EvalRunFilterSnapshot> {
    let range = narrow_window();
    Json(EvalRunFilterSnapshot {
        from: range.from,
        to: range.to,
        user_id: None,
        model: None,
        provider: None,
        compare_models: Vec::new(),
    })
}

async fn insert_gateway_request(pool: &PgPool, id: &str, user: &UserId, actor_kind: &str) {
    sqlx::query(
        "INSERT INTO ai_requests (
             id, request_id, user_id, provider, model, input_tokens, output_tokens,
             tokens_used, cost_microdollars, latency_ms, status, actor_kind, actor_id,
             created_at, updated_at)
         VALUES ($1, $1, $2, 'anthropic', 'claude-eval-model', 10, 5, 15, 100, 50,
                 'completed', $3, $2, NOW(), NOW())",
    )
    .bind(id)
    .bind(user.as_str())
    .bind(actor_kind)
    .execute(pool)
    .await
    .expect("insert ai_request");
    sqlx::query(
        "INSERT INTO ai_request_payloads (
             ai_request_id, request_body, response_body, request_excerpt,
             response_excerpt, request_truncated, response_truncated)
         VALUES ($1, '{}'::jsonb, '{}'::jsonb, 'prompt', 'answer', FALSE, FALSE)",
    )
    .bind(id)
    .execute(pool)
    .await
    .expect("insert ai_request_payload");
}

#[tokio::test]
async fn run_lifecycle_roundtrips_through_completion() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("evalrun")).await;
    let run_id = unique("evrun");

    insert_run(
        &db.pool,
        InsertRunParams {
            id: &run_id,
            kind: EvalRunKind::Judge,
            judge_provider: "anthropic",
            judge_model: "claude-judge",
            filter: filter_snapshot(),
            sample_size: 5,
            created_by: user.as_str(),
        },
    )
    .await
    .expect("insert run");

    let running = find_run(&db.pool, &run_id)
        .await
        .expect("find run")
        .expect("run exists");
    assert_eq!(running.status, "running");
    assert_eq!(running.kind, "judge");
    assert_eq!(running.sample_size, 5);

    update_run_completion(
        &db.pool,
        CompleteRunParams {
            id: &run_id,
            status: EvalRunStatus::Completed,
            scored_count: 4,
            failed_count: 1,
            cost_microdollars: 12_345,
            error_message: None,
        },
    )
    .await
    .expect("complete run");

    let done = find_run(&db.pool, &run_id)
        .await
        .expect("find run")
        .expect("run exists");
    assert_eq!(done.status, "completed");
    assert_eq!(done.scored_count, 4);
    assert_eq!(done.failed_count, 1);
    assert_eq!(done.cost_microdollars, 12_345);
    assert!(done.completed_at.is_some());

    let listed = list_recent_runs(&db.pool, narrow_window(), 10)
        .await
        .expect("list runs");
    assert!(listed.iter().any(|r| r.id == run_id));
    db.cleanup().await;
}

#[tokio::test]
async fn result_rows_roundtrip_with_dimension_scores() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("evalres")).await;
    let run_id = unique("evrun");
    insert_run(
        &db.pool,
        InsertRunParams {
            id: &run_id,
            kind: EvalRunKind::Judge,
            judge_provider: "anthropic",
            judge_model: "claude-judge",
            filter: filter_snapshot(),
            sample_size: 1,
            created_by: user.as_str(),
        },
    )
    .await
    .expect("insert run");

    let request_id = unique("req");
    insert_gateway_request(&db.pool, &request_id, &user, "user").await;
    let result_id = unique("evres");
    insert_result(
        &db.pool,
        InsertResultParams {
            id: &result_id,
            run_id: &run_id,
            ai_request_id: Some(&request_id),
            case_id: None,
            user_id: Some(&user),
            session_id: None,
            provider: "anthropic",
            model: "claude-eval-model",
            overall_score: Some(2),
            dimension_scores: Json(DimensionScores {
                instruction_following: Some(2),
                correctness: Some(3),
                completeness: Some(1),
                format: Some(4),
                safety: None,
            }),
            verdict: EvalVerdict::Fail,
            rationale: Some("truncated answer"),
            flags: &[],
            prompt_excerpt: Some("prompt"),
            response_excerpt: Some("answer"),
            latency_ms: Some(120),
            cost_microdollars: 100,
            judge_cost_microdollars: 900,
        },
    )
    .await
    .expect("insert result");

    let rows = list_results_for_run(&db.pool, &run_id, 10)
        .await
        .expect("list results");
    assert_eq!(rows.len(), 1);
    let row = &rows[0];
    assert_eq!(row.id, result_id);
    assert_eq!(row.verdict, "fail");
    assert_eq!(row.overall_score, Some(2));
    assert_eq!(row.dimension_scores.correctness, Some(3));
    assert_eq!(row.ai_request_id.as_deref(), Some(request_id.as_str()));
    db.cleanup().await;
}

#[tokio::test]
async fn cases_insert_list_and_delete() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("evalcase")).await;
    let case_id = unique("evcase");
    insert_case(
        &db.pool,
        InsertCaseParams {
            id: &case_id,
            name: "golden greeting",
            prompt_body: serde_json::json!({"messages": [{"role": "user", "content": "hi"}]}),
            source_ai_request_id: None,
            expectation: Some("a greeting"),
            baseline_response: None,
            baseline_model: Some("claude-eval-model"),
            tags: &["smoke".to_owned()],
            created_by: user.as_str(),
        },
    )
    .await
    .expect("insert case");

    let cases = list_cases(&db.pool, true).await.expect("list cases");
    let case = cases
        .iter()
        .find(|c| c.id == case_id)
        .expect("inserted case listed");
    assert_eq!(case.name, "golden greeting");
    assert!(case.enabled);

    delete_case(&db.pool, &case_id).await.expect("delete case");
    let cases = list_cases(&db.pool, false).await.expect("list cases");
    assert!(cases.iter().all(|c| c.id != case_id));
    db.cleanup().await;
}

#[tokio::test]
async fn sampling_never_returns_judge_traffic() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("evalsamp")).await;
    let user_req = unique("req-user");
    let job_req = unique("req-job");
    insert_gateway_request(&db.pool, &user_req, &user, "user").await;
    insert_gateway_request(&db.pool, &job_req, &user, "job").await;

    let candidates =
        list_eval_candidates(&db.pool, &CandidateFilter::default(), narrow_window(), 50)
            .await
            .expect("list candidates");

    let ids: Vec<&str> = candidates
        .iter()
        .map(|c| c.ai_request_id.as_str())
        .collect();
    assert!(ids.contains(&user_req.as_str()), "user traffic is sampled");
    assert!(
        !ids.contains(&job_req.as_str()),
        "judge/job traffic must never be sampled"
    );
    db.cleanup().await;
}
