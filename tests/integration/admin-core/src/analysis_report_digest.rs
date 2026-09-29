//! Evidence digests must describe only the requested people and time window:
//! the report job receives this text instead of raw conversation transcripts.

use chrono::{Duration, Utc};
use systemprompt_web_admin::repositories::analysis::conversations::{
    BreakdownBy, ConversationAnalysisFilter, ConversationAnalysisPage, FactSort,
    load_conversation_analysis_page,
};
use systemprompt_web_admin::repositories::analysis::reports::{
    Assessment, DigestScope, NewReport, ReportCompletion, ReportDigest, ReportDigestInputs,
    ReportFindings, fail_report, find_report, get_report_digest, insert_report_request,
    render_digest_text, update_report_completion,
};

use crate::fixtures::{insert_user, unclaimed_email, unique};
use crate::tempdb::TempDb;

struct Fact<'a> {
    context_id: &'a str,
    user_id: &'a str,
    model: &'a str,
    client_kind: &'a str,
    at: chrono::DateTime<Utc>,
    cost: i64,
}

async fn insert_fact(db: &TempDb, fact: Fact<'_>) {
    let Fact {
        context_id,
        user_id,
        model,
        client_kind,
        at,
        cost,
    } = fact;
    sqlx::query(
        "INSERT INTO conversation_facts
             (context_id, user_id, client_kind, model, models, request_count, turn_count,
              input_tokens, output_tokens, cost_microdollars, error_count, gov_deny,
              tool_calls_intended, safety_findings, artifact_count, skills, first_at, last_at)
         VALUES ($1, $2, $3, $4, ARRAY[$4], 2, 2, 120, 80, $5, 1, 1, 3, 1, 1,
                 ARRAY['deploy:release'], $6, $6)",
    )
    .bind(context_id)
    .bind(user_id)
    .bind(client_kind)
    .bind(model)
    .bind(cost)
    .bind(at)
    .execute(&*db.pool)
    .await
    .expect("insert conversation fact");
}

#[tokio::test]
async fn report_completion_and_failure_are_fenced_by_the_active_lease() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let requester = insert_user(&db.pool, &unique("report-owner"), &unclaimed_email("owner")).await;
    let now = Utc::now();
    let lease = unique("lease");
    let report_id = insert_report_request(
        &db.pool,
        NewReport {
            scope_kind: "global".into(),
            scope_id: None,
            scope_label: None,
            window_start: now - Duration::days(1),
            window_end: now,
            requested_by: requester.to_string(),
            inputs: ReportDigestInputs {
                digest: ReportDigest::default(),
                filter_query: None,
                filter_label: None,
            },
            lease_token: lease.clone(),
        },
    )
    .await
    .expect("queue report");
    let findings = ReportFindings {
        headline: "No material issues".into(),
        assessment: Assessment::Ok,
        themes: vec![],
        recommendations: vec![],
    };
    let completion = ReportCompletion {
        id: &report_id,
        lease_token: &lease,
        findings: &findings,
        provider: "fixture-provider",
        model: "fixture-model",
        ai_request_id: None,
        input_tokens: Some(11),
        output_tokens: Some(7),
    };

    assert!(
        update_report_completion(&db.pool, completion)
            .await
            .expect("complete leased report")
    );
    assert!(
        !update_report_completion(
            &db.pool,
            ReportCompletion {
                id: &report_id,
                lease_token: &lease,
                findings: &findings,
                provider: "stale-provider",
                model: "stale-model",
                ai_request_id: None,
                input_tokens: None,
                output_tokens: None,
            },
        )
        .await
        .expect("stale completion is harmless")
    );
    let completed = find_report(&db.pool, &report_id)
        .await
        .expect("read report")
        .expect("stored report");
    assert_eq!(completed.status, "generated");
    assert_eq!(completed.provider.as_deref(), Some("fixture-provider"));
    assert_eq!(
        completed.findings.expect("findings").headline,
        findings.headline
    );
    assert!(completed.lease_token.is_none());

    let failed_id = insert_report_request(
        &db.pool,
        NewReport {
            scope_kind: "global".into(),
            scope_id: None,
            scope_label: None,
            window_start: now - Duration::days(1),
            window_end: now,
            requested_by: requester.to_string(),
            inputs: ReportDigestInputs {
                digest: ReportDigest::default(),
                filter_query: None,
                filter_label: None,
            },
            lease_token: unique("active-lease"),
        },
    )
    .await
    .expect("queue second report");
    fail_report(&db.pool, &failed_id, "wrong-lease", "must not win")
        .await
        .expect("stale failure is harmless");
    assert_eq!(
        find_report(&db.pool, &failed_id)
            .await
            .expect("read pending")
            .expect("queued")
            .status,
        "pending"
    );
    db.cleanup().await;
}

