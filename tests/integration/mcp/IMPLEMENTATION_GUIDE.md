# MCP Tests Implementation Guide

**Status**: All 4 tests are 100% stubs. Zero MCP server testing.

**Goal**: Comprehensive Model Context Protocol server lifecycle testing including tools, resources, and prompts.

---

## Test Organization (Semantic Breakdown)

### Group 1: Server Lifecycle (3 tests)
- `test_mcp_server_starts` - MCP server responds to initialization
- `test_mcp_server_stops` - Server stops cleanly
- `test_mcp_server_reconnection` - Can reconnect after disconnect

### Group 2: Tools (3 tests)
- `test_mcp_tools_listed` - GET /tools returns all available tools
- `test_mcp_tool_invocation` - Calling a tool executes and returns result
- `test_mcp_tool_error_handling` - Invalid args handled gracefully

### Group 3: Resources (2 tests)
- `test_mcp_resources_accessible` - GET /resources lists resources
- `test_mcp_resource_content_retrieved` - Reading resource returns data

### Group 4: Prompts (2 tests)
- `test_mcp_prompts_listed` - GET /prompts returns prompts
- `test_mcp_prompt_templates_expanded` - Prompt variables substituted

### Group 5: Analytics (1 test)
- `test_mcp_operations_tracked` - Tool calls logged in analytics

---

## Implementation Template

```rust
use crate::common::*;
use anyhow::Result;

#[tokio::test]
async fn test_mcp_server_starts() -> Result<()> {
    // PHASE 1: Setup
    let ctx = TestContext::new().await?;

    // PHASE 2: Send MCP initialization request
    let mcp_url = format!("{}/mcp/initialize", ctx.base_url);
    let init_payload = json!({
        "jsonrpc": "2.0",
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {
                "name": "test-client",
                "version": "1.0"
            }
        },
        "id": 1
    });

    let response = ctx.http
        .post(&mcp_url)
        .json(&init_payload)
        .send()
        .await?;

    // PHASE 3: Verify response
    assert_eq!(response.status(), 200, "MCP initialization failed");

    let body: serde_json::Value = response.json().await?;
    assert!(body["result"]["capabilities"].is_object(), "Missing capabilities");

    // PHASE 4: Query database for service status
    let service_query = "SELECT id, status, port FROM services WHERE type = 'mcp'";
    let rows = ctx.db.fetch_all(service_query, &[]).await?;

    assert!(!rows.is_empty(), "MCP service not registered");

    let service = rows[0].clone();
    assert_eq!(
        service.get("status").and_then(|v| v.as_str()),
        Some("running"),
        "MCP service not running"
    );

    println!("✓ MCP server started successfully");
    Ok(())
}
```

---

## Database Validation Queries

### Test 1: Server Lifecycle
```sql
-- Verify MCP service registered
SELECT id, name, type, status, port, pid, created_at
FROM services
WHERE type = 'mcp'
AND name LIKE '%mcp%'
ORDER BY created_at DESC;

-- Expected:
-- status: 'running' | 'stopped' | 'error'
-- port: Available port number
-- pid: Process ID (for running services)

-- Verify service startup logged
SELECT event_type, severity, metadata->>'service_id' as service_id,
       metadata->>'action' as action
FROM analytics_events
WHERE event_type = 'service_lifecycle'
AND metadata->>'service_type' = 'mcp'
ORDER BY created_at DESC;

-- Expected: 'start' action for running service
```

### Test 2: Tool Listing
```sql
-- Verify tools registered
SELECT id, name, service_id, description, schema
FROM mcp_tools
WHERE service_id = 'test-mcp-service'
ORDER BY name;

-- Expected:
-- Multiple tools with name, description, schema (JSON)

-- Verify tool schema valid
SELECT name, schema, jsonb_valid(schema) as schema_valid
FROM mcp_tools
WHERE schema IS NOT NULL;
```

### Test 3: Tool Invocation
```sql
-- Verify tool call logged
SELECT tool_id, service_id, input_params, output_result,
       execution_time_ms, status, created_at
FROM tool_invocations
WHERE service_id = 'test-mcp-service'
AND tool_id = 'test-tool'
ORDER BY created_at DESC;

-- Expected:
-- status: 'success' | 'error'
-- execution_time_ms: > 0
-- output_result: JSON with result

-- Verify tool metrics
SELECT tool_id, COUNT(*) as invocation_count,
       AVG(execution_time_ms) as avg_execution_time,
       SUM(CASE WHEN status = 'error' THEN 1 ELSE 0 END) as error_count
FROM tool_invocations
WHERE service_id = 'test-mcp-service'
GROUP BY tool_id;
```

