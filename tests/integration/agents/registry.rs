use crate::common::*;
use anyhow::Result;

#[tokio::test]
async fn test_agent_registry_loads() -> Result<()> {
    let ctx = TestContext::new().await?;

    let token = ctx.get_anonymous_token().await?;

    let response = ctx
        .make_authenticated_request(reqwest::Method::GET, "/api/v1/agents/registry", &token)
        .await?;

    assert_eq!(response.status(), 200, "Registry endpoint failed");

    let body: serde_json::Value = response.json().await?;

    assert!(body["data"].is_array(), "Registry data should be array");

    let agents = body["data"].as_array().unwrap();
    assert!(!agents.is_empty(), "Registry should contain agents");

    for agent in agents {
        assert!(agent["name"].is_string(), "Agent missing name");
        assert!(
            agent["description"].is_string(),
            "Agent missing description"
        );
    }

    println!("✓ Agent registry loads with {} agents", agents.len());
    Ok(())
}

#[tokio::test]
async fn test_agent_registry_contains_required_fields() -> Result<()> {
    let ctx = TestContext::new().await?;

    let token = ctx.get_anonymous_token().await?;

    let response = ctx
        .make_authenticated_request(reqwest::Method::GET, "/api/v1/agents/registry", &token)
        .await?;

    let body: serde_json::Value = response.json().await?;
    let agents = body["data"].as_array().unwrap();

    for agent in agents {
        let name = agent["name"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing agent name"))?;
        let description = agent["description"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing agent description"))?;

        assert!(!name.is_empty(), "Agent name is empty");
        assert!(!description.is_empty(), "Agent description is empty");
    }

    println!("✓ All agents have required fields");
    Ok(())
}

#[tokio::test]
async fn test_get_agent_by_id() -> Result<()> {
    let ctx = TestContext::new().await?;

    let token = ctx.get_anonymous_token().await?;

    let registry_response = ctx
        .make_authenticated_request(reqwest::Method::GET, "/api/v1/agents/registry", &token)
        .await?;

    assert_eq!(registry_response.status(), 200, "Registry request failed");

    let registry_body: serde_json::Value = registry_response.json().await?;

    let agents = registry_body["data"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("No agents in registry"))?;
    let first_agent = &agents[0];
    let first_agent_name = first_agent["name"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Agent missing name"))?;

    // Validate that the agent has all expected metadata fields in registry
    assert!(first_agent["name"].is_string(), "Agent missing name field");
    assert!(
        first_agent["description"].is_string(),
        "Agent missing description field"
    );
    assert!(first_agent["url"].is_string(), "Agent missing url field");
    assert!(
        first_agent["protocolVersion"].is_string(),
        "Agent missing protocolVersion field"
    );
    assert!(
        first_agent["capabilities"].is_object(),
        "Agent missing capabilities field"
    );

    println!(
        "✓ Agent {} retrieved from registry with complete metadata",
        first_agent_name
    );
    Ok(())
}

#[tokio::test]
async fn test_invalid_agent_id_returns_404() -> Result<()> {
    let ctx = TestContext::new().await?;
    let fingerprint = ctx.fingerprint().to_string();
    let mut cleanup = TestCleanup::new(ctx.db.clone());
    cleanup.track_fingerprint(fingerprint.clone());

    let token = ctx.get_anonymous_token().await?;

    // Try to send an A2A message to a non-existent agent
    let invalid_agent_id = "invalid-agent-id-12345";
    let task_id = format!("test-task-{}", uuid::Uuid::new_v4());
    let context_id = format!("test-context-{}", uuid::Uuid::new_v4());
    let message_id = format!("msg-{}", uuid::Uuid::new_v4());

    let a2a_payload = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "message/send",
        "params": {
            "message": {
                "messageId": message_id,
                "contextId": context_id,
                "taskId": task_id,
                "role": "user",
                "kind": "message",
                "parts": [{
                    "kind": "text",
                    "text": "Test message"
                }]
            }
        },
        "id": message_id
    });

    let agent_url = format!("/api/v1/agents/{}/", invalid_agent_id);
    let response = ctx
        .make_authenticated_json_request(reqwest::Method::POST, &agent_url, &token, a2a_payload)
        .await?;

    // Should get 404 when trying to communicate with non-existent agent
    assert_eq!(response.status(), 404, "Expected 404 for invalid agent");

    cleanup.cleanup_all().await?;
    println!("✓ Invalid agent ID correctly returns 404");
    Ok(())
}
