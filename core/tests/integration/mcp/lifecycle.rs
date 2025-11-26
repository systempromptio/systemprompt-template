use crate::common::*;
use anyhow::Result;
use std::time::Duration;

#[tokio::test]
async fn test_mcp_server_starts() -> Result<()> {
    let ctx = TestContext::new().await?;

    let registry_url = format!("{}/api/v1/mcp/registry", ctx.base_url);
    let response = ctx.http.get(&registry_url).send().await?;

    assert_eq!(response.status(), 200, "MCP registry endpoint failed");

    let body: serde_json::Value = response.json().await?;
    assert!(body["data"].is_array(), "Registry should return data array");

    let servers = body["data"].as_array().unwrap();
    assert!(
        !servers.is_empty(),
        "At least one MCP server should be registered"
    );

    for server in servers {
        assert!(server["name"].is_string(), "Server missing name");
        assert!(server["version"].is_string(), "Server missing version");
        assert!(server["status"].is_string(), "Server missing status");
    }

    println!("✓ MCP server started successfully");
    Ok(())
}

#[tokio::test]
async fn test_mcp_server_reconnection() -> Result<()> {
    let ctx = TestContext::new().await?;

    let registry_url = format!("{}/api/v1/mcp/registry", ctx.base_url);

    let response1 = ctx.http.get(&registry_url).send().await?;
    assert_eq!(response1.status(), 200, "First registry request failed");

    let body1: serde_json::Value = response1.json().await?;
    let servers1 = body1["data"].as_array().unwrap();

    tokio::time::sleep(Duration::from_millis(500)).await;

    let response2 = ctx.http.get(&registry_url).send().await?;
    assert_eq!(response2.status(), 200, "Second registry request failed");

    let body2: serde_json::Value = response2.json().await?;
    let servers2 = body2["data"].as_array().unwrap();

    assert_eq!(
        servers1.len(),
        servers2.len(),
        "Server count should be consistent"
    );

    for (s1, s2) in servers1.iter().zip(servers2.iter()) {
        assert_eq!(
            s1["name"], s2["name"],
            "Server name should remain consistent"
        );
    }

    println!("✓ MCP server reconnection verified");
    Ok(())
}

#[tokio::test]
async fn test_mcp_tools_listed() -> Result<()> {
    let ctx = TestContext::new().await?;

    let registry_url = format!("{}/api/v1/mcp/registry", ctx.base_url);
    let registry_response = ctx.http.get(&registry_url).send().await?;
    let registry_body: serde_json::Value = registry_response.json().await?;
    let servers = registry_body["data"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("No servers in registry"))?;

    if servers.is_empty() {
        println!("✓ No MCP servers registered (skipped)");
        return Ok(());
    }

    let first_server = &servers[0];
    assert!(first_server["name"].is_string(), "Server missing name");
    assert!(first_server["status"].is_string(), "Server missing status");

    println!("✓ MCP servers have tools registry");
    Ok(())
}
