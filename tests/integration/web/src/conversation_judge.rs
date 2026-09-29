//! Integration coverage for the injectable production judge runner.
//! The database, discovery, leases and persistence are real; only inference is
//! a deterministic local collaborator.

use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

use chrono::{DateTime, Utc};
use systemprompt::identifiers::ContextId;
use systemprompt_web_jobs::JobError;
use systemprompt_web_jobs::internals::{
    Category, Classification, ConversationClassifier, JudgeParams, JudgeVerdict, Outcome,
    run_with_classifier,
};

use crate::tempdb::TempDb;

struct StubClassifier {
    calls: AtomicUsize,
    transcript: Mutex<Option<String>>,
    result: Result<JudgeVerdict, String>,
}

#[async_trait::async_trait]
impl ConversationClassifier for StubClassifier {
    async fn classify(&self, transcript: &str) -> Result<JudgeVerdict, JobError> {
        self.calls.fetch_add(1, Ordering::Relaxed);
        *self.transcript.lock().expect("transcript lock") = Some(transcript.to_owned());
        self.result.clone().map_err(JobError::other)
    }
}

fn params(context_id: Option<ContextId>, cap: i64, batch_size: u32) -> JudgeParams {
    JudgeParams {
        provider: "fixture".into(),
        model: "fixture-model".into(),
        batch_size,
        daily_cost_cap_microdollars: cap,
        quiet_minutes: 30,
        lookback_days: 30,
        transcript_token_budget: 1_000,
        max_output_tokens: 256,
        context_id,
    }
}

async fn seed_user(pool: &sqlx::PgPool) {
    sqlx::query(
        "INSERT INTO users (id, name, email, roles) VALUES ('judge-user', 'judge-user', 'judge@example.test', ARRAY['user'])",
    )
    .execute(pool)
    .await
    .expect("seed owner");
}

struct SeededRequest<'a> {
    id: &'a str,
    context_id: &'a str,
    actor_kind: &'a str,
    synthetic: bool,
    request_kind: &'a str,
}

// A lease row: context, attempts, lease token, lease expiry and next attempt.
type LeaseRow = (
    String,
    i32,
    Option<String>,
    Option<DateTime<Utc>>,
    DateTime<Utc>,
);

// The judge listing row as the query returns it: context, turns, title,
// last activity, status and created-at.
type JudgeRow = (
    String,
    i32,
    Option<String>,
    Option<DateTime<Utc>>,
    Option<String>,
    DateTime<Utc>,
);

async fn seed_request(pool: &sqlx::PgPool, row: SeededRequest<'_>) {
    let SeededRequest {
        id,
        context_id,
        actor_kind,
        synthetic,
        request_kind,
    } = row;
    sqlx::query(
        "INSERT INTO ai_requests
             (id, request_id, user_id, context_id, provider, model, status, actor_kind, actor_id,
              synthetic, request_kind, created_at, updated_at)
         VALUES ($1, $1, 'judge-user', $2, 'fixture', 'fixture-model', 'completed', $3,
                 'judge-user', $4, $5, clock_timestamp() - interval '2 hours',
                 clock_timestamp() - interval '2 hours')",
    )
    .bind(id)
    .bind(context_id)
    .bind(actor_kind)
    .bind(synthetic)
    .bind(request_kind)
    .execute(pool)
    .await
    .expect("seed request");
}

async fn seed_turn(pool: &sqlx::PgPool, id: &str, with_message: bool) -> String {
    let context_id = uuid::Uuid::new_v4().to_string();
    seed_request(
        pool,
        SeededRequest {
            id,
            context_id: &context_id,
            actor_kind: "user",
            synthetic: false,
            request_kind: "turn",
        },
    )
    .await;
    if with_message {
        sqlx::query(
            "INSERT INTO ai_request_messages (id, request_id, role, content, sequence_number)
             VALUES ($1, $2, 'user', 'Please release the change.', 0)",
        )
        .bind(format!("message-{id}"))
        .bind(id)
        .execute(pool)
        .await
        .expect("seed transcript");
    }
    context_id
}

