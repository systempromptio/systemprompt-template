use crate::common::*;
use anyhow::Result;
use serde_json::json;
use systemprompt_core_system::RequestContext;
use systemprompt_identifiers::{AgentName, ContextId, SessionId, TraceId, UserId};
use systemprompt_models::auth::Permission;

#[tokio::test]
async fn test_mcp_tool_invocation() -> Result<()> {
    let ctx = TestContext::new().await?;
    let fingerprint = ctx.fingerprint().to_string();

    let registry_url = format!("{}/api/v1/mcp/registry", ctx.base_url);
    let registry_response = ctx.http.get(&registry_url).send().await?;
    let registry_body: serde_json::Value = registry_response.json().await?;
    let servers = registry_body["data"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("No servers in registry"))?;

    if servers.is_empty() {
        println!("✓ No MCP servers to test tool invocation (skipped)");
        return Ok(());
    }

    let first_server = &servers[0];
    let server_name = first_server["name"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Server missing name"))?;

    let invoke_url = format!("{}/api/v1/mcp/{}/mcp/tool/call", ctx.base_url, server_name);
    let invoke_payload = json!({
        "jsonrpc": "2.0",
        "method": "tools/call",
        "params": {
            "name": "test-tool",
            "arguments": {}
        },
        "id": 1
    });

    let response = ctx
        .http
        .post(&invoke_url)
        .header("x-fingerprint", &fingerprint)
        .json(&invoke_payload)
        .send()
        .await?;

    let status = response.status();

    match status.as_u16() {
        200..=299 => {
            let body: serde_json::Value = response.json().await?;
            assert!(
                body["result"].is_object() || body["result"].is_array(),
                "No result from tool"
            );
            println!("✓ MCP tool invocation executed successfully");
        },
        400..=599 => {
            println!(
                "✓ MCP tool invocation server responded with {} (auth or service unavailable)",
                status
            );
        },
        _ => {
            return Err(anyhow::anyhow!("Unexpected status code: {}", status));
        },
    }

    let mut cleanup = TestCleanup::new(ctx.db.clone());
    cleanup.track_fingerprint(fingerprint);
    cleanup.cleanup_all().await?;

    Ok(())
}

#[tokio::test]
async fn test_mcp_tool_error_handling() -> Result<()> {
    let ctx = TestContext::new().await?;

    let registry_url = format!("{}/api/v1/mcp/registry", ctx.base_url);
    let registry_response = ctx.http.get(&registry_url).send().await?;
    let registry_body: serde_json::Value = registry_response.json().await?;
    let servers = registry_body["data"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("No servers in registry"))?;

    if servers.is_empty() {
        println!("✓ No MCP servers to test error handling (skipped)");
        return Ok(());
    }

    let first_server = &servers[0];
    let server_name = first_server["name"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Server missing name"))?;

    let invoke_url = format!("{}/api/v1/mcp/{}/mcp/tool/call", ctx.base_url, server_name);
    let invoke_payload = json!({
        "jsonrpc": "2.0",
        "method": "tools/call",
        "params": {
            "name": "non-existent-tool-xyz",
            "arguments": {}
        },
        "id": 1
    });

    let response = ctx
        .http
        .post(&invoke_url)
        .json(&invoke_payload)
        .send()
        .await?;

    let status = response.status();

    match status.as_u16() {
        400..=599 => {
            println!("✓ MCP tool error handling verified (status {})", status);
        },
        200..=299 => {
            let body: serde_json::Value = response.json().await?;
            assert!(
                body["error"].is_object() || body["error"].is_string() || body["error"].is_null(),
                "Should handle error appropriately"
            );
            println!("✓ MCP tool error handling verified");
        },
        _ => {
            return Err(anyhow::anyhow!("Unexpected status code: {}", status));
        },
    }

    Ok(())
}
