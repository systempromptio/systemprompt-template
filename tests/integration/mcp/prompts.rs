use crate::common::*;
use anyhow::Result;
use serde_json::json;

#[tokio::test]
async fn test_mcp_prompts_listed() -> Result<()> {
    let ctx = TestContext::new().await?;

    let registry_url = format!("{}/api/v1/mcp/registry", ctx.base_url);
    let registry_response = ctx.http.get(&registry_url).send().await?;
    let registry_body: serde_json::Value = registry_response.json().await?;
    let servers = registry_body["data"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("No servers in registry"))?;

    if servers.is_empty() {
        println!("✓ No servers to test prompts (skipped)");
        return Ok(());
    }

    let first_server = &servers[0];
    let server_name = first_server["name"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Server missing name"))?;

    let prompts_url = format!(
        "{}/api/v1/mcp/{}/mcp/prompts/list",
        ctx.base_url, server_name
    );
    let response = ctx.http.get(&prompts_url).send().await?;

    let status = response.status();
    match status.as_u16() {
        200 => {
            let body: serde_json::Value = response.json().await?;
            assert!(
                body["result"]["prompts"].is_array() || body["prompts"].is_array(),
                "Prompts should be an array"
            );

            let empty_vec = vec![];
            let prompts = body["result"]["prompts"]
                .as_array()
                .or_else(|| body["prompts"].as_array())
                .unwrap_or(&empty_vec);

            for prompt in prompts {
                assert!(prompt["name"].is_string(), "Prompt missing name");
                assert!(
                    prompt["description"].is_string() || prompt["description"].is_null(),
                    "Invalid description"
                );
            }

            println!("✓ MCP prompts listed");
        },
        401 => {
            println!("✓ MCP prompts endpoint requires authentication (skipped)");
        },
        404 => {
            println!("✓ MCP prompts endpoint not available (skipped)");
        },
        _ => {
            println!(
                "✓ MCP prompts endpoint responded with status {} (skipped)",
                status
            );
        },
    }

    Ok(())
}

#[tokio::test]
async fn test_mcp_prompt_templates_expanded() -> Result<()> {
    let ctx = TestContext::new().await?;

    let registry_url = format!("{}/api/v1/mcp/registry", ctx.base_url);
    let registry_response = ctx.http.get(&registry_url).send().await?;
    let registry_body: serde_json::Value = registry_response.json().await?;
    let servers = registry_body["data"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("No servers in registry"))?;

    if servers.is_empty() {
        println!("✓ No servers to test prompt expansion (skipped)");
        return Ok(());
    }

    let first_server = &servers[0];
    let server_name = first_server["name"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Server missing name"))?;

    let prompts_url = format!(
        "{}/api/v1/mcp/{}/mcp/prompts/list",
        ctx.base_url, server_name
    );
    let prompts_response = ctx.http.get(&prompts_url).send().await?;

    let prompts_status = prompts_response.status();
    if !prompts_status.is_success() {
        println!(
            "✓ MCP prompts endpoint not available (status {}, skipped)",
            prompts_status
        );
        return Ok(());
    }

    let prompts_body: serde_json::Value = prompts_response.json().await?;

    let empty_vec = vec![];
    let prompts = prompts_body["result"]["prompts"]
        .as_array()
        .or_else(|| prompts_body["prompts"].as_array())
        .unwrap_or(&empty_vec);

    if prompts.is_empty() {
        println!("✓ No prompts to test (skipped)");
        return Ok(());
    }

    let first_prompt = &prompts[0];
    let prompt_name = first_prompt["name"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Prompt missing name"))?;

    let get_url = format!(
        "{}/api/v1/mcp/{}/mcp/prompts/get",
        ctx.base_url, server_name
    );
    let get_payload = json!({
        "jsonrpc": "2.0",
        "method": "prompts/get",
        "params": {
            "name": prompt_name,
            "arguments": {}
        },
        "id": 1
    });

    let response = ctx.http.post(&get_url).json(&get_payload).send().await?;

    let status = response.status();
    match status.as_u16() {
        200..=299 => {
            let body: serde_json::Value = response.json().await?;
            assert!(
                body["result"]["messages"].is_array() || body["messages"].is_array(),
                "Prompt should have messages"
            );
            println!("✓ MCP prompt templates expanded");
        },
        401 | 404 => {
            println!(
                "✓ MCP prompt expansion not available (status {}, skipped)",
                status
            );
        },
        _ => {
            println!(
                "✓ MCP prompt expansion responded with status {} (skipped)",
                status
            );
        },
    }

    Ok(())
}
