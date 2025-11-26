use crate::common::*;
use anyhow::Result;
use systemprompt_core_database::DatabaseQueryEnum;

#[tokio::test]
async fn test_task_messages_persisted() -> Result<()> {
    let ctx = TestContext::new().await?;
    let fingerprint = ctx.fingerprint().to_string();
    let mut cleanup = TestCleanup::new(ctx.db.clone());
    cleanup.track_fingerprint(fingerprint.clone());

    let token = ctx.get_anonymous_token().await?;

    // Create context first
    let context_id = ctx.create_context(&token, "What is AI?").await?;

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
        create_a2a_message("What is AI?", &context_id);

    let agent_url = format!("/api/v1/agents/{}/", agent_id);
    let response = ctx
        .make_authenticated_json_request(reqwest::Method::POST, &agent_url, &token, message)
        .await?;

    assert!(response.status().is_success(), "Request failed");

    wait_for_async_processing().await;

    let query = DatabaseQueryEnum::GetTaskMessages.get(ctx.db.as_ref());
    let rows = ctx.db.fetch_all(&query, &[&task_id]).await?;

    assert!(!rows.is_empty(), "No messages found in database");

    let message_row = &rows[0];
    assert!(
        message_row.get("message_id").is_some(),
        "Missing message_id"
    );
    assert!(message_row.get("task_id").is_some(), "Missing task_id");
    assert!(message_row.get("role").is_some(), "Missing role");
    assert!(
        message_row.get("sequence_number").is_some(),
        "Missing sequence_number"
    );

    cleanup.track_task_id(task_id);
    cleanup.cleanup_all().await?;

    println!("✓ Task messages persisted");
    Ok(())
}

#[tokio::test]
async fn test_message_structure_complete() -> Result<()> {
    let ctx = TestContext::new().await?;
    let fingerprint = ctx.fingerprint().to_string();
    let mut cleanup = TestCleanup::new(ctx.db.clone());
    cleanup.track_fingerprint(fingerprint.clone());

    let token = ctx.get_anonymous_token().await?;

    // Create context first
    let context_id = ctx.create_context(&token, "Test message structure").await?;

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
        create_a2a_message("Test message structure", &context_id);

    let agent_url = format!("/api/v1/agents/{}/", agent_id);
    let response = ctx
        .make_authenticated_json_request(reqwest::Method::POST, &agent_url, &token, message)
        .await?;

    assert!(response.status().is_success(), "Request failed");

    wait_for_async_processing().await;

    let query = DatabaseQueryEnum::GetTaskMessages.get(ctx.db.as_ref());
    let rows = ctx.db.fetch_all(&query, &[&task_id]).await?;

    assert!(!rows.is_empty(), "No messages found");

    for row in &rows {
        if let Some(role) = row.get("role").and_then(|v| v.as_str()) {
            assert!(
                role == "user" || role == "assistant" || role == "agent" || role == "system",
                "Invalid role: {}",
                role
            );
        }

        assert!(
            row.get("sequence_number").is_some(),
            "Missing sequence_number"
        );
        assert!(row.get("created_at").is_some(), "Missing created_at");
    }

    cleanup.track_task_id(task_id);
    cleanup.cleanup_all().await?;

    println!("✓ Message structure is complete");
    Ok(())
}

#[tokio::test]
async fn test_receive_agent_response() -> Result<()> {
    let ctx = TestContext::new().await?;
    let fingerprint = ctx.fingerprint().to_string();
    let mut cleanup = TestCleanup::new(ctx.db.clone());
    cleanup.track_fingerprint(fingerprint.clone());

    let token = ctx.get_anonymous_token().await?;

    // Create context first
    let context_id = ctx.create_context(&token, "Hello agent").await?;

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
        create_a2a_message("Hello agent", &context_id);

    let agent_url = format!("/api/v1/agents/{}/", agent_id);
    let response = ctx
        .make_authenticated_json_request(reqwest::Method::POST, &agent_url, &token, message)
        .await?;

    assert!(response.status().is_success(), "Agent request failed");

    let body: serde_json::Value = response.json().await?;
    assert!(
        body.get("id").is_some() || body.get("result").is_some(),
        "Response missing id field"
    );

    cleanup.track_task_id(task_id);
    cleanup.cleanup_all().await?;

    println!("✓ Agent response received in expected format");
    Ok(())
}
