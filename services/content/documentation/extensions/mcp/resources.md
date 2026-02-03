---
title: "MCP Resources"
description: "Implementing MCP resources and templates for exposing data and UI artifacts to clients."
author: "SystemPrompt Team"
slug: "extensions/mcp/resources"
keywords: "mcp, resources, templates, ui, artifacts, rendering"
image: "/files/images/docs/mcp-resources.svg"
kind: "reference"
public: true
tags: []
published_at: "2026-02-02"
updated_at: "2026-02-02"
---

# MCP Resources

MCP resources allow servers to expose data that clients can read. Unlike tools which perform actions, resources provide read-only access to content. This is useful for:

- Exposing stored artifacts for display
- Providing UI renderings of tool results
- Sharing configuration or reference data
- Listing available content

## Resource Types

### Static Resources

Resources with fixed URIs that always exist:

```
my-server://config
my-server://status
```

### Dynamic Resources

Resources created at runtime (e.g., artifacts):

```
my-server://artifacts/abc123
my-server://content/blog-post-slug
```

### Resource Templates

URI patterns with variables that clients can fill in:

```
ui://my-server/{artifact_id}
content://my-server/posts/{slug}
```

## Enabling Resources

Enable resources in your server capabilities:

```rust
impl ServerHandler for MyServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .enable_resources()  // Enable resources
                .build(),
            // ...
        }
    }
}
```

## Implementing Resource Methods

### list_resources

Returns currently available resources:

```rust
use rmcp::model::{
    ListResourcesResult, PaginatedRequestParams, RawResource, Resource,
};

async fn list_resources(
    &self,
    _request: Option<PaginatedRequestParams>,
    _ctx: RequestContext<RoleServer>,
) -> Result<ListResourcesResult, McpError> {
    // Example: List all published blog posts as resources
    let posts = self.load_published_posts().await?;

    let resources: Vec<Resource> = posts
        .into_iter()
        .map(|post| Resource {
            raw: RawResource {
                uri: format!("content://my-server/posts/{}", post.slug),
                name: post.title,
                description: Some(post.description),
                mime_type: Some("text/markdown".to_string()),
                size: Some(post.content.len() as u64),
                icons: None,
            },
            annotations: None,
        })
        .collect();

    Ok(ListResourcesResult {
        resources,
        next_cursor: None,
        meta: None,
    })
}
```

### list_resource_templates

Returns URI templates for dynamic resources:

```rust
use rmcp::model::{
    ListResourceTemplatesResult, RawResourceTemplate, ResourceTemplate,
};
use systemprompt::mcp::services::ui_renderer::MCP_APP_MIME_TYPE;

const SERVER_NAME: &str = "my-server";

async fn list_resource_templates(
    &self,
    _request: Option<PaginatedRequestParams>,
    _ctx: RequestContext<RoleServer>,
) -> Result<ListResourceTemplatesResult, McpError> {
    let templates = vec![
        ResourceTemplate {
            raw: RawResourceTemplate {
                uri_template: format!("ui://{SERVER_NAME}/{{artifact_id}}"),
                name: "artifact-ui".to_string(),
                title: Some("Artifact UI".to_string()),
                description: Some(
                    "Interactive UI for artifacts. Provide artifact_id.".to_string()
                ),
                mime_type: Some(MCP_APP_MIME_TYPE.to_string()),
                icons: None,
            },
            annotations: None,
        },
        ResourceTemplate {
            raw: RawResourceTemplate {
                uri_template: format!("content://{SERVER_NAME}/posts/{{slug}}"),
                name: "blog-post".to_string(),
                title: Some("Blog Post".to_string()),
                description: Some("Read a blog post by slug".to_string()),
                mime_type: Some("text/markdown".to_string()),
                icons: None,
            },
            annotations: None,
        },
    ];

    Ok(ListResourceTemplatesResult {
        resource_templates: templates,
        next_cursor: None,
        meta: None,
    })
}
```

