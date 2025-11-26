# MCP Server Template

Boilerplate template for creating new MCP servers in SystemPrompt OS.

## Quick Start

1. **Copy template to new server:**
   ```bash
   cp -r crates/mcp-servers/template crates/mcp-servers/my-server
   cd crates/mcp-servers/my-server
   ```

2. **Update package name** in `Cargo.toml`:
   ```toml
   [package]
   name = "my-server"

   [[bin]]
   name = "my-server"
   path = "src/main.rs"
   ```

3. **Rename structs** throughout the codebase:
   - `TemplateServer` → `MyServer`
   - `TemplateTools` → `MyTools`
   - `TemplatePrompts` → `MyPrompts`

4. **Implement your tools** in `src/tools/mod.rs`
5. **Implement your prompts** in `src/prompts/mod.rs`
6. **Update server info** in `src/server.rs` (`get_info()` method)

## What's Included

### Core Infrastructure

- **Analytics Headers Extraction** - Automatic extraction of:
  - `x-user-id`, `x-session-id`, `x-trace-id`
  - `x-context-id`, `x-task-id`

- **Automatic Tool Execution Tracking** - Every tool call is tracked:
  - Start time, completion time, duration
  - Input parameters, output data, output schema
  - Success/failure status, error messages
  - Linked to user/session/trace for analytics

- **LogContext Building** - Automatic log context with:
  - User context when authenticated
  - System context when unauthenticated
  - Full traceability with trace_id

- **Database Connection** - Pre-configured `DbPool` access
- **Error Handling** - MCP protocol error handling
- **Validation** - Request validation logic

### Artifacts System

The template includes a complete artifacts system for structured data visualization.

#### Table Artifacts

Return tabular data with sorting, filtering, and pagination:

```rust
use crate::artifacts::{create_table_artifact_schema, create_table_response, TableHints};
use serde_json::{json, Map};
use std::collections::HashMap;

// Define schema
fn my_tool_schema() -> JsonValue {
    let mut properties = Map::new();
    properties.insert("id".to_string(), json!({"type": "string"}));
    properties.insert("name".to_string(), json!({"type": "string"}));
    properties.insert("count".to_string(), json!({"type": "integer"}));
    properties.insert("amount".to_string(), json!({"type": "number"}));

    let hints = TableHints::new(vec![
        "id".to_string(),
        "name".to_string(),
        "count".to_string(),
        "amount".to_string(),
    ])
    .with_sortable_columns(vec!["count".to_string(), "amount".to_string()])
    .with_default_sort("count".to_string(), "desc".to_string())
    .with_filterable(true)
    .with_page_size(10)
    .with_column_types({
        let mut types = HashMap::new();
        types.insert("id".to_string(), "string".to_string());
        types.insert("name".to_string(), "string".to_string());
        types.insert("count".to_string(), "integer".to_string());
        types.insert("amount".to_string(), "currency".to_string());
        types
    });

    create_table_artifact_schema(
        "My tool results as a table",
        properties,
        hints
    )
}

// Return data
async fn handle_my_tool(...) -> Result<CallToolResult, McpError> {
    let items = vec![
        json!({"id": "1", "name": "Item A", "count": 10, "amount": 99.50}),
        json!({"id": "2", "name": "Item B", "count": 5, "amount": 49.99}),
    ];

    let response = create_table_response(items, Some(execution_id));

    Ok(CallToolResult {
        content: vec![],
        is_error: None,
        structured_content: Some(response),
    })
}
```

**Table Column Types:**
- `string` - Text data
- `integer` - Whole numbers
- `number` - Floating point
- `currency` - Formatted as money ($X.XX)
- `percentage` - Formatted as percent (X%)
- `date` - Date/time formatting

#### Chart Artifacts

Return time-series or categorical data for visualization:

```rust
use crate::artifacts::{
    create_chart_artifact_schema, create_chart_response,
    ChartHints, ChartSeries, ChartDataset
};

// Define schema
fn my_chart_schema() -> JsonValue {
    let hints = ChartHints::new("line".to_string(), "Sales Over Time".to_string())
        .with_x_axis("Date".to_string(), "category".to_string())
        .with_y_axis("Amount".to_string(), "linear".to_string())
        .add_series("Revenue".to_string(), "#4CAF50".to_string())
        .add_series("Costs".to_string(), "#F44336".to_string());

    create_chart_artifact_schema("Sales chart", hints)
}

// Return data
async fn handle_my_chart(...) -> Result<CallToolResult, McpError> {
    let labels = vec!["Jan".to_string(), "Feb".to_string(), "Mar".to_string()];
    let datasets = vec![
        ChartDataset::new("Revenue".to_string(), vec![1000.0, 1200.0, 1500.0]),
        ChartDataset::new("Costs".to_string(), vec![800.0, 900.0, 950.0]),
    ];

    let response = create_chart_response(labels, datasets, Some(execution_id));

    Ok(CallToolResult {
        content: vec![],
        is_error: None,
        structured_content: Some(response),
    })
}
```

