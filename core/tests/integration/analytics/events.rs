/// Tests for analytics events tracking
use crate::common::*;
use anyhow::Result;
use serde_json::Value;
use systemprompt_core_database::DatabaseQueryEnum;

#[tokio::test]
async fn test_page_view_event_recorded() -> Result<()> {
    let ctx = TestContext::new().await?;
    let fingerprint = ctx.fingerprint().to_string();

    let response = ctx.make_request("/").await?;
    assert!(response.status().is_success());

    wait_for_async_processing().await;

    let session_query = DatabaseQueryEnum::FindSessionByFingerprintAny.get(ctx.db.as_ref());
    let session_rows = ctx.db.fetch_all(&session_query, &[&fingerprint]).await?;
    assert!(!session_rows.is_empty(), "Session not created");

    let session = get_session_from_row(&session_rows[0])?;

    let event_query = DatabaseQueryEnum::GetEventsBySession.get(ctx.db.as_ref());
    let event_rows = ctx
        .db
        .fetch_all(&event_query, &[&session.session_id])
        .await?;

    assert!(!event_rows.is_empty(), "No events recorded for session");

    let event_row = &event_rows[0];
    let event_type = event_row
        .get("event_type")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    assert!(
        event_type == "page_view" || event_type == "navigation",
        "Expected page_view or navigation event, got: {}",
        event_type
    );

    let mut cleanup = TestCleanup::new(ctx.db.clone());
    cleanup.track_fingerprint(fingerprint);
    cleanup.cleanup_all().await?;

    println!("✓ Page view event recorded");
    Ok(())
}

#[tokio::test]
async fn test_event_metadata_persisted() -> Result<()> {
    let ctx = TestContext::new().await?;
    let fingerprint = ctx.fingerprint().to_string();

    let response = ctx.make_request("/").await?;
    assert!(response.status().is_success());

    wait_for_async_processing().await;

    let session_query = DatabaseQueryEnum::FindSessionByFingerprintAny.get(ctx.db.as_ref());
    let session_rows = ctx.db.fetch_all(&session_query, &[&fingerprint]).await?;
    assert!(!session_rows.is_empty());

    let session = get_session_from_row(&session_rows[0])?;

    let event_query = DatabaseQueryEnum::GetEventsBySession.get(ctx.db.as_ref());
    let event_rows = ctx
        .db
        .fetch_all(&event_query, &[&session.session_id])
        .await?;

    assert!(!event_rows.is_empty(), "No events recorded");

    for event_row in event_rows {
        let metadata_value = event_row.get("metadata");
        assert!(metadata_value.is_some(), "Event metadata is missing");

        if let Some(Value::String(metadata_str)) = metadata_value {
            let parsed: Result<serde_json::Value, _> = serde_json::from_str(metadata_str);
            assert!(parsed.is_ok(), "Metadata is not valid JSON");
        }
    }

    let mut cleanup = TestCleanup::new(ctx.db.clone());
    cleanup.track_fingerprint(fingerprint);
    cleanup.cleanup_all().await?;

    println!("✓ Event metadata persisted correctly");
    Ok(())
}