### read_resource

Reads the content of a resource:

```rust
use rmcp::model::{
    ReadResourceRequestParams, ReadResourceResult, ResourceContents,
};

async fn read_resource(
    &self,
    request: ReadResourceRequestParams,
    _ctx: RequestContext<RoleServer>,
) -> Result<ReadResourceResult, McpError> {
    let uri = &request.uri;

    // Route to appropriate handler based on URI prefix
    if let Some(artifact_id) = parse_ui_uri(uri) {
        self.read_artifact_ui(&artifact_id).await
    } else if let Some(slug) = parse_content_uri(uri) {
        self.read_blog_post(&slug).await
    } else {
        Err(McpError::invalid_params(
            format!("Unknown resource URI: {uri}"),
            None,
        ))
    }
}
```

## URI Parsing

Create helpers to parse resource URIs:

```rust
const SERVER_NAME: &str = "my-server";

/// Parse ui://my-server/{artifact_id}
pub fn parse_ui_uri(uri: &str) -> Option<String> {
    let prefix = format!("ui://{SERVER_NAME}/");
    if uri.starts_with(&prefix) {
        Some(uri[prefix.len()..].to_string())
    } else {
        None
    }
}

/// Parse content://my-server/posts/{slug}
pub fn parse_content_uri(uri: &str) -> Option<String> {
    let prefix = format!("content://{SERVER_NAME}/posts/");
    if uri.starts_with(&prefix) {
        Some(uri[prefix.len()..].to_string())
    } else {
        None
    }
}
```

## Returning Resource Content

### Text Content

```rust
async fn read_blog_post(&self, slug: &str) -> Result<ReadResourceResult, McpError> {
    let post = self.load_post_by_slug(slug).await.map_err(|e| {
        McpError::internal_error(format!("Failed to load post: {e}"), None)
    })?;

    let contents = ResourceContents::TextResourceContents {
        uri: format!("content://{SERVER_NAME}/posts/{slug}"),
        mime_type: Some("text/markdown".to_string()),
        text: post.content,
        meta: None,
    };

    Ok(ReadResourceResult {
        contents: vec![contents],
    })
}
```

### Binary Content

```rust
async fn read_image(&self, id: &str) -> Result<ReadResourceResult, McpError> {
    let image = self.load_image(id).await?;

    let contents = ResourceContents::BlobResourceContents {
        uri: format!("images://{SERVER_NAME}/{id}"),
        mime_type: Some(image.mime_type),
        blob: base64::encode(&image.data),
        meta: None,
    };

    Ok(ReadResourceResult {
        contents: vec![contents],
    })
}
```

## UI Resources

UI resources render artifacts as HTML for display in clients that support it.

### Setup UI Registry

```rust
use systemprompt::mcp::services::ui_renderer::{
    registry::create_default_registry,
    UiRendererRegistry,
    MCP_APP_MIME_TYPE,
};

#[derive(Clone)]
pub struct MyServer {
    db_pool: DbPool,
    service_id: McpServerId,
    ui_registry: Arc<UiRendererRegistry>,
}

impl MyServer {
    pub fn new(db_pool: DbPool, service_id: McpServerId) -> Self {
        Self {
            db_pool,
            service_id,
            ui_registry: Arc::new(create_default_registry()),
        }
    }
}
```

### Read Artifact UI

```rust
async fn read_artifact_ui(&self, artifact_id: &str) -> Result<ReadResourceResult, McpError> {
    // Load artifact from database
    let artifact = self.load_artifact(artifact_id).await.map_err(|e| {
        McpError::internal_error(format!("Failed to load artifact: {e}"), None)
    })?;

    // Render to HTML using registry
    let html = self.ui_registry
        .render(&artifact)
        .map_err(|e| {
            McpError::internal_error(format!("Failed to render: {e}"), None)
        })?;

    let contents = ResourceContents::TextResourceContents {
        uri: format!("ui://{SERVER_NAME}/{artifact_id}"),
        mime_type: Some(MCP_APP_MIME_TYPE.to_string()),
        text: html,
        meta: None,
    };

    Ok(ReadResourceResult {
        contents: vec![contents],
    })
}
```

