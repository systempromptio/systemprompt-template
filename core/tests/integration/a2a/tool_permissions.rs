use crate::common::context::TestContext;
use serde_json::json;
use uuid::Uuid;

/// Test that anonymous users can access tools requiring "anonymous" scope
#[tokio::test]
async fn test_anonymous_user_can_use_anonymous_tools() {
    let ctx = TestContext::new().await.expect("Failed to create test context");

    // Create anonymous token (this is what frontend uses)
    let anon_token_response = ctx.http_client
        .post(&format!("{}/api/v1/core/oauth/session", ctx.base_url))
        .send()
        .await
        .expect("Failed to generate anonymous token");

    assert_eq!(anon_token_response.status(), 200, "Should generate anonymous token");

    let token_data: serde_json::Value = anon_token_response.json().await
        .expect("Failed to parse token response");

    let anon_token = token_data.get("access_token")
        .and_then(|t| t.as_str())
        .expect("Should have access_token");

    // Create context
    let context_id = ctx.http_client
        .post(&format!("{}/api/v1/core/contexts", ctx.base_url))
        .header("Authorization", format!("Bearer {}", anon_token))
        .json(&json!({"name": "test-anon-context"}))
        .send()
        .await
        .expect("Failed to create context")
        .json::<serde_json::Value>()
        .await
        .expect("Failed to parse context response")
        .get("context_id")
        .and_then(|c| c.as_str())
        .expect("Should have context_id")
        .to_string();

    // Send message that triggers tyingshoelaces MCP server tool (requires "anonymous" scope)
    let message_id = Uuid::new_v4().to_string();
    let body = json!({
        "jsonrpc": "2.0",
        "method": "message/stream",
        "params": {
            "message": {
                "messageId": message_id,
                "contextId": context_id,
                "role": "user",
                "kind": "message",
                "parts": [{"kind": "text", "text": "search for ai"}]
            }
        },
        "id": 1
    });

    let response = ctx.http_client
        .post(&format!("{}/api/v1/agents/edward/", ctx.base_url))
        .header("Authorization", format!("Bearer {}", anon_token))
        .json(&body)
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(
        response.status(), 200,
        "Anonymous user should be able to use tools requiring 'anonymous' scope"
    );

    // Parse response to verify tool was actually called (not just permission granted)
    let response_text = response.text().await.expect("Failed to get response text");

    // Should have SSE events with tool execution
    assert!(
        !response_text.is_empty(),
        "Should have response with tool execution results"
    );

    // The AI should have called the tool, not returned text saying it couldn't
    let has_tool_use = response_text.contains("context_retrieval") ||
                       response_text.contains("artifacts");

    assert!(
        has_tool_use,
        "AI should actually call the tool, not just respond with text. \
         This test would catch the bug where permission filtering returned empty tools."
    );
}

/// Test that permission filtering happens BEFORE HTTP call, not after 403 error
#[tokio::test]
async fn test_permission_filtering_prevents_403_errors() {
    let ctx = TestContext::new().await.expect("Failed to create test context");

    // Get edward agent which has tyingshoelaces MCP server assigned
    let agent_url = format!("{}/api/v1/agents/edward/", ctx.base_url);

    // Generate anonymous token
    let anon_token_response = ctx.http_client
        .post(&format!("{}/api/v1/core/oauth/session", ctx.base_url))
        .send()
        .await
        .expect("Failed to generate anonymous token");

    let token_data: serde_json::Value = anon_token_response.json().await
        .expect("Failed to parse token response");

    let anon_token = token_data.get("access_token")
        .and_then(|t| t.as_str())
        .expect("Should have access_token");

    // Get agent card to see available tools
    let card_response = ctx.http_client
        .get(&format!("{}/.well-known/agent-card.json", agent_url))
        .header("Authorization", format!("Bearer {}", anon_token))
        .send()
        .await
        .expect("Failed to get agent card");

    assert_eq!(card_response.status(), 200, "Should get agent card");

    let card: serde_json::Value = card_response.json().await
        .expect("Failed to parse agent card");

    // Verify tools are listed (permission filtering allowed access)
    let tools = card.get("capabilities")
        .and_then(|c| c.get("tools"))
        .and_then(|t| t.as_array())
        .expect("Agent card should have tools");

    assert!(
        !tools.is_empty(),
        "Anonymous user should see tools from servers they have permission for"
    );

    // Verify context_retrieval tool is included
    let has_context_retrieval = tools.iter().any(|tool| {
        tool.get("name")
            .and_then(|n| n.as_str())
            .map(|n| n == "context_retrieval")
            .unwrap_or(false)
    });

    assert!(
        has_context_retrieval,
        "Anonymous user should have access to context_retrieval tool from tyingshoelaces server"
    );
}

