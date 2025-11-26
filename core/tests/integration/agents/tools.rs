use crate::common::*;
use anyhow::Result;

#[tokio::test]
async fn test_agent_tools_available() -> Result<()> {
    let ctx = TestContext::new().await?;

    let token = ctx.get_anonymous_token().await?;

    let response = ctx
        .make_authenticated_request(reqwest::Method::GET, "/api/v1/agents/registry", &token)
        .await?;

    assert_eq!(response.status(), 200, "Registry endpoint failed");

    let body: serde_json::Value = response.json().await?;
    let agents = body["data"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("No agents in registry"))?;

    for agent in agents {
        let agent_name = agent["name"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Agent missing name"))?;

        // Check that agent has MCP tools capability in extensions
        let capabilities = agent
            .get("capabilities")
            .ok_or_else(|| anyhow::anyhow!("Agent {} missing capabilities", agent_name))?;

        let extensions = capabilities
            .get("extensions")
            .and_then(|e| e.as_array())
            .ok_or_else(|| {
                anyhow::anyhow!("Agent {} missing capabilities.extensions", agent_name)
            })?;

        // Find the MCP tools extension
        let has_mcp_tools = extensions.iter().any(|ext| {
            ext.get("uri")
                .and_then(|u| u.as_str())
                .map(|u| u.contains("mcp-tools"))
                .unwrap_or(false)
        });

        assert!(
            has_mcp_tools,
            "Agent {} missing MCP tools capability",
            agent_name
        );

        println!("✓ Agent {} has MCP tools available", agent_name);
    }

    println!("✓ All agent tools are available");
    Ok(())
}

#[tokio::test]
async fn test_agent_capabilities() -> Result<()> {
    let ctx = TestContext::new().await?;

    let token = ctx.get_anonymous_token().await?;

    let response = ctx
        .make_authenticated_request(reqwest::Method::GET, "/api/v1/agents/registry", &token)
        .await?;

    let body: serde_json::Value = response.json().await?;
    let agents = body["data"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("No agents in registry"))?;

    assert!(!agents.is_empty(), "No agents available");

    for agent in agents {
        assert!(agent.get("name").is_some(), "Agent missing name");
        assert!(
            agent.get("description").is_some(),
            "Agent missing description"
        );
        assert!(agent.get("url").is_some(), "Agent missing url");
        assert!(
            agent.get("capabilities").is_some(),
            "Agent missing capabilities"
        );

        let capabilities = agent["capabilities"]
            .as_object()
            .ok_or_else(|| anyhow::anyhow!("capabilities is not an object"))?;

        assert!(
            capabilities.get("streaming").is_some(),
            "Agent missing streaming capability"
        );
    }

    println!("✓ Agent capabilities verified");
    Ok(())
}

#[tokio::test]
async fn test_multiple_agents_in_registry() -> Result<()> {
    let ctx = TestContext::new().await?;

    let token = ctx.get_anonymous_token().await?;

    let response = ctx
        .make_authenticated_request(reqwest::Method::GET, "/api/v1/agents/registry", &token)
        .await?;

    assert_eq!(response.status(), 200, "Registry endpoint failed");

    let body: serde_json::Value = response.json().await?;
    let agents = body["data"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("No agents in registry"))?;

    assert!(!agents.is_empty(), "No agents in registry");

    let mut agent_ids = vec![];
    for agent in agents {
        if let Some(id) = agent["name"].as_str() {
            agent_ids.push(id.to_string());
        }
    }

    assert!(!agent_ids.is_empty(), "No valid agent IDs found");
    println!("✓ Found {} agents in registry", agent_ids.len());
    Ok(())
}