#[tokio::test]
async fn cap_releases_every_claimed_eligible_conversation_without_inference() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    seed_user(&db.pool).await;
    let eligible_one = seed_turn(&db.pool, "judge-eligible-one", false).await;
    let eligible_two = seed_turn(&db.pool, "judge-eligible-two", false).await;
    let synthetic = uuid::Uuid::new_v4().to_string();
    seed_request(
        &db.pool,
        SeededRequest {
            id: "judge-synthetic",
            context_id: &synthetic,
            actor_kind: "user",
            synthetic: true,
            request_kind: "turn",
        },
    )
    .await;
    let job = uuid::Uuid::new_v4().to_string();
    seed_request(
        &db.pool,
        SeededRequest {
            id: "judge-job",
            context_id: &job,
            actor_kind: "job",
            synthetic: false,
            request_kind: "turn",
        },
    )
    .await;
    let utility = uuid::Uuid::new_v4().to_string();
    seed_request(
        &db.pool,
        SeededRequest {
            id: "judge-utility",
            context_id: &utility,
            actor_kind: "user",
            synthetic: false,
            request_kind: "utility",
        },
    )
    .await;
    let classifier = StubClassifier {
        calls: AtomicUsize::new(0),
        transcript: Mutex::new(None),
        result: Err("must not run".into()),
    };

    let result = run_with_classifier(&db.pool, params(None, 0, 10), false, &classifier)
        .await
        .expect("cap is a successful stop");

    assert_eq!(
        (result.items_processed, result.items_failed),
        (Some(0), Some(0))
    );
    let admitted: Vec<String> = sqlx::query_scalar(
        "SELECT context_id FROM conversation_analyses
         WHERE context_id IN ($1, $2) ORDER BY context_id",
    )
    .bind(&eligible_one)
    .bind(&eligible_two)
    .fetch_all(&*db.pool)
    .await
    .expect("read queue");
    let mut expected = vec![eligible_one.clone(), eligible_two.clone()];
    expected.sort();
    assert_eq!(admitted, expected);
    let excluded: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM conversation_analyses WHERE context_id IN ($1, $2, $3)",
    )
    .bind(&synthetic)
    .bind(&job)
    .bind(&utility)
    .fetch_one(&*db.pool)
    .await
    .expect("read excluded contexts");
    assert_eq!(excluded, 0);
    let released: Vec<LeaseRow> = sqlx::query_as(
        "SELECT context_id, attempts, lease_token, lease_until, next_attempt
           FROM conversation_analyses
          WHERE context_id IN ($1, $2) ORDER BY context_id",
    )
    .bind(&eligible_one)
    .bind(&eligible_two)
    .fetch_all(&*db.pool)
    .await
    .expect("read released leases");
    assert!(
        released
            .iter()
            .all(|(_, attempts, token, until, next)| *attempts == 0
                && token.is_none()
                && until.is_none()
                && *next > Utc::now() + chrono::Duration::minutes(59))
    );
    assert_eq!(classifier.calls.load(Ordering::Relaxed), 0);
    db.cleanup().await;
}

#[tokio::test]
async fn classifier_result_persists_with_the_real_lease_and_receives_transcript() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    seed_user(&db.pool).await;
    let context = seed_turn(&db.pool, "judge-success", true).await;
    let classifier = StubClassifier {
        calls: AtomicUsize::new(0),
        transcript: Mutex::new(None),
        result: Ok(JudgeVerdict {
            classification: Classification {
                title: "Release change".into(),
                category: Category::Operations,
                summary: "A release was requested.".into(),
                tags: vec!["release".into()],
                outcome: Outcome::Partial,
                skills_observed: vec![],
                confidence: 0.8,
                completion: 60,
                completion_rationale: "The transcript contains no confirmation.".into(),
            },
            ai_request_id: "fixture-ai-call".into(),
            input_tokens: Some(10),
            output_tokens: Some(20),
        }),
    };

    let result = run_with_classifier(
        &db.pool,
        params(Some(ContextId::try_new(&context).expect("context")), 1, 1),
        false,
        &classifier,
    )
    .await
    .expect("judge tick");
    assert_eq!(
        (result.items_processed, result.items_failed),
        (Some(1), Some(0))
    );
    let row: (String, Option<String>, Option<String>, Option<i16>, Option<String>, i32) = sqlx::query_as(
        "SELECT status, title, outcome, completion, lease_token, attempts FROM conversation_analyses WHERE context_id = $1",
    ).bind(&context).fetch_one(&*db.pool).await.expect("classified row");
    assert_eq!(
        (row.0.as_str(), row.1.as_deref(), row.2.as_deref(), row.3),
        (
            "classified",
            Some("Release change"),
            Some("partial"),
            Some(60)
        )
    );
    assert!(row.4.is_none());
    assert_eq!(classifier.calls.load(Ordering::Relaxed), 1);
    assert!(
        classifier
            .transcript
            .lock()
            .expect("transcript lock")
            .as_deref()
            .is_some_and(|text| text.contains("Please release the change."))
    );
    db.cleanup().await;
}