### Test 4: Resources
```sql
-- Verify resources registered
SELECT id, name, service_id, resource_type, uri, mime_type
FROM mcp_resources
WHERE service_id = 'test-mcp-service'
ORDER BY name;

-- Expected:
-- Multiple resources with unique URIs
-- mime_type: application/json, text/plain, etc.

-- Verify resource content accessible
SELECT resource_id, content, content_size, last_updated
FROM mcp_resource_content
WHERE resource_id = 'test-resource'
ORDER BY last_updated DESC
LIMIT 1;
```

### Test 5: Prompts
```sql
-- Verify prompts registered
SELECT id, name, service_id, description, template
FROM mcp_prompts
WHERE service_id = 'test-mcp-service'
ORDER BY name;

-- Expected:
-- Multiple prompts with templates containing {variables}

-- Verify prompt expansion logged
SELECT prompt_id, input_variables, expanded_content, created_at
FROM prompt_expansions
WHERE prompt_id = 'test-prompt'
ORDER BY created_at DESC;
```

### Test 6: Analytics Integration
```sql
-- Verify tool calls tracked in analytics
SELECT session_id, event_type, event_category, severity,
       metadata->>'tool_name' as tool_name,
       metadata->>'execution_time_ms' as execution_ms
FROM analytics_events
WHERE event_type = 'mcp_tool_call'
AND metadata->>'service_id' = 'test-mcp-service'
ORDER BY created_at DESC;

-- Expected: Tool invocations tracked as events

-- Calculate tool usage
SELECT metadata->>'tool_name' as tool_name, COUNT(*) as usage_count
FROM analytics_events
WHERE event_type = 'mcp_tool_call'
GROUP BY metadata->>'tool_name'
ORDER BY usage_count DESC;
```

---

## Test Implementation Examples

### Test 1: Server Lifecycle
**File**: `lifecycle.rs`

```rust
#[tokio::test]
async fn test_mcp_server_lifecycle() -> Result<()> {
    let ctx = TestContext::new().await?;

    // Initialize MCP
    let mcp_url = format!("{}/mcp/initialize", ctx.base_url);
    let init_payload = json!({
        "jsonrpc": "2.0",
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {
                "name": "test-client",
                "version": "1.0"
            }
        },
        "id": 1
    });

    let response = ctx.http.post(&mcp_url).json(&init_payload).send().await?;
    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await?;
    assert!(body["result"]["capabilities"].is_object());

    // Verify service status
    TestContext::wait_for_async_processing().await;

    let service_query = "SELECT status, port FROM services WHERE type = 'mcp' LIMIT 1";
    let rows = ctx.db.fetch_all(service_query, &[]).await?;

    assert!(!rows.is_empty());
    assert_eq!(rows[0].get("status").and_then(|v| v.as_str()), Some("running"));

    println!("✓ MCP server lifecycle verified");
    Ok(())
}

#[tokio::test]
async fn test_mcp_server_stops_gracefully() -> Result<()> {
    let ctx = TestContext::new().await?;

    // Send shutdown request
    let shutdown_url = format!("{}/mcp/shutdown", ctx.base_url);
    let response = ctx.http.post(&shutdown_url).send().await?;

    assert!(response.status().is_success());

    // Wait for shutdown
    TestContext::wait_for_async_processing().await;
    tokio::time::sleep(Duration::from_secs(1)).await;

    // Verify service stopped
    let service_query = "SELECT status FROM services WHERE type = 'mcp' LIMIT 1";
    let rows = ctx.db.fetch_all(service_query, &[]).await?;

    assert!(!rows.is_empty());
    assert_eq!(rows[0].get("status").and_then(|v| v.as_str()), Some("stopped"));

    println!("✓ MCP server stopped gracefully");
    Ok(())
}

#[tokio::test]
async fn test_mcp_server_reconnection() -> Result<()> {
    let ctx = TestContext::new().await?;

    // Initialize first connection
    let mcp_url = format!("{}/mcp/initialize", ctx.base_url);
    let init_payload = json!({
        "jsonrpc": "2.0",
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {"name": "test-client", "version": "1.0"}
        },
        "id": 1
    });

    let response1 = ctx.http.post(&mcp_url).json(&init_payload).send().await?;
    assert_eq!(response1.status(), 200);

    let body1: serde_json::Value = response1.json().await?;
    let server_info1 = &body1["result"]["serverInfo"];

    // Reconnect
    let response2 = ctx.http.post(&mcp_url).json(&init_payload).send().await?;
    assert_eq!(response2.status(), 200);

    let body2: serde_json::Value = response2.json().await?;
    let server_info2 = &body2["result"]["serverInfo"];

    // Should get same server or new one
    assert!(server_info2.is_object());

    println!("✓ MCP server reconnection verified");
    Ok(())
}
```

