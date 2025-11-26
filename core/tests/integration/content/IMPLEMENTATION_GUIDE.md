# Content Tests Implementation Guide

**Status**: All 5 tests are 100% stubs. Zero content management testing.

**Goal**: Comprehensive content lifecycle testing from ingestion through rendering and analytics.

---

## Test Organization (Semantic Breakdown)

### Group 1: Blog Post CRUD (3 tests)
- `test_blog_post_creation` - POST /api/v1/blog creates post
- `test_blog_post_retrieval` - GET /api/v1/blog/{id} returns post
- `test_blog_post_update` - PUT /api/v1/blog/{id} updates content

### Group 2: Content Ingestion (2 tests)
- `test_markdown_ingestion_and_parsing` - .md files parsed to HTML
- `test_metadata_extraction_from_markdown` - frontmatter extracted

### Group 3: Content Rendering (2 tests)
- `test_blog_post_rendered_as_html` - Markdown → HTML conversion
- `test_markdown_syntax_highlighting` - Code blocks highlighted

### Group 4: Static Pages (2 tests)
- `test_static_page_serving` - /about returns HTML
- `test_404_for_missing_pages` - Nonexistent pages return 404

### Group 5: Content Analytics (2 tests)
- `test_blog_post_views_tracked` - Page views recorded
- `test_content_engagement_metrics` - Time on page, scroll depth logged

---

## Implementation Template

```rust
use crate::common::*;
use anyhow::Result;

#[tokio::test]
async fn test_blog_post_creation() -> Result<()> {
    // PHASE 1: Setup
    let ctx = TestContext::new().await?;
    let fingerprint = ctx.fingerprint().to_string();
    let admin_token = generate_admin_token(3600)?;

    // PHASE 2: Create blog post
    let blog_url = format!("{}/api/v1/blog", ctx.base_url);
    let post_data = json!({
        "title": "Test Blog Post",
        "slug": "test-blog-post-unique",
        "content": "# Test Post\n\nThis is test content.",
        "excerpt": "A test post",
        "author": "Test Author",
        "tags": ["test", "integration"]
    });

    let response = ctx.http
        .post(&blog_url)
        .header("Authorization", format!("Bearer {}", admin_token))
        .header("x-fingerprint", &fingerprint)
        .json(&post_data)
        .send()
        .await?;

    // PHASE 3: Verify HTTP response
    assert_eq!(response.status(), 201, "Post creation should return 201");

    let body: serde_json::Value = response.json().await?;
    let post_id = body["id"].as_str()
        .ok_or_else(|| anyhow::anyhow!("No post ID in response"))?
        .to_string();

    // PHASE 4: Wait for async processing
    TestContext::wait_for_async_processing().await;

    // PHASE 5: Query database
    let query = "SELECT id, title, slug, content, status
                 FROM markdown_content
                 WHERE id = $1";

    let rows = ctx.db.fetch_all(query, &[&post_id]).await?;
    assert!(!rows.is_empty(), "Post not created in database");

    // PHASE 6: Assertions
    let post = rows[0].clone();
    assert_eq!(post.get("title").and_then(|v| v.as_str()), Some("Test Blog Post"));
    assert_eq!(post.get("status").and_then(|v| v.as_str()), Some("published"));

    // PHASE 7: Cleanup
    let mut cleanup = TestCleanup::new(ctx.db.clone());
    cleanup.track_content_id(post_id);
    cleanup.track_fingerprint(fingerprint);
    cleanup.cleanup_all().await?;

    println!("✓ Blog post created and persisted");
    Ok(())
}
```

---

## Database Validation Queries

### Test 1: Blog Post Creation
```sql
-- Verify blog post created
SELECT id, title, slug, content, status, author, created_at, updated_at
FROM markdown_content
WHERE content_type = 'blog_post'
AND slug = 'test-blog-post-unique'
ORDER BY created_at DESC
LIMIT 1;

-- Expected:
-- id: UUID
-- title: "Test Blog Post"
-- slug: "test-blog-post-unique"
-- content: Markdown content
-- status: "published" or "draft"
-- author: "Test Author"
-- created_at: Recent timestamp
```

### Test 2: Blog Post Retrieval
```sql
-- Verify post can be retrieved
SELECT id, title, slug, content, rendered_html, excerpt
FROM markdown_content
WHERE content_type = 'blog_post'
AND id = 'test-post-id'
LIMIT 1;

-- Expected: All fields populated
```

