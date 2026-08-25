//! REQ-026 Immutable AI Audit Trail — "Every model call produces an immutable
//! audit record containing actor, time, model/provider, request/response
//! lineage, policy decision, and relevant consumption metadata."
//!
//! Seeds one `ai_requests` row through the same insert path the rest of the
//! suite uses, then reads it back two ways: through the requests entity view
//! the admin console renders (`list_requests_paged`), asserting the actor,
//! time, model, provider, cost, and trace linkage all survive the read model;
//! and directly from the audit table, asserting the actor and lineage columns
//! the view does not project are populated rather than null.

use systemprompt_web_admin::repositories::analytics::requests::{
    RequestFilter, RequestPage, RequestSortSpec, list_requests_paged,
};

use crate::fixtures::{
    RequestSpec, insert_request, insert_session, insert_user, narrow_window, unclaimed_email,
    unique,
};
use crate::tempdb::TempDb;

#[tokio::test]
async fn the_requests_view_carries_the_full_audit_chain() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("audit")).await;
    let session = unique("session");
    insert_session(&db.pool, &session, &user).await;
    let trace = unique("trace");
    let id = unique("req");
    let mut spec = RequestSpec::completed(&id, &user);
    spec.session_id = Some(&session);
    spec.trace_id = Some(&trace);
    spec.provider = "audited-provider";
    spec.model = "audited-model";
    insert_request(&db.pool, &spec).await;

    let page = RequestPage {
        sort: RequestSortSpec::default(),
        limit: 10,
        offset: 0,
    };
    let (rows, total) =
        list_requests_paged(&db.pool, &RequestFilter::default(), narrow_window(), page)
            .await
            .expect("query succeeds");

    assert_eq!(total, 1);
    let row = &rows[0];
    assert_eq!(row.id, id);
    assert_eq!(row.user_id, user, "the actor is on the record");
    assert_eq!(row.model, "audited-model");
    assert_eq!(row.provider, "audited-provider");
    assert_eq!(row.cost_microdollars, 5_000, "consumption metadata is costed");
    assert_eq!(
        row.session_id.as_ref().map(|s| s.as_str().to_owned()),
        Some(session.clone()),
        "the record links back to its session"
    );
    assert_eq!(
        row.trace_id.as_ref().map(|t| t.as_str().to_owned()),
        Some(trace.clone()),
        "the record links back to its trace"
    );
    assert!(
        row.input_tokens.is_some() && row.output_tokens.is_some(),
        "token consumption is recorded"
    );
    db.cleanup().await;
}

#[tokio::test]
async fn the_audit_row_itself_names_its_actor_and_lineage() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("lineage")).await;
    let id = unique("req");
    insert_request(&db.pool, &RequestSpec::completed(&id, &user)).await;

    let (actor_kind, actor_id, request_id, created_at): (
        Option<String>,
        Option<String>,
        Option<String>,
        Option<chrono::DateTime<chrono::Utc>>,
    ) = sqlx::query_as(
        "SELECT actor_kind, actor_id, request_id, created_at FROM ai_requests WHERE id = $1",
    )
    .bind(&id)
    .fetch_one(&*db.pool)
    .await
    .expect("the audit row exists");

    assert_eq!(actor_kind.as_deref(), Some("user"), "the actor kind is recorded");
    assert_eq!(
        actor_id.as_deref(),
        Some(user.as_str()),
        "the actor id matches the caller"
    );
    assert_eq!(
        request_id.as_deref(),
        Some(id.as_str()),
        "request lineage is recorded"
    );
    assert!(created_at.is_some(), "the record is timestamped");
    db.cleanup().await;
}
