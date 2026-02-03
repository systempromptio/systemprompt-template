# SystemPrompt Blog Extension

A full-featured blog and content management extension for SystemPrompt. This serves as the reference implementation demonstrating how to build extensions with database schemas, API routes, background jobs, and type-safe dependencies.

## Features

- **Content Management**: Store and retrieve markdown-based blog posts, articles, papers, and tutorials
- **Full-Text Search**: PostgreSQL-powered search with filters and pagination
- **Link Tracking**: Generate trackable campaign links with UTM parameters
- **Analytics**: Track clicks, measure campaign performance, and analyze content journeys
- **Content Ingestion**: Automatically ingest markdown files from the filesystem
- **Background Jobs**: Scheduled content ingestion job

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
systemprompt-blog-extension = { path = "extensions/blog" }
```

## Quick Start

```rust
use systemprompt_blog_extension::{BlogExtension, BlogConfig};
use std::sync::Arc;
use sqlx::PgPool;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create database pool
    let pool = Arc::new(PgPool::connect("postgres://...").await?);

    // Configure the extension
    let config = BlogConfig::default();

    // Create the extension and get its router
    let extension = BlogExtension::default();
    let router = extension.router(pool.clone(), config);

    // Mount at /api/v1/content
    let app = axum::Router::new()
        .nest(BlogExtension::base_path(), router)
        .nest("/r", BlogExtension::redirect_router(pool));

    // Start server...
    Ok(())
}
```

## Configuration

Create `services/config/blog.yaml`:

```yaml
content_sources:
  - source_id: "blog"
    category_id: "articles"
    path: "./services/content/blog"
    allowed_content_types:
      - article
      - tutorial
      - guide
    enabled: true
    override_existing: false

base_url: "https://example.com"
enable_link_tracking: true
```

## API Endpoints

### Content

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/v1/content/query` | POST | Search content with filters |
| `/api/v1/content/:source_id` | GET | List content by source |
| `/api/v1/content/:source_id/:slug` | GET | Get single content item |

### Link Tracking

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/v1/content/links/generate` | POST | Generate a tracking link |
| `/api/v1/content/links` | GET | List all links |
| `/api/v1/content/links/:id/performance` | GET | Get link performance metrics |
| `/api/v1/content/links/:id/clicks` | GET | Get click history |
| `/api/v1/content/links/campaigns/:id/performance` | GET | Campaign performance |
| `/api/v1/content/links/journey` | GET | Content journey analytics |
| `/r/:short_code` | GET | Redirect handler |

## Content Format

Blog posts use markdown with YAML frontmatter:

```markdown
---
title: "Getting Started with SystemPrompt"
description: "A comprehensive guide to building with SystemPrompt"
author: "Your Name"
published_at: "2024-01-15"
slug: "getting-started"
keywords: "systemprompt, tutorial, getting-started"
kind: "tutorial"
image: "/images/getting-started.png"
category: "guides"
tags:
  - beginner
  - tutorial
links:
  - title: "Documentation"
    url: "https://docs.example.com"
---

Your content here...
```

### Content Kinds

- `article` - General blog posts
- `paper` - Technical papers and whitepapers
- `guide` - How-to guides
- `tutorial` - Step-by-step tutorials

## Database Schema

The extension creates the following tables:

| Table | Purpose |
|-------|---------|
| `markdown_content` | Main content storage |
| `markdown_categories` | Content categories |
| `campaign_links` | Trackable links |
| `link_clicks` | Click events |
| `link_analytics_daily` | Aggregated analytics |
| `content_performance_metrics` | Performance data |
| (FTS index) | Full-text search |

## Services

### ContentService

```rust
use systemprompt_blog_extension::ContentService;

let service = ContentService::new(pool.clone());

// Get content by ID
let content = service.get_by_id("content-123").await?;

// Get content by slug
let content = service.get_by_slug("getting-started").await?;

// List content with pagination
let items = service.list(10, 0).await?;
```

### SearchService

```rust
use systemprompt_blog_extension::{SearchService, SearchRequest, SearchFilters};

let service = SearchService::new(pool.clone());

let request = SearchRequest {
    query: Some("rust programming".to_string()),
    filters: SearchFilters::default(),
    limit: 10,
    offset: 0,
};

let results = service.search(&request).await?;
```

### LinkGenerationService

```rust
use systemprompt_blog_extension::{LinkGenerationService, UtmParams};

let service = LinkGenerationService::new(pool.clone());

let utm = UtmParams {
    source: Some("newsletter".to_string()),
    medium: Some("email".to_string()),
    campaign: Some("spring-2024".to_string()),
    ..Default::default()
};

let link = service.generate(
    "https://example.com/article",
    Some("Spring Campaign"),
    Some(utm),
).await?;

println!("Short URL: /r/{}", link.short_code);
```

### ValidationService

```rust
use systemprompt_blog_extension::{ValidationService, ContentMetadata};

let validator = ValidationService::new();

// Validate metadata
let result = validator.validate_metadata(&metadata);
if !result.is_valid {
    for error in &result.errors {
        println!("{}: {}", error.field, error.message);
    }
}

// Validate content body
let result = validator.validate_body(&body);
```

### IngestionService

```rust
use systemprompt_blog_extension::IngestionService;

let service = IngestionService::new(pool.clone());

// Ingest from a configured source
let report = service.ingest_source(&source_config).await?;

println!("Processed {} of {} files", report.files_processed, report.files_found);
if !report.errors.is_empty() {
    for error in &report.errors {
        eprintln!("Error: {}", error);
    }
}
```

## Background Jobs

### ContentIngestionJob

Runs hourly to ingest content from configured directories:

```rust
use systemprompt_blog_extension::ContentIngestionJob;

// The job is automatically registered when using JobExtension
// Schedule: "0 0 * * * *" (every hour)
```

## Extension Traits

The blog extension implements:

- `ExtensionType` - Extension metadata (ID, name, version)
- `SchemaExtension` - Database migrations
- `ApiExtension` - HTTP routes
- `JobExtension` - Background jobs

## License

MIT
