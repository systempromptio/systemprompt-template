# MCP Prompts Implementation

This document provides a complete guide for implementing MCP prompts in SystemPrompt extensions.

---

## Overview

MCP prompts provide pre-built prompt templates that clients can retrieve and use. Each extension exposes prompts through:

- `list_prompts()` - Returns available prompts with metadata
- `get_prompt()` - Returns prompt content with arguments applied

---

## Directory Structure

```
src/prompts/
    ├── mod.rs                    → {Extension}Prompts struct
    ├── {prompt_name}.rs          → Content builder for complex prompts
    └── {another_prompt}.rs       → Additional prompt builders
```

---

## Prompts Struct Pattern

Create a struct to handle prompt operations:

```rust
use rmcp::{
    model::{
        GetPromptRequestParam, GetPromptResult, ListPromptsResult,
        PaginatedRequestParam, Prompt, PromptArgument, PromptMessage,
        PromptMessageContent, PromptMessageRole,
    },
    service::RequestContext,
    ErrorData as McpError, RoleServer,
};
use systemprompt::database::DbPool;

#[derive(Debug, Clone)]
pub struct InfrastructurePrompts {
    _db_pool: DbPool,
    _server_name: String,
}

impl InfrastructurePrompts {
    #[must_use]
    pub fn new(db_pool: DbPool, server_name: String) -> Self {
        Self {
            _db_pool: db_pool,
            _server_name: server_name,
        }
    }
}
```

---

## list_prompts() Implementation

Return all available prompts with their metadata:

```rust
impl InfrastructurePrompts {
    pub async fn list_prompts(
        &self,
        _request: Option<PaginatedRequestParam>,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<ListPromptsResult, McpError> {
        Ok(ListPromptsResult {
            prompts: vec![
                Prompt {
                    name: "deployment_guide".into(),
                    description: Some(
                        "Step-by-step deployment guide for SystemPrompt applications".into()
                    ),
                    arguments: Some(vec![
                        PromptArgument {
                            name: "environment".into(),
                            description: Some(
                                "Target environment (development, staging, production)".into()
                            ),
                            required: Some(false),
                            title: None,
                        },
                        PromptArgument {
                            name: "include_rollback".into(),
                            description: Some(
                                "Include rollback procedures in the guide".into()
                            ),
                            required: Some(false),
                            title: None,
                        },
                    ]),
                    title: None,
                    icons: None,
                },
                Prompt {
                    name: "sync_workflow".into(),
                    description: Some(
                        "Recommended sync workflow for keeping local and cloud in sync".into()
                    ),
                    arguments: Some(vec![
                        PromptArgument {
                            name: "direction".into(),
                            description: Some("Sync direction: push or pull".into()),
                            required: Some(false),
                            title: None,
                        },
                        PromptArgument {
                            name: "scope".into(),
                            description: Some("Sync scope: files, database, all".into()),
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
}
```

### Prompt Fields

| Field | Required | Description |
|-------|----------|-------------|
| `name` | YES | Unique identifier for the prompt |
| `description` | NO | Human-readable description |
| `arguments` | NO | List of arguments the prompt accepts |
| `title` | NO | Display title |
| `icons` | NO | Icon configuration |

### PromptArgument Fields

| Field | Required | Description |
|-------|----------|-------------|
| `name` | YES | Argument identifier |
| `description` | NO | Human-readable description |
| `required` | NO | Whether argument is required |
| `title` | NO | Display title |

---

## get_prompt() Implementation

Return prompt content with arguments applied:

```rust
impl InfrastructurePrompts {
    pub async fn get_prompt(
        &self,
        request: GetPromptRequestParam,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<GetPromptResult, McpError> {
        match request.name.as_ref() {
            "deployment_guide" => {
                let environment = request
                    .arguments
                    .as_ref()
                    .and_then(|args| args.get("environment"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("production");

                let include_rollback = request
                    .arguments
                    .as_ref()
                    .and_then(|args| args.get("include_rollback"))
                    .and_then(serde_json::Value::as_bool)
                    .unwrap_or(true);

                let prompt_content = build_deployment_guide_prompt(
                    environment,
                    include_rollback
                );

                Ok(GetPromptResult {
                    description: Some(format!(
                        "Deployment guide for {} environment",
                        environment
                    )),
                    messages: vec![PromptMessage {
                        role: PromptMessageRole::User,
                        content: PromptMessageContent::text(prompt_content),
                    }],
                })
            }
            "sync_workflow" => {
                let direction = request
                    .arguments
                    .as_ref()
                    .and_then(|args| args.get("direction"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("push");

                let scope = request
                    .arguments
                    .as_ref()
                    .and_then(|args| args.get("scope"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("all");

                let prompt_content = build_sync_workflow_prompt(direction, scope);

                Ok(GetPromptResult {
                    description: Some(format!(
                        "Sync workflow guide for {} {} operations",
                        direction, scope
                    )),
                    messages: vec![PromptMessage {
                        role: PromptMessageRole::User,
                        content: PromptMessageContent::text(prompt_content),
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

### Argument Extraction Pattern

Extract arguments with defaults:

**String extraction:**

```rust
let value = request
    .arguments
    .as_ref()
    .and_then(|args| args.get("arg_name"))
    .and_then(|v| v.as_str())
    .unwrap_or("default_value");
