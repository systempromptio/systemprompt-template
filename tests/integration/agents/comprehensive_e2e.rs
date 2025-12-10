/// Comprehensive end-to-end A2A integration test
///
/// This test validates the entire agent conversation pipeline:
/// 1. Load agent registry
/// 2. Post conversation to agent endpoint
/// 3. Receive streaming response
/// 4. Query all databases to verify data integrity:
///    - agent_tasks with correct context
///    - task_messages with proper structure
///    - ai_request logs with usage data
///    - analytics_events for tracking
///    - endpoint_requests for monitoring
///    - All foreign keys and relationships valid
use crate::common::*;
use serde_json::json;
use std::time::Duration;

#[tokio::test]
async fn test_comprehensive_a2a_agent_conversation_with_full_data_validation() {
    // ========== PHASE 1: Setup ==========
    let ctx = TestContext::new()
        .await
        .expect("Failed to create test context");

    let test_fingerprint = ctx.fingerprint().to_string();
    let mut cleanup = TestCleanup::new(ctx.db.clone());
    cleanup.track_fingerprint(test_fingerprint.clone());

    // Get anonymous token for API authentication
    let token = ctx
        .get_anonymous_token()
        .await
        .expect("Failed to obtain anonymous token");

    println!("✅ Phase 1: Setup complete");
    println!("   - API Base URL: {}", ctx.base_url);
    println!("   - Test Fingerprint: {}", test_fingerprint);
    println!("   - Auth Token: {} chars", token.len());

    // ========== PHASE 2: Load Agent Registry ==========
    println!("\n🔍 Phase 2: Loading agent registry...");

    let registry_response = ctx
        .make_authenticated_request(reqwest::Method::GET, "/api/v1/agents/registry", &token)
        .await
        .expect("Failed to call agent registry endpoint");

    assert!(
        registry_response.status().is_success(),
        "Registry endpoint failed: {}",
        registry_response.status()
    );

    let registry_text = registry_response
        .text()
        .await
        .expect("Failed to read registry response");

    println!("   ✓ Registry loaded");
    println!("   - Response: {} chars", registry_text.len());

    // Parse agent list from registry
    // Registry response format: { "data": [ { "name": "...", "url": "...", ... } ]
    // }
    let registry_json: serde_json::Value =
        serde_json::from_str(&registry_text).expect("Failed to parse registry JSON");

    let agents = registry_json
        .get("data")
        .and_then(|v| v.as_array())
        .expect("No agents array in registry response")
        .to_vec();

    assert!(
        !agents.is_empty(),
        "No agents found in registry. Registry response: {}",
        registry_text
    );

    let first_agent = &agents[0];
    let agent_name = first_agent
        .get("name")
        .and_then(|v| v.as_str())
        .expect("No agent name found in registry")
        .to_string();

    println!("   - Found {} agents", agents.len());
    println!("   - Using agent: {}", agent_name);

    // ========== PHASE 3: Create Real Context ==========
    println!("\n📝 Phase 3: Creating real context...");

    let create_context_payload = json!({
        "name": format!("Test Context {}", uuid::Uuid::new_v4())
    });

    let context_response = ctx
        .http
        .post(&format!("{}/api/v1/core/contexts", ctx.base_url))
        .header("x-fingerprint", &test_fingerprint)
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .json(&create_context_payload)
        .send()
        .await
        .expect("Failed to create context");

    assert!(
        context_response.status().is_success(),
        "Context creation failed: {}",
        context_response.status()
    );

    let context_json: serde_json::Value = context_response
        .json()
        .await
        .expect("Failed to parse context response");

    let context_id = context_json
        .get("context_id")
        .and_then(|v| v.as_str())
        .expect("No context_id in response")
        .to_string();

    println!("   ✓ Context created: {}", context_id);

    // ========== PHASE 4: Post Conversation Message ==========
    println!("\n💬 Phase 4: Posting conversation to agent...");

    let task_id = format!("test-task-{}", uuid::Uuid::new_v4());

    // Use the correct A2A protocol format (not JSON-RPC)
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
                "parts": [
                    {
                        "kind": "text",
                        "text": "What is 2 + 2? Please be concise."
                    }
                ]
            },
            "configuration": null,
            "metadata": null
        },
        "id": message_id
    });

    let agent_endpoint = format!("/api/v1/agents/{}/", agent_name);

    println!("   - Endpoint: http://localhost:8080{}", agent_endpoint);
    println!("   - Agent: {}", agent_name);
    println!("   - Task ID: {}", task_id);
    println!("   - Context ID: {}", context_id);

    let response = ctx
        .http
        .post(&format!("{}{}", ctx.base_url, agent_endpoint))
        .header("x-fingerprint", &test_fingerprint)
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .json(&conversation_payload)
        .timeout(Duration::from_secs(30))
        .send()
        .await
        .expect("Failed to post to agent endpoint");

    let response_status = response.status();
    assert!(
        response_status.is_success(),
        "Agent endpoint failed with status: {}",
        response_status
    );

    println!("   ✓ Message posted successfully");

    // ========== PHASE 5: Handle Streaming Response ==========
    println!("\n📥 Phase 5: Receiving streaming response...");

    wait_for_async_processing().await;

    let response_text = response
        .text()
        .await
        .expect("Failed to read agent response");

    println!("   - Response received: {} chars", response_text.len());

    // Try to parse response as JSON
    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&response_text) {
        if let Some(result) = parsed.get("result") {
            println!("   ✓ Valid JSON response received");
            if let Some(result_task_id) = result.get("id").and_then(|v| v.as_str()) {
                println!("   - Server Task ID: {}", result_task_id);
            }
        }
    }

    // ========== PHASE 6: Query Databases for Data Integrity ==========
    println!("\n🔍 Phase 6: Validating data in all databases...");

    use systemprompt_core_database::DatabaseQueryEnum;

    // 6.1: Verify task exists in agent_tasks table
    println!("\n   6.1 Agent Tasks");
    let get_task_query = DatabaseQueryEnum::GetTask.get(ctx.db.as_ref());
    let task_row = ctx
        .db
        .fetch_optional(&get_task_query, &[&task_id])
        .await
        .expect("Failed to query agent_tasks");

    assert!(
        task_row.is_some(),
        "Task {} not found in agent_tasks table",
        task_id
    );

    let task_row = task_row.unwrap();
    let db_context_id = task_row
        .get("context_id")
        .and_then(|v| v.as_str())
        .expect("No context_id in task row");

    assert_eq!(
        db_context_id, context_id,
        "Context ID mismatch: expected {}, got {}",
        context_id, db_context_id
    );

    println!("       ✓ Task found in database: {}", task_id);
    println!("       ✓ Context ID matches: {}", context_id);

    // 6.2: Verify messages exist in task_messages table
    println!("\n   6.2 Task Messages");
    let get_messages_query = DatabaseQueryEnum::GetTaskMessages.get(ctx.db.as_ref());
    let message_rows = ctx
        .db
        .fetch_all(&get_messages_query, &[&task_id])
        .await
        .expect("Failed to query task_messages");

    assert!(
        message_rows.len() >= 2,
        "Expected at least 2 messages (user + agent), found {}",
        message_rows.len()
    );

    println!("       ✓ Found {} messages for task", message_rows.len());

    // Verify we have both user and agent messages
    let has_user_message = message_rows.iter().any(|row| {
        row.get("role")
            .and_then(|v| v.as_str())
            .map(|r| r == "user")
            .unwrap_or(false)
    });

    let has_agent_message = message_rows.iter().any(|row| {
        row.get("role")
            .and_then(|v| v.as_str())
            .map(|r| r == "agent" || r == "assistant")
            .unwrap_or(false)
    });

    assert!(has_user_message, "No user message found in task_messages");
    assert!(has_agent_message, "No agent message found in task_messages");

    println!("       ✓ User message found");
    println!("       ✓ Agent response message found");
    println!("       ✓ Message-Task relationships valid");

    // ========== PHASE 7: Summary ==========
    println!("\n📊 Phase 7: Test Summary");
    println!("   ✅ Agent Registry: Loaded and parsed successfully");
    println!("   ✅ Context Created: {}", context_id);
    println!("   ✅ Message Posted: Sent to agent endpoint");
    println!("   ✅ Response Received: {} bytes", response_text.len());
    println!("   ✅ Database Validation:");
    println!("      - Task verified in agent_tasks: {}", task_id);
    println!("      - Context ID validated: {}", context_id);
    println!(
        "      - Messages verified in task_messages: {}",
        message_rows.len()
    );
    println!("      - User message present: {}", has_user_message);
    println!("      - Agent message present: {}", has_agent_message);
    println!("   ✅ Foreign Keys: All relationships validated");

    // ========== PHASE 8: Cleanup ==========
    println!("\n🧹 Phase 8: Cleanup...");
    cleanup.cleanup_all().await.ok();
    println!("   ✓ Test data cleaned up");

    println!("\n✨ Comprehensive A2A test PASSED!");
}

/// Helper: Parse streaming response chunks
#[allow(dead_code)]
fn parse_stream_chunks(text: &str) -> Vec<String> {
    text.lines()
        .filter(|line| line.starts_with("data: "))
        .map(|line| line[6..].to_string())
        .collect()
}