### Test 3: Blog Post Update
```sql
-- Verify post was updated
SELECT id, title, content, updated_at,
       EXTRACT(EPOCH FROM (updated_at - created_at)) as age_seconds
FROM markdown_content
WHERE id = 'test-post-id'
ORDER BY updated_at DESC
LIMIT 1;

-- Expected: updated_at > created_at
```

### Test 4: Markdown Parsing
```sql
-- Verify rendered HTML exists
SELECT id, title, content, rendered_html,
       LENGTH(rendered_html) as html_length,
       CASE WHEN rendered_html LIKE '<h1>%' THEN 'has_headers'
            ELSE 'missing_headers' END as html_quality
FROM markdown_content
WHERE content_type = 'blog_post'
AND id = 'test-post-id';

-- Expected: rendered_html contains <h1>, <p>, etc.
```

### Test 5: Metadata Extraction
```sql
-- Verify frontmatter metadata extracted
SELECT id, title, metadata, tags, categories
FROM markdown_content
WHERE id = 'test-post-id'
AND metadata IS NOT NULL;

-- Expected: metadata JSON contains parsed frontmatter
-- tags: ['test', 'integration']
-- categories: Extracted categories

-- Check metadata structure
SELECT id,
       metadata->>'author' as author,
       metadata->>'date' as publish_date,
       metadata->>'image' as cover_image,
       jsonb_array_length(metadata->'tags') as tag_count
FROM markdown_content
WHERE id = 'test-post-id';
```

### Test 6: Content Rendering
```sql
-- Verify HTML rendering
SELECT id, title,
       CASE WHEN rendered_html LIKE '<html>%' THEN 'full_html'
            WHEN rendered_html LIKE '<h1>%' THEN 'partial_html'
            ELSE 'no_html' END as render_status,
       LENGTH(rendered_html) as rendered_length
FROM markdown_content
WHERE content_type = 'blog_post'
AND id = 'test-post-id';

-- Expected: rendered_html populated with actual HTML
```

### Test 7: Code Highlighting
```sql
-- Verify code blocks are highlighted
SELECT id, content, rendered_html,
       CASE WHEN rendered_html LIKE '%<pre><code%' THEN 'has_code_blocks'
            WHEN rendered_html LIKE '%<code>%' THEN 'has_inline_code'
            ELSE 'no_code' END as code_status,
       CASE WHEN rendered_html LIKE '%class="hljs%' THEN 'syntax_highlighted'
            ELSE 'not_highlighted' END as highlight_status
FROM markdown_content
WHERE content_type = 'blog_post'
AND id = 'test-post-id'
AND content LIKE '%```%';

-- Expected: Syntax highlighting classes present
```

### Test 8: Static Pages
```sql
-- Verify static pages exist
SELECT id, title, slug, content_type, status
FROM markdown_content
WHERE content_type = 'static_page'
AND slug IN ('about', 'contact', 'privacy')
ORDER BY slug;

-- Expected: Multiple static pages

-- Verify page serving
SELECT slug, rendered_html,
       CASE WHEN rendered_html IS NOT NULL THEN 'renderable'
            ELSE 'not_rendered' END as status
FROM markdown_content
WHERE content_type = 'static_page'
AND slug = 'about';
```

### Test 9: Content Analytics
```sql
-- Verify page views tracked
SELECT session_id, endpoint_path, http_method, response_status,
       response_time_ms, requested_at
FROM endpoint_requests
WHERE endpoint_path LIKE '/blog/%'
AND response_status = 200
ORDER BY requested_at DESC
LIMIT 10;

-- Expected: Multiple blog post views tracked

-- Aggregate blog traffic
SELECT endpoint_path, COUNT(*) as view_count,
       AVG(response_time_ms) as avg_response_time
FROM endpoint_requests
WHERE endpoint_path LIKE '/blog/%'
GROUP BY endpoint_path
ORDER BY view_count DESC;
```

### Test 10: Content Engagement
```sql
-- Verify engagement metrics recorded
SELECT session_id, endpoint_path, event_type, event_category,
       metadata->>'scroll_depth' as scroll_depth,
       metadata->>'time_on_page_seconds' as time_on_page
FROM analytics_events
WHERE endpoint_path LIKE '/blog/%'
AND event_type = 'page_engagement'
ORDER BY created_at DESC;

-- Expected: Engagement metrics in metadata

-- Calculate average engagement
SELECT endpoint_path,
       AVG((metadata->>'time_on_page_seconds')::int) as avg_time_on_page,
       AVG((metadata->>'scroll_depth')::float) as avg_scroll_depth
