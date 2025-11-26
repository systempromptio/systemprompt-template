use crate::common::*;
use anyhow::Result;
use serde_json::json;

#[tokio::test]
async fn test_initiate_agent_conversation() -> Result<()> {
    let ctx = TestContext::new().await?;
    let fingerprint = ctx.fingerprint().to_string();
    let mut cleanup = TestCleanup::new(ctx.db.clone());
    cleanup.track_fingerprint(fingerprint.clone());

    let token = ctx.get_anonymous_token().await?;

    // Create context first
    let context_id = ctx.create_context(&token, "Hello, can you help?").await?;

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

    let task_id = format!("test-task-{}", uuid::Uuid::new_v4());
    let message_id = format!("msg-{}", uuid::Uuid::new_v4());

    let conversation_payload = json!({
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
                    "text": "Hello, can you help?"
                }]
            }
        },
        "id": message_id
    });

    let agent_url = format!("/api/v1/agents/{}/", agent_id);
    let response = ctx
        .make_authenticated_json_request(
            reqwest::Method::POST,
            &agent_url,
            &token,
            conversation_payload,
        )
        .await?;

    assert_eq!(response.status(), 200, "Conversation request failed");

    let body: serde_json::Value = response.json().await?;
    assert!(
        body.get("result").is_some() || body.get("id").is_some(),
        "Response missing result"
    );

    wait_for_async_processing().await;

    use systemprompt_core_database::DatabaseQueryEnum;
    let query = DatabaseQueryEnum::GetTask.get(ctx.db.as_ref());
    let rows = ctx.db.fetch_all(&query, &[&task_id]).await?;

    assert!(!rows.is_empty(), "Task not created in database");

    cleanup.track_task_id(task_id.clone());
    cleanup.cleanup_all().await?;

    println!("✓ Agent conversation initiated and task created");
    Ok(())
}

#[tokio::test]
async fn test_conversation_requires_message() -> Result<()> {
    let ctx = TestContext::new().await?;

    let token = ctx.get_anonymous_token().await?;

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

    let message_id = format!("msg-{}", uuid::Uuid::new_v4());

    let empty_message = json!({
        "jsonrpc": "2.0",
        "method": "message/send",
        "params": {
            "message": {
                "messageId": message_id,
                "role": "user",
                "kind": "message",
                "parts": []
            }
        },
        "id": message_id
    });

    let agent_url = format!("/api/v1/agents/{}/", agent_id);
    let response = ctx
        .make_authenticated_json_request(reqwest::Method::POST, &agent_url, &token, empty_message)
        .await?;

    assert!(
        !response.status().is_success(),
        "Empty message should be rejected"
    );

    println!("✓ Empty message correctly rejected");
    Ok(())
}

#[tokio::test]
async fn test_send_message_to_agent() -> Result<()> {
    let ctx = TestContext::new().await?;
    let fingerprint = ctx.fingerprint().to_string();
    let mut cleanup = TestCleanup::new(ctx.db.clone());
    cleanup.track_fingerprint(fingerprint.clone());

    let token = ctx.get_anonymous_token().await?;

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

    let task_id = format!("test-task-{}", uuid::Uuid::new_v4());
    let context_id = format!("test-context-{}", uuid::Uuid::new_v4());
    let message_id = format!("msg-{}", uuid::Uuid::new_v4());

    let message = json!({
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
                    "text": "What is 2 + 2?"
                }]
            }
        },
        "id": message_id
    });

    let agent_url = format!("/api/v1/agents/{}/", agent_id);
    let response = ctx
        .make_authenticated_json_request(reqwest::Method::POST, &agent_url, &token, message)
        .await?;

    assert!(response.status().is_success(), "Message posting failed");

    let body: serde_json::Value = response.json().await?;
    assert!(
        body.get("result").is_some() || body.get("id").is_some(),
        "No response ID"
    );

    cleanup.track_task_id(task_id);
    cleanup.cleanup_all().await?;

    println!("✓ Message successfully sent to agent");
    Ok(())
}

#[tokio::test]
async fn test_multiple_message_exchange() -> Result<()> {
    let ctx = TestContext::new().await?;
    let fingerprint = ctx.fingerprint().to_string();
    let mut cleanup = TestCleanup::new(ctx.db.clone());
    cleanup.track_fingerprint(fingerprint.clone());

    let token = ctx.get_anonymous_token().await?;

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

    let agent_url = format!("/api/v1/agents/{}/", agent_id);

    let messages = vec!["What is AI?", "How does machine learning work?"];

    for message_text in &messages {
        let task_id = format!("test-task-{}", uuid::Uuid::new_v4());
        let context_id = format!("test-context-{}", uuid::Uuid::new_v4());
        let message_id = format!("msg-{}", uuid::Uuid::new_v4());

        let payload = json!({
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
                        "text": message_text
                    }]
                }
            },
            "id": message_id
        });

        let response = ctx
            .make_authenticated_json_request(reqwest::Method::POST, &agent_url, &token, payload)
            .await?;

        assert!(response.status().is_success(), "Message posting failed");

        cleanup.track_task_id(task_id.to_string());
    }

    wait_for_async_processing().await;

    cleanup.cleanup_all().await?;

    println!("✓ Multiple message exchange successful");
    Ok(())
}