#[tokio::test]
async fn report_digest_scopes_aggregates_and_model_evidence_to_one_person() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let selected = insert_user(&db.pool, &unique("report-user"), &unclaimed_email("report")).await;
    let other = insert_user(&db.pool, &unique("report-other"), &unclaimed_email("other")).await;
    let now = Utc::now();
    let selected_context = uuid::Uuid::new_v4().to_string();

    insert_fact(
        &db,
        Fact {
            context_id: &selected_context,
            user_id: selected.as_str(),
            model: "selected-model",
            client_kind: "claude-code",
            at: now,
            cost: 5_000_000,
        },
    )
    .await;
    insert_fact(
        &db,
        Fact {
            context_id: &uuid::Uuid::new_v4().to_string(),
            user_id: other.as_str(),
            model: "noise-model",
            client_kind: "other-client",
            at: now,
            cost: 9_000_000,
        },
    )
    .await;
    insert_fact(
        &db,
        Fact {
            context_id: &uuid::Uuid::new_v4().to_string(),
            user_id: selected.as_str(),
            model: "old-model",
            client_kind: "claude-code",
            at: now - Duration::days(2),
            cost: 7_000_000,
        },
    )
    .await;
    sqlx::query(
        "INSERT INTO conversation_analyses
             (context_id, user_id, status, title, summary, category, outcome, completion,
              classified_at)
         VALUES ($1, $2, 'classified', 'Successful release', 'Release completed',
                 'operations', 'achieved', 92, clock_timestamp())",
    )
    .bind(&selected_context)
    .bind(selected.as_str())
    .execute(&*db.pool)
    .await
    .expect("insert judgement");

    let digest = get_report_digest(
        &db.pool,
        &DigestScope {
            window_start: now - Duration::hours(1),
            window_end: now + Duration::hours(1),
            user_key: Some(selected.to_string()),
            ..DigestScope::default()
        },
    )
    .await
    .expect("load scoped digest");

    assert_eq!(digest.totals.conversations, 1);
    assert_eq!(digest.totals.people, 1);
    assert_eq!(digest.totals.turns, 2);
    assert_eq!(digest.totals.tokens, 200);
    assert_eq!(digest.totals.cost_microdollars, 5_000_000);
    assert_eq!(digest.totals.judged, 1);
    assert_eq!(digest.totals.achieved, 1);
    assert_eq!(digest.models[0].model, "selected-model");
    assert_eq!(digest.people[0].user_key, selected.as_str());
    assert_eq!(digest.clients[0].client_kind, "claude-code");
    assert_eq!(digest.best[0].context_key, selected_context);
    assert_eq!(
        digest.worst[0].judge_title.as_deref(),
        Some("Successful release")
    );

    let text = render_digest_text(&digest);
    assert!(text.contains("cost=$5.00"));
    assert!(text.contains("selected-model"));
    assert!(text.contains(&selected_context));
    assert!(!text.contains("noise-model"));
    assert!(!text.contains("old-model"));
    db.cleanup().await;
}

#[tokio::test]
async fn conversation_analysis_page_keeps_rows_totals_facets_and_breakdown_in_one_scope() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let selected = insert_user(
        &db.pool,
        &unique("analysis-user"),
        &unclaimed_email("analysis"),
    )
    .await;
    let other = insert_user(
        &db.pool,
        &unique("analysis-noise"),
        &unclaimed_email("noise"),
    )
    .await;
    let now = Utc::now();
    let selected_context = uuid::Uuid::new_v4().to_string();
    insert_fact(
        &db,
        Fact {
            context_id: &selected_context,
            user_id: selected.as_str(),
            model: "scoped-model",
            client_kind: "claude-code",
            at: now,
            cost: 2_000_000,
        },
    )
    .await;
    insert_fact(
        &db,
        Fact {
            context_id: &uuid::Uuid::new_v4().to_string(),
            user_id: other.as_str(),
            model: "unscoped-model",
            client_kind: "other-client",
            at: now,
            cost: 9_000_000,
        },
    )
    .await;

    let result = load_conversation_analysis_page(
        &db.pool,
        &ConversationAnalysisFilter {
            user_id: Some(selected.clone()),
            since: Some(now - Duration::hours(1)),
            until: Some(now + Duration::hours(1)),
            ..ConversationAnalysisFilter::default()
        },
        ConversationAnalysisPage {
            sort: FactSort::Cost,
            descending: true,
            limit: 20,
            offset: 0,
            breakdown: BreakdownBy::Model,
        },
    )
    .await
    .expect("load scoped analysis page");

    assert_eq!(result.rows.len(), 1);
    assert_eq!(result.rows[0].context_id.to_string(), selected_context);
    assert_eq!(result.totals.conversations, 1);
    assert_eq!(result.totals.users, 1);
    assert_eq!(result.totals.turns, 2);
    assert_eq!(result.totals.total_cost_microdollars, 2_000_000);
    assert_eq!(result.totals.with_skills, 1);
    assert_eq!(result.models[0].model, "scoped-model");
    assert_eq!(result.clients[0].client_kind, "claude-code");
    assert_eq!(result.skills[0].skill, "deploy:release");
    assert_eq!(result.breakdown[0].label, "scoped-model");
    db.cleanup().await;
}