#[tokio::test]
async fn forced_rejudge_does_not_claim_an_older_unrelated_pending_conversation() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    seed_user(&db.pool).await;
    let forced = seed_turn(&db.pool, "judge-forced", true).await;
    let older = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO conversation_analyses (context_id, user_id, status, next_attempt)
         VALUES ($1, 'judge-user', 'pending', clock_timestamp() - interval '1 hour')",
    )
    .bind(&older)
    .execute(&*db.pool)
    .await
    .expect("seed older queued conversation");
    let classifier = StubClassifier {
        calls: AtomicUsize::new(0),
        transcript: Mutex::new(None),
        result: Ok(JudgeVerdict {
            classification: Classification::unreadable(),
            ai_request_id: "forced-ai-call".into(),
            input_tokens: None,
            output_tokens: None,
        }),
    };

    let result = run_with_classifier(
        &db.pool,
        params(Some(ContextId::try_new(&forced).expect("context")), 1, 1),
        false,
        &classifier,
    )
    .await
    .expect("forced rejudge");

    assert_eq!(result.items_processed, Some(1));
    let states: Vec<(String, String, i32)> = sqlx::query_as(
        "SELECT context_id, status, attempts FROM conversation_analyses
         WHERE context_id IN ($1, $2) ORDER BY context_id",
    )
    .bind(&forced)
    .bind(&older)
    .fetch_all(&*db.pool)
    .await
    .expect("read queue states");
    assert!(
        states
            .iter()
            .any(|(id, status, _)| id == &forced && status == "classified")
    );
    assert!(
        states
            .iter()
            .any(|(id, status, attempts)| id == &older && status == "pending" && *attempts == 0)
    );
    db.cleanup().await;
}

#[tokio::test]
async fn manual_only_run_leaves_automatic_queue_items_unclaimed() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    seed_user(&db.pool).await;
    let manual = seed_turn(&db.pool, "judge-manual", true).await;
    let automatic = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO conversation_analyses (context_id, user_id, status, trigger, next_attempt)
         VALUES ($1, 'judge-user', 'pending', 'manual', clock_timestamp()),
                ($2, 'judge-user', 'pending', 'automatic', clock_timestamp() - interval '1 hour')",
    )
    .bind(&manual)
    .bind(&automatic)
    .execute(&*db.pool)
    .await
    .expect("seed manual and automatic queue items");
    let classifier = StubClassifier {
        calls: AtomicUsize::new(0),
        transcript: Mutex::new(None),
        result: Ok(JudgeVerdict {
            classification: Classification::unreadable(),
            ai_request_id: "manual-ai-call".into(),
            input_tokens: None,
            output_tokens: None,
        }),
    };

    let result = run_with_classifier(&db.pool, params(None, 1, 10), true, &classifier)
        .await
        .expect("manual-only tick");

    assert_eq!(result.items_processed, Some(1));
    let states: Vec<(String, String, i32)> = sqlx::query_as(
        "SELECT context_id, status, attempts FROM conversation_analyses
         WHERE context_id IN ($1, $2) ORDER BY context_id",
    )
    .bind(&manual)
    .bind(&automatic)
    .fetch_all(&*db.pool)
    .await
    .expect("read manual-only queue states");
    assert!(
        states
            .iter()
            .any(|(id, status, _)| id == &manual && status == "classified")
    );
    assert!(
        states.iter().any(|(id, status, attempts)| id == &automatic
            && status == "pending"
            && *attempts == 0)
    );
    db.cleanup().await;
}

#[tokio::test]
async fn unreadable_transcript_is_classified_without_inference() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    seed_user(&db.pool).await;
    let context = seed_turn(&db.pool, "judge-unreadable", false).await;
    let classifier = StubClassifier {
        calls: AtomicUsize::new(0),
        transcript: Mutex::new(None),
        result: Err("must not run for empty transcript".into()),
    };

    let result = run_with_classifier(
        &db.pool,
        params(Some(ContextId::try_new(&context).expect("context")), 1, 1),
        false,
        &classifier,
    )
    .await
    .expect("unreadable conversation is settled");

    assert_eq!(result.items_processed, Some(1));
    let row: (String, Option<String>, Option<String>, Option<String>) = sqlx::query_as(
        "SELECT status, title, outcome, ai_request_id FROM conversation_analyses WHERE context_id = $1",
    )
    .bind(&context)
    .fetch_one(&*db.pool)
    .await
    .expect("read unreadable judgement");
    assert_eq!(row.0, "classified");
    assert_eq!(row.1.as_deref(), Some("Unreadable conversation"));
    assert_eq!(row.2.as_deref(), Some("unclear"));
    assert!(row.3.is_none());
    assert_eq!(classifier.calls.load(Ordering::Relaxed), 0);
    db.cleanup().await;
}