### Custom Renderers

Extend the registry with custom renderers:

```rust
use systemprompt::mcp::services::ui_renderer::{UiRenderer, UiRendererRegistry};

struct MyCustomRenderer;

impl UiRenderer for MyCustomRenderer {
    fn render(&self, artifact: &Artifact) -> Result<String> {
        // Custom HTML rendering logic
        let data = extract_data(artifact)?;
        Ok(format!(
            r#"<div class="my-custom-artifact">
                <h2>{}</h2>
                <p>{}</p>
            </div>"#,
            data.title, data.content
        ))
    }
}

// Register custom renderer
let mut registry = create_default_registry();
registry.register("my_custom_type", Box::new(MyCustomRenderer));
```

## Resource MIME Types

| MIME Type | Use For |
|-----------|---------|
| `text/plain` | Plain text |
| `text/markdown` | Markdown content |
| `text/html` | HTML content |
| `application/json` | JSON data |
| `application/x-systemprompt-ui` | UI artifacts (MCP_APP_MIME_TYPE) |
| `image/png`, `image/jpeg` | Images (as blob) |

## Pagination

For large resource lists, implement pagination:

```rust
async fn list_resources(
    &self,
    request: Option<PaginatedRequestParams>,
    _ctx: RequestContext<RoleServer>,
) -> Result<ListResourcesResult, McpError> {
    let cursor = request
        .as_ref()
        .and_then(|r| r.cursor.as_ref())
        .and_then(|c| c.parse::<usize>().ok())
        .unwrap_or(0);

    let page_size = 50;

    let all_resources = self.load_all_resources().await?;
    let page: Vec<_> = all_resources
        .into_iter()
        .skip(cursor)
        .take(page_size)
        .collect();

    let next_cursor = if page.len() == page_size {
        Some((cursor + page_size).to_string())
    } else {
        None
    };

    Ok(ListResourcesResult {
        resources: page,
        next_cursor,
        meta: None,
    })
}
```

## Complete Example

```rust
impl ServerHandler for MyServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .enable_resources()
                .build(),
            // ...
        }
    }

    async fn list_resources(
        &self,
        _request: Option<PaginatedRequestParams>,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, McpError> {
        Ok(ListResourcesResult {
            resources: vec![],  // No static resources
            next_cursor: None,
            meta: None,
        })
    }

    async fn list_resource_templates(
        &self,
        _request: Option<PaginatedRequestParams>,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<ListResourceTemplatesResult, McpError> {
        let template = ResourceTemplate {
            raw: RawResourceTemplate {
                uri_template: format!("ui://{}/{{artifact_id}}", SERVER_NAME),
                name: "artifact-ui".to_string(),
                title: Some("Artifact UI".to_string()),
                description: Some("Render artifact as HTML".to_string()),
                mime_type: Some(MCP_APP_MIME_TYPE.to_string()),
                icons: None,
            },
            annotations: None,
        };

        Ok(ListResourceTemplatesResult {
            resource_templates: vec![template],
            next_cursor: None,
            meta: None,
        })
    }

    async fn read_resource(
        &self,
        request: ReadResourceRequestParams,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResult, McpError> {
        let uri = &request.uri;

        let artifact_id = parse_ui_uri(uri).ok_or_else(|| {
            McpError::invalid_params(format!("Invalid URI: {uri}"), None)
        })?;

        let html = render_artifact(&self.db_pool, &self.ui_registry, &artifact_id)
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        Ok(ReadResourceResult {
            contents: vec![ResourceContents::TextResourceContents {
                uri: uri.clone(),
                mime_type: Some(MCP_APP_MIME_TYPE.to_string()),
                text: html,
                meta: None,
            }],
        })
    }
}
```