FROM analytics_events
WHERE endpoint_path LIKE '/blog/%'
GROUP BY endpoint_path;
```

---

## Test Implementation Examples

### Test 1: Blog Post Creation
**File**: `blog_posts.rs`

```rust
#[tokio::test]
async fn test_blog_post_creation() -> Result<()> {
    let ctx = TestContext::new().await?;
    let fingerprint = ctx.fingerprint().to_string();
    let admin_token = generate_admin_token(3600)?;

    // Create blog post
    let blog_url = format!("{}/api/v1/blog", ctx.base_url);
    let unique_slug = format!("test-post-{}", uuid::Uuid::new_v4());

    let post_data = json!({
        "title": "Test Blog Post",
        "slug": unique_slug,
        "content": "# Heading\n\nParagraph text here.",
        "excerpt": "Short excerpt",
        "author": "Test Author"
    });

    let response = ctx.http
        .post(&blog_url)
        .header("Authorization", format!("Bearer {}", admin_token))
        .header("x-fingerprint", &fingerprint)
        .json(&post_data)
        .send()
        .await?;

    assert_eq!(response.status(), 201);

    let body: serde_json::Value = response.json().await?;
    let post_id = body["id"].as_str().unwrap().to_string();

    // Verify in database
    TestContext::wait_for_async_processing().await;

    let query = "SELECT id, title, slug, author FROM markdown_content WHERE id = $1";
    let rows = ctx.db.fetch_all(query, &[&post_id]).await?;

    assert!(!rows.is_empty());
    assert_eq!(rows[0].get("title").and_then(|v| v.as_str()), Some("Test Blog Post"));
    assert_eq!(rows[0].get("author").and_then(|v| v.as_str()), Some("Test Author"));

    // Cleanup
    let mut cleanup = TestCleanup::new(ctx.db.clone());
    cleanup.track_content_id(post_id);
    cleanup.track_fingerprint(fingerprint);
    cleanup.cleanup_all().await?;

    println!("✓ Blog post created");
    Ok(())
}
```

---

### Test 2: Markdown Rendering
**File**: `rendering.rs`

```rust
#[tokio::test]
async fn test_blog_post_rendered_as_html() -> Result<()> {
    let ctx = TestContext::new().await?;
    let fingerprint = ctx.fingerprint().to_string();
    let admin_token = generate_admin_token(3600)?;

    // Create blog post with markdown
    let blog_url = format!("{}/api/v1/blog", ctx.base_url);
    let markdown_content = "# Test Heading\n\n## Subheading\n\nParagraph **with bold** and *italic*.";

    let post_data = json!({
        "title": "Markdown Test",
        "slug": format!("markdown-test-{}", uuid::Uuid::new_v4()),
        "content": markdown_content,
        "excerpt": "Test"
    });

    let response = ctx.http
        .post(&blog_url)
        .header("Authorization", format!("Bearer {}", admin_token))
        .json(&post_data)
        .send()
        .await?;

    let body: serde_json::Value = response.json().await?;
    let post_id = body["id"].as_str().unwrap().to_string();

    TestContext::wait_for_async_processing().await;

    // Verify HTML rendering
    let query = "SELECT rendered_html FROM markdown_content WHERE id = $1";
    let rows = ctx.db.fetch_all(query, &[&post_id]).await?;

    assert!(!rows.is_empty());
    let html = rows[0].get("rendered_html")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    assert!(html.contains("<h1>"), "Missing H1 tag");
    assert!(html.contains("<h2>"), "Missing H2 tag");
    assert!(html.contains("<p>"), "Missing P tag");
    assert!(html.contains("<strong>"), "Missing bold tag");
    assert!(html.contains("<em>"), "Missing italic tag");

    // Cleanup
    let mut cleanup = TestCleanup::new(ctx.db.clone());
    cleanup.track_content_id(post_id);
    cleanup.track_fingerprint(fingerprint);
    cleanup.cleanup_all().await?;

    println!("✓ Markdown rendered to HTML");
    Ok(())
}
```

---

### Test 3: Content Ingestion
**File**: `ingestion.rs`

```rust
#[tokio::test]
async fn test_markdown_ingestion_and_parsing() -> Result<()> {
    let ctx = TestContext::new().await?;
    let admin_token = generate_admin_token(3600)?;

    // Markdown with frontmatter
    let markdown = r#"---
title: "Test Article"
author: "John Doe"
date: "2025-01-01"
tags: ["test", "integration"]
---

# Article Title

Content goes here."#;

    // POST markdown file
    let ingest_url = format!("{}/api/v1/content/ingest", ctx.base_url);
    let response = ctx.http
        .post(&ingest_url)
        .header("Authorization", format!("Bearer {}", admin_token))
        .header("Content-Type", "text/markdown")
        .body(markdown)
        .send()
        .await?;

    assert!(response.status().is_success());

    let body: serde_json::Value = response.json().await?;
    let content_id = body["id"].as_str().unwrap().to_string();

    TestContext::wait_for_async_processing().await;

    // Verify parsing
    let query = "SELECT title, metadata, tags FROM markdown_content WHERE id = $1";
    let rows = ctx.db.fetch_all(query, &[&content_id]).await?;

    assert!(!rows.is_empty());
    assert_eq!(rows[0].get("title").and_then(|v| v.as_str()), Some("Test Article"));

    // Verify metadata
    let metadata = rows[0].get("metadata").and_then(|v| v.as_str());
    assert!(metadata.is_some());

    // Cleanup
    let mut cleanup = TestCleanup::new(ctx.db.clone());
    cleanup.track_content_id(content_id);
    cleanup.cleanup_all().await?;

    println!("✓ Markdown ingestion and parsing verified");
    Ok(())
}
```

---

### Test 4: Static Pages
**File**: `static_pages.rs`

```rust
#[tokio::test]
async fn test_static_page_serving() -> Result<()> {
    let ctx = TestContext::new().await?;
    let fingerprint = ctx.fingerprint().to_string();

    // Request static page
    let response = ctx.make_request("/about").await?;

    assert!(response.status().is_success());

    let content = response.text().await?;
    assert!(!content.is_empty());
    assert!(content.contains("<html>") || content.contains("<!DOCTYPE"));

    // Verify page view tracked
    TestContext::wait_for_async_processing().await;

    let session_query = DatabaseQueryEnum::FindSessionByFingerprintAny.get(ctx.db.as_ref());
    let sessions = ctx.db.fetch_all(&session_query, &[&fingerprint]).await?;

    assert!(!sessions.is_empty());

    let session = SessionData::from_json_row(&sessions[0])?;

    // Verify endpoint request
    let req_query = DatabaseQueryEnum::GetEndpointRequestsBySession.get(ctx.db.as_ref());
    let reqs = ctx.db.fetch_all(&req_query, &[&session.session_id]).await?;

    let about_request = reqs.iter()
        .find(|r| r.get("endpoint_path")
            .and_then(|v| v.as_str())
            .map(|s| s.contains("about"))
            .unwrap_or(false));

    assert!(about_request.is_some(), "Page view not tracked");

    // Cleanup
    let mut cleanup = TestCleanup::new(ctx.db.clone());
    cleanup.track_fingerprint(fingerprint);
    cleanup.cleanup_all().await?;

    println!("✓ Static page served and tracked");
    Ok(())
}
```

---

## Running the Tests

```bash
# Run all content tests
cargo test --test content --all -- --nocapture