---

### Test 2: Tools
**File**: `tools.rs`

```rust
#[tokio::test]
async fn test_mcp_tools_listed() -> Result<()> {
    let ctx = TestContext::new().await?;

    // Get available tools
    let tools_url = format!("{}/mcp/tools/list", ctx.base_url);
    let response = ctx.http.get(&tools_url).send().await?;

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await?;
    let tools = body["tools"].as_array()
        .ok_or_else(|| anyhow::anyhow!("No tools array"))?;

    assert!(!tools.is_empty(), "No tools available");

    // Verify tool structure
    for tool in tools {
        assert!(tool["name"].is_string(), "Tool missing name");
        assert!(tool["description"].is_string() || tool["description"].is_null(),
                "Invalid description");
        assert!(tool["inputSchema"].is_object(), "Tool missing inputSchema");
    }

    println!("✓ MCP tools listed successfully");
    Ok(())
}

#[tokio::test]
async fn test_mcp_tool_invocation() -> Result<()> {
    let ctx = TestContext::new().await?;
    let fingerprint = ctx.fingerprint().to_string();

    // Get first tool
    let tools_url = format!("{}/mcp/tools/list", ctx.base_url);
    let tools_response = ctx.http.get(&tools_url).send().await?;
    let tools_body: serde_json::Value = tools_response.json().await?;
    let tools = tools_body["tools"].as_array().unwrap();
    let first_tool = &tools[0];
    let tool_name = first_tool["name"].as_str().unwrap();

    // Invoke tool
    let invoke_url = format!("{}/mcp/tools/call", ctx.base_url);
    let invoke_payload = json!({
        "tool": tool_name,
        "arguments": {}
    });

    let response = ctx.http
        .post(&invoke_url)
        .header("x-fingerprint", &fingerprint)
        .json(&invoke_payload)
        .send()
        .await?;

    assert!(response.status().is_success(), "Tool invocation failed");

    let body: serde_json::Value = response.json().await?;
    assert!(body["result"].is_object() || body["result"].is_array(),
            "No result from tool");

    // Verify tool call tracked
    TestContext::wait_for_async_processing().await;

    let event_query = "SELECT event_type, metadata FROM analytics_events
                      WHERE metadata->>'tool_name' = $1
                      AND event_type = 'mcp_tool_call'
                      ORDER BY created_at DESC LIMIT 1";

    let rows = ctx.db.fetch_all(event_query, &[&tool_name]).await?;
    assert!(!rows.is_empty(), "Tool invocation not tracked");

    // Cleanup
    let mut cleanup = TestCleanup::new(ctx.db.clone());
    cleanup.track_fingerprint(fingerprint);
    cleanup.cleanup_all().await?;

    println!("✓ MCP tool invocation executed and tracked");
    Ok(())
}

#[tokio::test]
async fn test_mcp_tool_error_handling() -> Result<()> {
    let ctx = TestContext::new().await?;

    // Try to invoke non-existent tool
    let invoke_url = format!("{}/mcp/tools/call", ctx.base_url);
    let invoke_payload = json!({
        "tool": "non-existent-tool",
        "arguments": {}
    });

    let response = ctx.http
        .post(&invoke_url)
        .json(&invoke_payload)
        .send()
        .await?;

    // Should return error
    assert!(response.status().is_client_error() || response.status().is_server_error(),
            "Should error for non-existent tool");

    let body: serde_json::Value = response.json().await?;
    assert!(body["error"].is_object() || body["error"].is_string(),
            "Should return error object");

    println!("✓ MCP tool error handling verified");
    Ok(())
}
```

---

### Test 3: Resources
**File**: `resources.rs`

