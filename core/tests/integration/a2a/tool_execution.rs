use crate::common::context::TestContext;
use serde_json::json;
use uuid::Uuid;

/// Test that A2A messages with contextId properly propagate context through tool execution
#[tokio::test]
async fn test_tool_execution_with_context_propagation() {
    let ctx = TestContext::new().await.expect("Failed to create test context");

    // Create a proper context first
    let context_id = ctx.create_context().await.expect("Failed to create context");

    // Send message that should trigger the context_retrieval tool
    let message_id = Uuid::new_v4().to_string();
    let body = json!({
        "jsonrpc": "2.0",
        "method": "message/stream",
        "params": {
            "message": {
                "messageId": message_id,
                "contextId": context_id,  // This MUST propagate through to tool execution
                "role": "user",
                "kind": "message",
                "parts": [{"kind": "text", "text": "search for ai"}]
            }
        },
        "id": 1
    });

    let response = ctx.http_client
        .post(&format!("{}/api/v1/agents/edward/", ctx.base_url))
        .header("Authorization", format!("Bearer {}", ctx.auth_token))
        .json(&body)
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), 200, "Request should succeed");

    // Parse SSE stream to get final task
    let response_text = response.text().await.expect("Failed to get response text");

    // The response should contain task events with artifacts
    assert!(!response_text.is_empty(), "Response should not be empty");

    // Find the final task in SSE events
    let task_event = response_text
        .lines()
        .filter(|line| line.starts_with("data: "))
        .last()
        .expect("Should have at least one data event");

    let json_str = &task_event[6..]; // Skip "data: " prefix
    let task: serde_json::Value = serde_json::from_str(json_str)
        .expect("Should be valid JSON");

    // Verify the tool was actually called
    let history = task.get("history")
        .and_then(|h| h.as_array())
        .expect("Task should have history");

    // Should have user message + agent response with tool results
    assert!(history.len() >= 2, "Should have at least user message and agent response");

    // Check for artifacts (proves tool executed and artifact creation succeeded)
    let artifacts = task.get("artifacts")
        .and_then(|a| a.as_array());

    if let Some(artifacts) = artifacts {
        assert!(!artifacts.is_empty(), "Tool execution should produce artifacts");

        // Verify artifact has proper metadata with context_id
        for artifact in artifacts {
            let metadata = artifact.get("metadata")
                .expect("Artifact should have metadata");

            let artifact_context_id = metadata.get("context_id")
                .or_else(|| metadata.get("contextId"))
                .and_then(|c| c.as_str())
                .expect("Artifact metadata should have context_id");

            assert_eq!(
                artifact_context_id, context_id,
                "Artifact context_id should match message context_id"
            );

            // Verify task_id is also present
            let task_id = metadata.get("task_id")
                .or_else(|| metadata.get("taskId"))
                .and_then(|t| t.as_str())
                .expect("Artifact metadata should have task_id");

            assert!(!task_id.is_empty(), "Task ID should not be empty");
        }
    }

    // Query tool execution records to verify context_id was stored correctly
    let tool_executions = ctx.db_pool
        .fetch_all(
            "SELECT context_id, tool_name FROM mcp_tool_executions WHERE context_id = $1",
            &[&context_id]
        )
        .await
        .expect("Failed to query tool executions");

    assert!(
        !tool_executions.is_empty(),
        "Should have tool execution records with correct context_id"
    );

    for execution in &tool_executions {
        let exec_context_id = execution.get("context_id")
            .and_then(|v| v.as_str())
            .expect("Execution should have context_id");

        assert_eq!(
            exec_context_id, context_id,
            "Tool execution should be stored with correct context_id from A2A message body"
        );

        assert!(
            !exec_context_id.is_empty(),
            "Tool execution context_id should never be empty - this was the bug"
        );
    }
}

/// Test that tool execution without proper context fails gracefully
#[tokio::test]
async fn test_tool_execution_requires_valid_context() {
    let ctx = TestContext::new().await.expect("Failed to create test context");

    let message_id = Uuid::new_v4().to_string();
    let body = json!({
        "jsonrpc": "2.0",
        "method": "message/stream",
        "params": {
            "message": {
                "messageId": message_id,
                "contextId": "",  // Empty context_id should be rejected
                "role": "user",
                "kind": "message",
                "parts": [{"kind": "text", "text": "search for ai"}]
            }
        },
        "id": 1
    });

    let response = ctx.http_client
        .post(&format!("{}/api/v1/agents/edward/", ctx.base_url))
        .header("Authorization", format!("Bearer {}", ctx.auth_token))
        .json(&body)
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(
        response.status(), 400,
        "Empty contextId should be rejected with 400"
    );

    let error_body: serde_json::Value = response.json().await
        .expect("Should return JSON error");

    let error = error_body.get("error")
        .expect("Should have error field");

    let error_message = error.get("message")
        .and_then(|m| m.as_str())
        .expect("Error should have message");

    assert!(
        error_message.contains("contextId") || error_message.contains("empty"),
        "Error should mention contextId issue"
    );
}

/// Test that streaming and non-streaming paths both propagate context properly
#[tokio::test]
async fn test_context_propagation_both_paths() {
    let ctx = TestContext::new().await.expect("Failed to create test context");
    let context_id = ctx.create_context().await.expect("Failed to create context");

    // Test streaming path (message/stream)
    let streaming_body = json!({
        "jsonrpc": "2.0",
        "method": "message/stream",
        "params": {
            "message": {
                "messageId": Uuid::new_v4().to_string(),
                "contextId": context_id,
                "role": "user",
                "kind": "message",
                "parts": [{"kind": "text", "text": "search for rust"}]
            }
        },
        "id": 1
    });

    let streaming_response = ctx.http_client
        .post(&format!("{}/api/v1/agents/edward/", ctx.base_url))
        .header("Authorization", format!("Bearer {}", ctx.auth_token))
        .json(&streaming_body)
        .send()
        .await
        .expect("Streaming request failed");

    assert_eq!(streaming_response.status(), 200, "Streaming should succeed");

    // Test non-streaming path (message/send)
    let non_streaming_body = json!({
        "jsonrpc": "2.0",
        "method": "message/send",
        "params": {
            "message": {
                "messageId": Uuid::new_v4().to_string(),
                "contextId": context_id,
                "role": "user",
                "kind": "message",
                "parts": [{"kind": "text", "text": "search for python"}]
            }
        },
        "id": 2
    });

    let non_streaming_response = ctx.http_client
        .post(&format!("{}/api/v1/agents/edward/", ctx.base_url))
        .header("Authorization", format!("Bearer {}", ctx.auth_token))
        .json(&non_streaming_body)
        .send()
        .await
        .expect("Non-streaming request failed");

    assert_eq!(non_streaming_response.status(), 200, "Non-streaming should succeed");

    // Both should have stored tool executions with correct context_id
    let executions = ctx.db_pool
        .fetch_all(
            "SELECT tool_name, context_id FROM mcp_tool_executions WHERE context_id = $1",
            &[&context_id]
        )
        .await
        .expect("Failed to query executions");

    // Should have at least 2 executions (one from each path)
    assert!(
        executions.len() >= 2,
        "Both streaming and non-streaming should create tool execution records"
    );

    for execution in &executions {
        let exec_context_id = execution.get("context_id")
            .and_then(|v| v.as_str())
            .expect("Should have context_id");

        assert_eq!(
            exec_context_id, context_id,
            "Both code paths should store correct context_id"
        );
    }
}
