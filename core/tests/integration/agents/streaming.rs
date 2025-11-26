use crate::common::*;
use anyhow::Result;

#[tokio::test]
async fn test_streaming_response_format() -> Result<()> {
    let ctx = TestContext::new().await?;
    let fingerprint = ctx.fingerprint().to_string();
    let mut cleanup = TestCleanup::new(ctx.db.clone());
    cleanup.track_fingerprint(fingerprint.clone());

    let token = ctx.get_anonymous_token().await?;

    // Create context first
    let context_id = ctx.create_context(&token, "What is streaming?").await?;

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
        create_a2a_message("What is streaming?", &context_id);

    let agent_url = format!("/api/v1/agents/{}/", agent_id);
    let response = ctx
        .make_authenticated_json_request(reqwest::Method::POST, &agent_url, &token, message)
        .await?;

    assert!(response.status().is_success(), "Request failed");

    let body: serde_json::Value = response.json().await?;
    assert!(
        body.get("id").is_some() || body.get("result").is_some(),
        "Missing id in response"
    );

    cleanup.track_task_id(task_id);
    cleanup.cleanup_all().await?;

    println!("✓ Streaming response format is valid");
    Ok(())
}

#[tokio::test]
async fn test_response_contains_valid_json() -> Result<()> {
    let ctx = TestContext::new().await?;
    let fingerprint = ctx.fingerprint().to_string();
    let mut cleanup = TestCleanup::new(ctx.db.clone());
    cleanup.track_fingerprint(fingerprint.clone());

    let token = ctx.get_anonymous_token().await?;

    // Create context first
    let context_id = ctx.create_context(&token, "Test JSON response").await?;

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
        create_a2a_message("Test JSON response", &context_id);

    let agent_url = format!("/api/v1/agents/{}/", agent_id);
    let response = ctx
        .make_authenticated_json_request(reqwest::Method::POST, &agent_url, &token, message)
        .await?;

    assert!(response.status().is_success(), "Request failed");

    let text = response.text().await?;
    assert!(!text.is_empty(), "Response body is empty");

    let parsed: Result<serde_json::Value, _> = serde_json::from_str(&text);
    assert!(parsed.is_ok(), "Response is not valid JSON");

    let body = parsed?;
    assert!(
        body.get("id").is_some() || body.get("result").is_some(),
        "JSON missing id field"
    );

    cleanup.track_task_id(task_id);
    cleanup.cleanup_all().await?;

    println!("✓ Response contains valid JSON");
    Ok(())
}

#[tokio::test]
async fn test_agent_processes_request() -> Result<()> {
    let ctx = TestContext::new().await?;
    let fingerprint = ctx.fingerprint().to_string();
    let mut cleanup = TestCleanup::new(ctx.db.clone());
    cleanup.track_fingerprint(fingerprint.clone());

    let token = ctx.get_anonymous_token().await?;

    // Create context first
    let context_id = ctx.create_context(&token, "Process this request").await?;

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
        create_a2a_message("Process this request", &context_id);

    let agent_url = format!("/api/v1/agents/{}/", agent_id);
    let response = ctx
        .make_authenticated_json_request(reqwest::Method::POST, &agent_url, &token, message)
        .await?;

    assert!(
        response.status().is_success(),
        "Agent failed to process request"
    );

    wait_for_async_processing().await;

    use systemprompt_core_database::DatabaseQueryEnum;
    let query = DatabaseQueryEnum::GetTask.get(ctx.db.as_ref());
    let rows = ctx.db.fetch_all(&query, &[&task_id]).await?;

    assert!(!rows.is_empty(), "Task not created");

    cleanup.track_task_id(task_id);
    cleanup.cleanup_all().await?;

    println!("✓ Agent successfully processed request");
    Ok(())
}