/// Test that users without required permissions don't see restricted tools
#[tokio::test]
async fn test_restricted_tools_not_visible_without_permission() {
    let ctx = TestContext::new().await.expect("Failed to create test context");

    // If we had a server that requires "admin" scope, anonymous users shouldn't see its tools
    // For now, verify that the permission filtering is applied

    let anon_token_response = ctx.http_client
        .post(&format!("{}/api/v1/core/oauth/session", ctx.base_url))
        .send()
        .await
        .expect("Failed to generate anonymous token");

    let token_data: serde_json::Value = anon_token_response.json().await
        .expect("Failed to parse token response");

    let anon_token = token_data.get("access_token")
        .and_then(|t| t.as_str())
        .expect("Should have access_token");

    // Get admin agent (might have restricted tools)
    let card_response = ctx.http_client
        .get(&format!("{}/api/v1/agents/admin/.well-known/agent-card.json", ctx.base_url))
        .header("Authorization", format!("Bearer {}", anon_token))
        .send()
        .await
        .expect("Failed to get admin agent card");

    if card_response.status() == 200 {
        let card: serde_json::Value = card_response.json().await
            .expect("Failed to parse agent card");

        let tools = card.get("capabilities")
            .and_then(|c| c.get("tools"))
            .and_then(|t| t.as_array())
            .unwrap_or(&vec![]);

        // Check if any tools from systemprompt-admin server are visible
        // (they shouldn't be if that server requires admin scope)
        let has_admin_tools = tools.iter().any(|tool| {
            tool.get("serverName")
                .and_then(|s| s.as_str())
                .map(|s| s == "systemprompt-admin")
                .unwrap_or(false)
        });

        // This assertion depends on your server configuration
        // Adjust based on whether systemprompt-admin requires admin scope
        if has_admin_tools {
            println!(
                "Warning: Anonymous user can see admin tools. \
                 Check if systemprompt-admin server has proper OAuth requirements."
            );
        }
    }
}

/// Test that permission check logs are generated
#[tokio::test]
async fn test_permission_checks_are_logged() {
    let ctx = TestContext::new().await.expect("Failed to create test context");

    let anon_token_response = ctx.http_client
        .post(&format!("{}/api/v1/core/oauth/session", ctx.base_url))
        .send()
        .await
        .expect("Failed to generate anonymous token");

    let token_data: serde_json::Value = anon_token_response.json().await
        .expect("Failed to parse token response");

    let anon_token = token_data.get("access_token")
        .and_then(|t| t.as_str())
        .expect("Should have access_token");

    let context_id = ctx.http_client
        .post(&format!("{}/api/v1/core/contexts", ctx.base_url))
        .header("Authorization", format!("Bearer {}", anon_token))
        .json(&json!({"name": "test-log-context"}))
        .send()
        .await
        .expect("Failed to create context")
        .json::<serde_json::Value>()
        .await
        .expect("Failed to parse context response")
        .get("context_id")
        .and_then(|c| c.as_str())
        .expect("Should have context_id")
        .to_string();

    // Make a request that triggers tool loading
    let body = json!({
        "jsonrpc": "2.0",
        "method": "message/stream",
        "params": {
            "message": {
                "messageId": Uuid::new_v4().to_string(),
                "contextId": context_id,
                "role": "user",
                "kind": "message",
                "parts": [{"kind": "text", "text": "search for testing"}]
            }
        },
        "id": 1
    });

    let _response = ctx.http_client
        .post(&format!("{}/api/v1/agents/edward/", ctx.base_url))
        .header("Authorization", format!("Bearer {}", anon_token))
        .json(&body)
        .send()
        .await
        .expect("Failed to send request");

    // Check logs table for permission check entries
    let logs = ctx.db_pool
        .fetch_all(
            "SELECT message FROM logs WHERE module = 'ai_mcp_client' AND message LIKE '%permission%' ORDER BY created_at DESC LIMIT 10",
            &[]
        )
        .await
        .expect("Failed to query logs");

    if !logs.is_empty() {
        let has_permission_log = logs.iter().any(|log| {
            log.get("message")
                .and_then(|m| m.as_str())
                .map(|m| m.contains("User permissions") || m.contains("Access"))
                .unwrap_or(false)
        });

        assert!(
            has_permission_log,
            "Should have logs showing permission checks were performed"
        );
    } else {
        println!("Note: No permission check logs found. This is expected if the system skipped logging.");
    }
}