#[tokio::test]
async fn inference_failure_releases_ownership_and_records_retry_audit() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    seed_user(&db.pool).await;
    let context = seed_turn(&db.pool, "judge-retry", true).await;
    let classifier = StubClassifier {
        calls: AtomicUsize::new(0),
        transcript: Mutex::new(None),
        result: Err("provider unavailable".into()),
    };

    let result = run_with_classifier(
        &db.pool,
        params(Some(ContextId::try_new(&context).expect("context")), 1, 1),
        false,
        &classifier,
    )
    .await
    .expect("per-item failure is recorded");
    assert_eq!(
        (result.items_processed, result.items_failed),
        (Some(0), Some(1))
    );
    let row: JudgeRow = sqlx::query_as(
        "SELECT status, attempts, lease_token, lease_until, last_error, next_attempt FROM conversation_analyses WHERE context_id = $1",
    ).bind(&context).fetch_one(&*db.pool).await.expect("retry row");
    assert_eq!((row.0.as_str(), row.1), ("pending", 1));
    assert!(row.2.is_none() && row.3.is_none());
    assert!(
        row.4
            .as_deref()
            .is_some_and(|error| error.contains("provider unavailable"))
    );
    assert!(row.5 > Utc::now());
    db.cleanup().await;
}

#[tokio::test]
async fn fifth_inference_failure_parks_the_conversation_without_another_paid_retry() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    seed_user(&db.pool).await;
    let context = seed_turn(&db.pool, "judge-final-retry", true).await;
    sqlx::query(
        "INSERT INTO conversation_analyses
             (context_id, user_id, status, trigger, attempts, next_attempt)
         VALUES ($1, 'judge-user', 'pending', 'manual', 4, clock_timestamp())",
    )
    .bind(&context)
    .execute(&*db.pool)
    .await
    .expect("seed fourth failed attempt");
    let classifier = StubClassifier {
        calls: AtomicUsize::new(0),
        transcript: Mutex::new(None),
        result: Err("provider remains unavailable".into()),
    };

    let result = run_with_classifier(&db.pool, params(None, 1, 1), true, &classifier)
        .await
        .expect("the terminal failure is recorded");
    assert_eq!(
        (result.items_processed, result.items_failed),
        (Some(0), Some(1))
    );
    let row: (
        String,
        i32,
        Option<String>,
        Option<DateTime<Utc>>,
        Option<String>,
    ) = sqlx::query_as(
        "SELECT status, attempts, lease_token, lease_until, last_error
             FROM conversation_analyses WHERE context_id = $1",
    )
    .bind(&context)
    .fetch_one(&*db.pool)
    .await
    .expect("read terminal retry row");
    assert_eq!((row.0.as_str(), row.1), ("failed", 5));
    assert!(row.2.is_none() && row.3.is_none());
    assert!(
        row.4
            .as_deref()
            .is_some_and(|error| error.contains("provider remains unavailable"))
    );

    let next = run_with_classifier(&db.pool, params(None, 1, 1), true, &classifier)
        .await
        .expect("failed rows are not claimed again");
    assert_eq!(
        (next.items_processed, next.items_failed),
        (Some(0), Some(0))
    );
    assert_eq!(classifier.calls.load(Ordering::Relaxed), 1);
    db.cleanup().await;
}

#[tokio::test]
async fn idle_tick_backfills_late_settled_judge_spend() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    seed_user(&db.pool).await;
    let request_context = uuid::Uuid::new_v4().to_string();
    seed_request(
        &db.pool,
        SeededRequest {
            id: "judge-late-cost-request",
            context_id: &request_context,
            actor_kind: "job",
            synthetic: false,
            request_kind: "utility",
        },
    )
    .await;
    sqlx::query(
        "UPDATE ai_requests
         SET actor_id = 'conversation_judge', cost_microdollars = 4321
         WHERE id = 'judge-late-cost-request'",
    )
    .execute(&*db.pool)
    .await
    .expect("settle the judge request cost");
    let analysis_context = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO conversation_analyses
             (context_id, user_id, status, ai_request_id, classified_at, cost_microdollars)
         VALUES ($1, 'judge-user', 'classified', 'judge-late-cost-request',
                 clock_timestamp(), NULL)",
    )
    .bind(&analysis_context)
    .execute(&*db.pool)
    .await
    .expect("seed judgement awaiting settlement");
    let classifier = StubClassifier {
        calls: AtomicUsize::new(0),
        transcript: Mutex::new(None),
        result: Err("idle tick must not infer".into()),
    };

    let result = run_with_classifier(&db.pool, params(None, 1, 1), true, &classifier)
        .await
        .expect("idle tick reconciles settled spend");
    assert_eq!(
        (result.items_processed, result.items_failed),
        (Some(0), Some(0))
    );
    let cost: Option<i64> = sqlx::query_scalar(
        "SELECT cost_microdollars FROM conversation_analyses WHERE context_id = $1",
    )
    .bind(&analysis_context)
    .fetch_one(&*db.pool)
    .await
    .expect("read reconciled cost");
    assert_eq!(cost, Some(4321));
    assert_eq!(classifier.calls.load(Ordering::Relaxed), 0);
    db.cleanup().await;
}
