//! A retained fact may outlive directory rows or never have a judge result.
//! The detail query must still return its non-null rollup metrics while each
//! outer-joined field stays absent.

use chrono::{Duration, Utc};
use systemprompt::identifiers::{ContextId, UserId};
use systemprompt_web_admin::repositories::analysis::conversations::detail::find_conversation_facts;

use crate::fixtures::{new_context_id, unique};
use crate::tempdb::TempDb;

#[tokio::test]
async fn detail_keeps_required_fact_metrics_when_directory_and_judge_rows_are_absent() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let context = ContextId::try_new(new_context_id()).expect("valid fixture identifier");
    let historical_user = UserId::new(unique("deleted-directory-user"));
    let now = chrono::DateTime::from_timestamp_micros(Utc::now().timestamp_micros())
        .expect("current time has microsecond precision");
    let first_at = now - Duration::minutes(4);
    let last_at = now - Duration::minutes(1);
    sqlx::query(
        "INSERT INTO conversation_facts
             (context_id, user_id, client_kind, client_attestation, wire_protocol,
              models, providers, request_count, turn_count, input_tokens, output_tokens,
              cost_microdollars, tool_calls_intended, skills, first_at, last_at, duration_seconds)
         VALUES ($1, $2, 'historical-client', 'unattested', 'legacy-wire',
                 ARRAY['historical-model'], ARRAY['historical-provider'], 3, 2, 120, 80,
                 9000, 4, ARRAY['historical:skill'], $3, $4, 180)",
    )
    .bind(context.as_str())
    .bind(historical_user.as_str())
    .bind(first_at)
    .bind(last_at)
    .execute(&*db.pool)
    .await
    .expect("insert retained historical fact");

    let row = find_conversation_facts(&db.pool, &context)
        .await
        .expect("detail fact query")
        .expect("retained fact row");
    assert_eq!(row.user_id, historical_user);
    assert_eq!(row.client_kind, "historical-client");
    assert_eq!(row.models, vec!["historical-model".to_owned()]);
    assert_eq!(row.providers, vec!["historical-provider".to_owned()]);
    assert_eq!(row.request_count, 3);
    assert_eq!(row.turn_count, 2);
    assert_eq!(row.total_tokens, 200);
    assert_eq!(row.cost_microdollars, 9000);
    assert_eq!(row.tool_calls_intended, 4);
    assert_eq!(row.skills, vec!["historical:skill".to_owned()]);
    assert_eq!(row.first_at, first_at);
    assert_eq!(row.last_at, last_at);
    assert_eq!(row.duration_seconds, 180);
    assert!(row.display_name.is_none());
    assert!(row.group_id.is_none());
    assert!(row.project_id.is_none());
    assert!(row.group_name.is_none());
    assert!(row.project_name.is_none());
    assert!(row.judge_status.is_none());
    assert!(row.summary.is_none());
    assert!(row.tags.is_empty());
    assert!(row.skills_used.is_empty());
    assert!(row.outcome.is_none());
    assert!(row.completion.is_none());
    db.cleanup().await;
}