```

**Boolean extraction:**

```rust
let bool_value = request
    .arguments
    .as_ref()
    .and_then(|args| args.get("bool_arg"))
    .and_then(serde_json::Value::as_bool)
    .unwrap_or(false);
```

---

## Content Builders

For complex prompts, create separate builder functions:

### deployment_guide.rs

```rust
#[must_use]
pub fn build_deployment_guide_prompt(environment: &str, include_rollback: bool) -> String {
    let mut content = format!(
        r#"# Deployment Guide for {environment}

## Prerequisites

1. Ensure all tests pass
2. Verify configuration for {environment}
3. Check database migrations are up to date

## Deployment Steps

1. Build the application:
   ```bash
   cargo build --release
   ```

2. Run database migrations:
   ```bash
   just db-migrate
   ```

3. Deploy to {environment}:
   ```bash
   just deploy-{environment}
   ```

4. Verify deployment:
   ```bash
   just health-check-{environment}
   ```
"#,
        environment = environment
    );

    if include_rollback {
        content.push_str(
            r#"
## Rollback Procedures

If issues are detected:

1. Revert to previous version:
   ```bash
   just rollback-{environment}
   ```

2. Restore database backup if needed:
   ```bash
   just db-restore-{environment}
   ```
"#,
        );
    }

    content
}
```

### sync_workflow.rs

```rust
#[must_use]
pub fn build_sync_workflow_prompt(direction: &str, scope: &str) -> String {
    let direction_desc = match direction {
        "push" => "uploading local changes to cloud",
        "pull" => "downloading cloud state to local",
        _ => "syncing",
    };

    let scope_desc = match scope {
        "files" => "configuration files only",
        "database" => "database records only",
        "all" => "files, database, and deployment",
        _ => scope,
    };

    format!(
        r#"# Sync Workflow: {direction} ({scope})

## Overview

This workflow covers {direction_desc} for {scope_desc}.

## Pre-Sync Checklist

- [ ] Backup current state
- [ ] Verify no conflicting changes
- [ ] Check network connectivity

## Sync Steps

1. Preview changes (dry run):
   ```
   Use sync_{scope} tool with dry_run=true
   ```

2. Review the preview output

3. Execute sync:
   ```
   Use sync_{scope} tool with dry_run=false
   ```

4. Verify sync completed successfully

## Post-Sync Verification

- Check sync_status tool for current state
- Verify application functionality
"#,
        direction = direction,
        scope = scope,
        direction_desc = direction_desc,
        scope_desc = scope_desc
    )
}
```

---

## ServerHandler Integration

Delegate prompt methods to the prompts module:

```rust
impl ServerHandler for InfrastructureServer {
    async fn list_prompts(
        &self,
        request: Option<PaginatedRequestParam>,
        ctx: RequestContext<RoleServer>,
    ) -> Result<ListPromptsResult, McpError> {
        self.prompts.list_prompts(request, ctx).await
    }

    async fn get_prompt(
        &self,
        request: GetPromptRequestParam,
        ctx: RequestContext<RoleServer>,
    ) -> Result<GetPromptResult, McpError> {
        self.prompts.get_prompt(request, ctx).await
    }
}
```

---

## Module Exports

Export builder functions for testing and reuse:

```rust
mod deployment_guide;
mod sync_workflow;

pub use deployment_guide::build_deployment_guide_prompt;
pub use sync_workflow::build_sync_workflow_prompt;
```

---

## Best Practices

### Prompt Design

| Guideline | Description |
|-----------|-------------|
| Clear names | Use descriptive, action-oriented names |
| Good defaults | Provide sensible defaults for optional arguments |
| Markdown format | Use markdown for rich formatting |
| Actionable content | Include specific commands and steps |

### Argument Handling

| Guideline | Description |
|-----------|-------------|
| Validate early | Check argument validity before building content |
| Type-safe extraction | Use proper type conversions |
| Default values | Always provide defaults for optional arguments |
| Error messages | Return clear errors for unknown prompts |

### Content Builders

| Guideline | Description |
|-----------|-------------|
| Separate files | Put complex builders in dedicated files |
| Pure functions | Builders should be pure (no side effects) |
| `#[must_use]` | Mark builder functions with `#[must_use]` |
| Testable | Keep builders testable without MCP context |

---

## See Also

- [tools.md](./tools.md) - Tool implementation patterns
- [../architecture/overview.md](../architecture/overview.md) - Extension architecture
- [../architecture/boundaries.md](../architecture/boundaries.md) - Module boundaries