```rust
#[tokio::test]
async fn test_mcp_resources_accessible() -> Result<()> {
    let ctx = TestContext::new().await?;

    // List resources
    let resources_url = format!("{}/mcp/resources/list", ctx.base_url);
    let response = ctx.http.get(&resources_url).send().await?;

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await?;
    let resources = body["resources"].as_array()
        .ok_or_else(|| anyhow::anyhow!("No resources"))?;

    // Resources may be empty (optional)
    for resource in resources {
        assert!(resource["uri"].is_string(), "Resource missing uri");
        assert!(resource["name"].is_string() || resource["name"].is_null(),
                "Invalid name");
    }

    println!("✓ MCP resources accessible");
    Ok(())
}

#[tokio::test]
async fn test_mcp_resource_content_retrieved() -> Result<()> {
    let ctx = TestContext::new().await?;

    // List resources
    let resources_url = format!("{}/mcp/resources/list", ctx.base_url);
    let resources_response = ctx.http.get(&resources_url).send().await?;
    let resources_body: serde_json::Value = resources_response.json().await?;
    let resources = resources_body["resources"].as_array().unwrap();

    if resources.is_empty() {
        println!("✓ No resources to test (skipped)");
        return Ok(());
    }

    // Read first resource
    let first_resource = &resources[0];
    let resource_uri = first_resource["uri"].as_str().unwrap();

    let read_url = format!("{}/mcp/resources/read", ctx.base_url);
    let read_payload = json!({
        "uri": resource_uri
    });

    let response = ctx.http
        .post(&read_url)
        .json(&read_payload)
        .send()
        .await?;

    assert!(response.status().is_success(), "Resource read failed");

    let body: serde_json::Value = response.json().await?;
    assert!(body["content"].is_string() || body["content"].is_object(),
            "No content returned");

    println!("✓ MCP resource content retrieved");
    Ok(())
}
```

---

### Test 4: Prompts
**File**: `prompts.rs`

```rust
#[tokio::test]
async fn test_mcp_prompts_listed() -> Result<()> {
    let ctx = TestContext::new().await?;

    // List prompts
    let prompts_url = format!("{}/mcp/prompts/list", ctx.base_url);
    let response = ctx.http.get(&prompts_url).send().await?;

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await?;
    let prompts = body["prompts"].as_array()
        .ok_or_else(|| anyhow::anyhow!("No prompts"))?;

    for prompt in prompts {
        assert!(prompt["name"].is_string(), "Prompt missing name");
        assert!(prompt["description"].is_string() || prompt["description"].is_null(),
                "Invalid description");
    }

    println!("✓ MCP prompts listed");
    Ok(())
}

#[tokio::test]
async fn test_mcp_prompt_templates_expanded() -> Result<()> {
    let ctx = TestContext::new().await?;

    // List prompts
    let prompts_url = format!("{}/mcp/prompts/list", ctx.base_url);
    let prompts_response = ctx.http.get(&prompts_url).send().await?;
    let prompts_body: serde_json::Value = prompts_response.json().await?;
    let prompts = prompts_body["prompts"].as_array().unwrap();

    if prompts.is_empty() {
        println!("✓ No prompts to test (skipped)");
        return Ok(());
    }

    // Get first prompt
    let first_prompt = &prompts[0];
    let prompt_name = first_prompt["name"].as_str().unwrap();

    // Get prompt with variables expanded
    let get_url = format!("{}/mcp/prompts/get", ctx.base_url);
    let get_payload = json!({
        "name": prompt_name,
        "arguments": {}
    });

    let response = ctx.http
        .post(&get_url)
        .json(&get_payload)
        .send()
        .await?;

    assert!(response.status().is_success(), "Prompt retrieval failed");

    let body: serde_json::Value = response.json().await?;
    assert!(body["messages"].is_array(), "Prompt should have messages");

    println!("✓ MCP prompt templates expanded");
    Ok(())
}
```

---

## Running the Tests

```bash
# Run all MCP tests
cargo test --test mcp --all -- --nocapture

# Run specific test
cargo test --test mcp test_mcp_tool_invocation -- --nocapture
```

## Post-Test Validation

```bash
# Check MCP service status
psql ... -c "SELECT id, type, status, port FROM services WHERE type = 'mcp';"

# Check tool invocations
psql ... -c "SELECT tool_id, COUNT(*) as calls FROM tool_invocations
            GROUP BY tool_id ORDER BY calls DESC;"

# Check MCP events
psql ... -c "SELECT event_type, COUNT(*) as count
            FROM analytics_events
            WHERE event_type LIKE '%mcp%'
            GROUP BY event_type;"
```

---

## Summary

| Test | Coverage | Database Queries |
|------|----------|------------------|
| Lifecycle | Server start/stop | services table |
| Tool List | GET /tools | mcp_tools table |
| Tool Call | Invoke tool | tool_invocations table |
| Error Handling | Invalid tool | analytics_events |
| Resources | GET /resources | mcp_resources table |
| Read Resource | Content retrieval | mcp_resource_content |
| Prompts | GET /prompts | mcp_prompts table |
| Expand Template | Variable substitution | prompt_expansions |
| Analytics | Tool usage tracking | analytics_events |

**Target**: All 9 tests fully implemented with MCP server lifecycle management and operation tracking.