**Chart Types:**
- `line` - Line chart
- `bar` - Bar chart
- `pie` - Pie chart
- `doughnut` - Doughnut chart
- `area` - Area chart

**Axis Types:**
- `category` - Categorical data (labels)
- `linear` - Numeric linear scale
- `logarithmic` - Logarithmic scale
- `time` - Time-based scale

## Project Structure

```
my-server/
├── Cargo.toml              # Package configuration
├── src/
│   ├── main.rs             # Entry point (port/name configuration)
│   ├── lib.rs              # Module exports
│   ├── server.rs           # Server implementation
│   ├── artifacts.rs        # Artifact system (tables, charts)
│   ├── tools/
│   │   └── mod.rs         # Tool definitions and handlers
│   └── prompts/
│       └── mod.rs         # Prompt definitions and handlers
```

## Adding Tools

Edit `src/tools/mod.rs`:

```rust
impl MyTools {
    pub async fn list_tools(&self) -> Result<ListToolsResult, McpError> {
        let tools = vec![
            Tool {
                name: "my_tool".into(),
                title: Some("My Tool".into()),
                description: Some("Tool description".into()),
                input_schema: Arc::new(my_tool_input_schema()),
                output_schema: Some(Arc::new(my_tool_output_schema())),
                annotations: None,
                icons: None,
            },
        ];

        Ok(ListToolsResult {
            tools,
            next_cursor: None,
        })
    }

    pub async fn call_tool(
        &self,
        name: &str,
        request: CallToolRequestParam,
        ctx: RequestContext<RoleServer>,
        logger: LogService,
    ) -> Result<CallToolResult, McpError> {
        match name {
            "my_tool" => handle_my_tool(&self._db_pool, request, ctx, logger).await,
            _ => Err(McpError::method_not_found::<CallToolRequestMethod>()),
        }
    }
}
```

## Adding Prompts

Edit `src/prompts/mod.rs`:

```rust
impl MyPrompts {
    pub async fn list_prompts(
        &self,
        _request: Option<PaginatedRequestParam>,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<ListPromptsResult, McpError> {
        Ok(ListPromptsResult {
            prompts: vec![
                Prompt {
                    name: "my_prompt".into(),
                    description: Some("Prompt description".into()),
                    arguments: Some(vec![
                        PromptArgument {
                            name: "param1".into(),
                            description: Some("Parameter description".into()),
                            required: Some(false),
                            title: None,
                        },
                    ]),
                    title: None,
                    icons: None,
                },
            ],
            next_cursor: None,
        })
    }

    pub async fn get_prompt(
        &self,
        request: GetPromptRequestParam,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<GetPromptResult, McpError> {
        match request.name.as_ref() {
            "my_prompt" => {
                let param1 = request
                    .arguments
                    .as_ref()
                    .and_then(|args| args.get("param1"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("default");

                let content = format!("Prompt content with {}", param1);

                Ok(GetPromptResult {
                    description: Some("Prompt description".to_string()),
                    messages: vec![PromptMessage {
                        role: PromptMessageRole::User,
                        content: PromptMessageContent::text(content),
                    }],
                })
            }
            _ => Err(McpError::invalid_params(
                format!("Unknown prompt: {}", request.name),
                None,
            )),
        }
    }
}
```

## Configuration

Set environment variables when running:

```bash
MCP_PORT=5002 MCP_NAME=my-server cargo run --bin my-server
```

Or configure in your MCP server registry.

## Best Practices

1. **Use Artifacts for Structured Data** - Return tables/charts instead of plain text
2. **Leverage Execution Tracking** - All tool calls are automatically tracked
3. **Use LogService with Context** - All logs include user/session/trace context
4. **Validate Input Parameters** - Use JSON schema validation
5. **Return Proper Error Codes** - Use MCP error codes (invalid_params, method_not_found, etc.)
6. **Document Your Tools** - Add clear descriptions and examples

## Testing

```bash
# Build
cargo build --bin my-server

# Run
MCP_PORT=5002 MCP_NAME=my-server cargo run --bin my-server

# Test tool call
curl -X POST http://localhost:5002/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "tools/call",
    "params": {
      "name": "my_tool",
      "arguments": {}
    }
  }'
```

## Examples

See `crates/mcp-servers/systemprompt-admin` for a complete implementation example with:
- Multiple tools (analytics, agent management)
- Table artifacts with complex schemas
- Chart artifacts for trends
- Prompt management
- Comprehensive error handling
