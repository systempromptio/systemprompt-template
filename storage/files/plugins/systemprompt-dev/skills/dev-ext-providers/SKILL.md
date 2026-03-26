---
name: "Extension: AI & Tool Providers"
description: "LlmProvider, ToolProvider trait implementations and MCP server integration"
---

# Extension: AI & Tool Providers

Provider traits enable custom LLM integrations, tool implementations, and MCP server management. Traits live in `crates/shared/provider-contracts/src/`.

---

## 1. LlmProvider

Custom LLM provider integration for AI generation.

### Trait

```rust
#[async_trait]
pub trait LlmProvider: Send + Sync {
    fn provider_id(&self) -> &'static str;
    async fn generate(&self, request: ChatRequest) -> Result<ChatResponse>;
    async fn stream(
        &self,
        request: ChatRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamChunk>> + Send>>>;
}
```

### Registration

```rust
impl Extension for MyExtension {
    fn llm_providers(&self) -> Vec<Arc<dyn LlmProvider>> {
        vec![Arc::new(CustomLlmProvider::new())]
    }
}
```

### AI Configuration Hierarchy

Priority: Tool > Agent > Global.

| Level | Source | Scope |
|-------|--------|-------|
| Global | `services/ai/config.yaml` | All requests |
| Agent | `services/agents/*.yaml` metadata | Per-agent |
| Tool | `tool_model_overrides` | Per-tool-call |

---

## 2. ToolProvider

Custom tool implementations for agent use.

### Trait

```rust
#[async_trait]
pub trait ToolProvider: Send + Sync {
    fn provider_id(&self) -> &'static str;
    fn available_tools(&self) -> Vec<ToolDefinition>;
    async fn execute_tool(&self, request: ToolCallRequest) -> Result<ToolCallResult>;
}
```

### Registration

```rust
impl Extension for MyExtension {
    fn tool_providers(&self) -> Vec<Arc<dyn ToolProvider>> {
        vec![Arc::new(WebSearchToolProvider)]
    }
}
```

---

## 3. MCP Server Integration

Model Context Protocol servers provide tools to agents via JSON-RPC.

### MCP Server Config

```yaml
mcp_servers:
  my-server:
    type: internal
    binary: my-mcp-server
    package: my-mcp-server
    port: 5020
    endpoint: http://localhost:8080/api/v1/mcp/my-server/mcp
    enabled: true
    description: "My custom MCP server"
    oauth:
      required: true
      scopes:
        - admin
```

### MCP Extension Manifest

```yaml
type: mcp
name: my-mcp-server
binary: my-mcp-server
description: "My custom MCP server"
port: 5020
build_type: workspace
enabled: true
```

### Agent-to-MCP Wiring

```yaml
metadata:
  mcpServers:
    - my-server
```

---

## 4. Rules

| Rule | Rationale |
|------|-----------|
| Provider IDs must be unique | Registry keyed by provider_id |
| All providers are `Send + Sync` | Multi-threaded async context |
| MCP ports in 5000-5999 range | Configured in `services/config/config.yaml` |
| Agent ports in 9000-9999 range | Separate range from MCP |
| Tool definitions use JSON Schema | Standard parameter validation |
| LLM providers must support streaming | Required for real-time agent responses |