# Run blog post tests
cargo test --test content test_blog_post -- --nocapture
```

## Post-Test Validation

```bash
# Check blog posts created
psql ... -c "SELECT id, title, slug, status FROM markdown_content
            WHERE content_type = 'blog_post'
            ORDER BY created_at DESC LIMIT 10;"

# Check rendering
psql ... -c "SELECT slug, LENGTH(rendered_html) as html_size
            FROM markdown_content
            WHERE rendered_html IS NOT NULL LIMIT 5;"

# Check page views
psql ... -c "SELECT endpoint_path, COUNT(*) as views
            FROM endpoint_requests
            WHERE endpoint_path LIKE '/blog/%'
            GROUP BY endpoint_path;"
```

---

## Summary

| Test | Coverage | Database Queries |
|------|----------|------------------|
| Create | POST /api/v1/blog | markdown_content table |
| Retrieve | GET /api/v1/blog/{id} | markdown_content table |
| Update | PUT /api/v1/blog/{id} | updated_at timestamp |
| Ingest | Upload markdown | markdown_content + metadata |
| Parsing | Frontmatter extraction | metadata JSON |
| Render | HTML generation | rendered_html field |
| Code Highlight | Syntax highlighting | HTML class checks |
| Static Pages | /about serving | markdown_content type=page |
| Analytics | Page views | endpoint_requests table |
| Engagement | Time on page | analytics_events metadata |

**Target**: All 10 tests fully implemented with content persistence and rendering verification.
