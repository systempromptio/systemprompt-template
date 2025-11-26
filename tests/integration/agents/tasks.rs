use crate::common::*;
use anyhow::Result;
use systemprompt_core_database::DatabaseQueryEnum;

#[tokio::test]
async fn test_agent_task_created_in_database() -> Result<()> {
    let ctx = TestContext::new().await?;
    let fingerprint = ctx.fingerprint().to_string();
    let mut cleanup = TestCleanup::new(ctx.db.clone());
    cleanup.track_fingerprint(fingerprint.clone());

    let token = ctx.get_anonymous_token().await?;

    // Create context first
    let context_id = ctx.create_context(&token, "Test task creation").await?;

    let registry = ctx
        .make_authenticated_request(reqwest::Method::GET, "/api/v1/agents/registry", &token)
        .await?;
    let registry_body: serde_json::Value = registry.json().await?;
    let agents = registry_body["data"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("No agents in registry"))?;
    let agent_id = agents[0]["name"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Agent missing ID"))?;

    let (task_id, _context_id, _message_id, message) =
        create_a2a_message("Test task creation", &context_id);

    let agent_url = format!("/api/v1/agents/{}/", agent_id);
    let response = ctx
        .make_authenticated_json_request(reqwest::Method::POST, &agent_url, &token, message)
        .await?;

    assert!(response.status().is_success(), "Conversation failed");

    wait_for_async_processing().await;

    let query = DatabaseQueryEnum::GetTask.get(ctx.db.as_ref());
    let rows = ctx.db.fetch_all(&query, &[&task_id]).await?;

    assert!(!rows.is_empty(), "Task not found in database");

    let task_row = &rows[0];
    assert!(task_row.get("task_id").is_some(), "Missing task_id");
    assert!(task_row.get("agent_name").is_some(), "Missing agent_name");
    assert!(task_row.get("created_at").is_some(), "Missing created_at");

    cleanup.track_task_id(task_id);
    cleanup.cleanup_all().await?;

    println!("✓ Agent task created in database");
    Ok(())
}

#[tokio::test]
async fn test_task_context_preserved() -> Result<()> {
    let ctx = TestContext::new().await?;
    let fingerprint = ctx.fingerprint().to_string();
    let mut cleanup = TestCleanup::new(ctx.db.clone());
    cleanup.track_fingerprint(fingerprint.clone());

    let token = ctx.get_anonymous_token().await?;

    // Create context first
    let context_id = ctx
        .create_context(&token, "Test context preservation")
        .await?;

    let registry = ctx
        .make_authenticated_request(reqwest::Method::GET, "/api/v1/agents/registry", &token)
        .await?;
    let registry_body: serde_json::Value = registry.json().await?;
    let agents = registry_body["data"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("No agents in registry"))?;
    let agent_id = agents[0]["name"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Agent missing ID"))?;

    let (task_id, _context_id, _message_id, message) =
        create_a2a_message("Test context preservation", &context_id);

    let agent_url = format!("/api/v1/agents/{}/", agent_id);
    let response = ctx
        .make_authenticated_json_request(reqwest::Method::POST, &agent_url, &token, message)
        .await?;

    assert!(response.status().is_success(), "Request failed");

    wait_for_async_processing().await;

    let query = DatabaseQueryEnum::GetTask.get(ctx.db.as_ref());
    let rows = ctx.db.fetch_all(&query, &[&task_id]).await?;

    assert!(!rows.is_empty(), "Task not found");

    let task_row = &rows[0];
    if let Some(context) = task_row.get("task_context") {
        assert!(!context.is_null(), "Task context should be preserved");
    }

    cleanup.track_task_id(task_id);
    cleanup.cleanup_all().await?;

    println!("✓ Task context preserved");
    Ok(())
}
