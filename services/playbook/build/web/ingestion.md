---
title: "Web Content Ingestion"
description: "How content flows from markdown frontmatter through ingestion to database storage, and how to add custom frontmatter fields."
author: "SystemPrompt"
slug: "build-web-ingestion"
keywords: "ingestion, frontmatter, ContentMetadata, ContentDataProvider, database"
kind: "playbook"
published_at: "2026-01-31"
tags:
  - build
  - content
  - ingestion
after_reading_this:
  - "Understand the complete data flow from frontmatter to database"
  - "Add custom frontmatter fields to ContentMetadata"
  - "Create database migrations for new fields"
  - "Use ContentDataProvider to enrich content at runtime"
  - "Register providers in extension.rs"
related_playbooks:
  - title: "Web Content"
    url: "/playbooks/build-web-content"
  - title: "Web Prerendering"
    url: "/playbooks/build-web-prerender"
related_code:
  - title: "Ingestion Service"
    url: "https://github.com/systempromptio/systemprompt-web/blob/main/extensions/web/src/services/ingestion.rs"
  - title: "ContentMetadata Model"
    url: "https://github.com/systempromptio/systemprompt-web/blob/main/extensions/web/src/models/content.rs#L143-L171"
  - title: "DocsContentDataProvider"
    url: "https://github.com/systempromptio/systemprompt-web/blob/main/extensions/web/src/docs/content_provider.rs"
---

# Web Content Ingestion

How content flows from markdown frontmatter through ingestion to database storage, and how to extend the system with custom fields.

---

## Data Flow Overview

```
Markdown File (frontmatter + body)
         ↓
    parse_markdown()
         ↓
    ContentMetadata (struct)
         ↓
    CreateContentParams (builder)
         ↓
    ContentRepository::create()
         ↓
    markdown_content (PostgreSQL table)
         ↓
    ContentDataProvider::enrich_content()
         ↓
    Enriched JSON (for templates)
```

---

## Ingestion Process

### Step 1: Parse Markdown

**File**: `extensions/web/src/services/ingestion.rs`

The `parse_markdown()` function extracts YAML frontmatter and body:

```rust
fn parse_markdown(content: &str) -> Result<(ContentMetadata, String), BlogError> {
    // Find frontmatter delimiters (---)
    let frontmatter = &content[4..end_idx].trim();
    let body = content[end_idx + 3..].trim().to_string();

    // Deserialize YAML to ContentMetadata
    let metadata: ContentMetadata = serde_yaml::from_str(frontmatter)?;
    Ok((metadata, body))
}
```

### Step 2: ContentMetadata Struct

**File**: `extensions/web/src/models/content.rs`

All frontmatter fields map to this struct:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentMetadata {
    // Required fields
    pub title: String,
    pub description: String,
    pub author: String,
    pub published_at: String,
    pub slug: String,
    pub keywords: String,
    pub kind: String,

    // Optional fields
    #[serde(default)]
    pub image: Option<String>,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,

    // Relation fields (stored as JSONB)
    #[serde(default)]
    pub links: Vec<ContentLinkMetadata>,
    #[serde(default)]
    pub after_reading_this: Vec<String>,
    #[serde(default)]
    pub related_playbooks: Vec<ContentLinkMetadata>,
    #[serde(default)]
    pub related_code: Vec<ContentLinkMetadata>,
    #[serde(default)]
    pub related_docs: Vec<ContentLinkMetadata>,
}
```

### Step 3: Database Storage

**File**: `extensions/web/src/repository/content.rs`

The `ContentRepository::create()` method inserts content:

```rust
sqlx::query!(
    r#"
    INSERT INTO markdown_content (
        id, slug, title, description, body, author,
        published_at, keywords, kind, image, category_id, source_id,
        version_hash, links, after_reading_this, related_playbooks,
        related_code, related_docs, updated_at
    )
    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19)
    ON CONFLICT (slug) DO UPDATE SET ...
    "#,
    // ... parameters
)
```

---

## Adding Custom Frontmatter Fields

To add a new field (e.g., `category`), follow these steps:

### 1. Add Database Column

**File**: `extensions/web/schema/011_content_category.sql` (new)

```sql
-- Add category column for content filtering
ALTER TABLE markdown_content
ADD COLUMN IF NOT EXISTS category TEXT;

CREATE INDEX IF NOT EXISTS idx_markdown_content_category_filter
ON markdown_content(category);
```

### 2. Add to ContentMetadata

**File**: `extensions/web/src/models/content.rs`

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentMetadata {
    // ... existing fields ...

    #[serde(default)]
    pub category: Option<String>,  // Add new field
}
```

### 3. Add to CreateContentParams Builder

**File**: `extensions/web/src/models/builders/content.rs`

```rust
pub struct CreateContentParams {
    // ... existing fields ...
    pub category: Option<String>,
}

impl CreateContentParams {
    // ... existing methods ...

    #[must_use]
    pub fn with_category(mut self, category: Option<String>) -> Self {
        self.category = category;
        self
    }
}
```

### 4. Update Repository INSERT

**File**: `extensions/web/src/repository/content.rs`

Add `category` to the INSERT and UPDATE queries:

```rust
INSERT INTO markdown_content (
    ..., category, ...
)
VALUES (..., $20, ...)
ON CONFLICT (slug) DO UPDATE SET
    ...,
    category = EXCLUDED.category,
    ...
```

### 5. Update Ingestion

**File**: `extensions/web/src/services/ingestion.rs`

Pass the field to the builder:

```rust
let params = CreateContentParams::new(source_id.clone(), metadata.slug.clone())
    // ... existing fields ...
    .with_category(metadata.category);
```

### 6. Rebuild and Migrate

```bash
just build
systemprompt infra db migrate
systemprompt infra jobs run blog_content_ingestion
```

---

## ContentDataProvider

ContentDataProvider enriches content after loading from the database. Use this when you need to:

- Add computed fields
- Fetch related data
- Transform data for templates

### Trait Definition

```rust
#[async_trait]
pub trait ContentDataProvider: Send + Sync {
    /// Unique identifier for this provider
    fn provider_id(&self) -> &'static str;

    /// Which content sources this provider applies to
    fn applies_to_sources(&self) -> Vec<String>;

    /// Enrich content item with additional data
    async fn enrich_content(
        &self,
        ctx: &ContentDataContext<'_>,
        item: &mut serde_json::Value,
    ) -> Result<()>;
}
```

### Example: DocsContentDataProvider

**File**: `extensions/web/src/docs/content_provider.rs`

```rust
pub struct DocsContentDataProvider;

#[async_trait]
impl ContentDataProvider for DocsContentDataProvider {
    fn provider_id(&self) -> &'static str {
        "docs-content-enricher"
    }

    fn applies_to_sources(&self) -> Vec<String> {
        vec!["documentation".to_string()]
    }

    async fn enrich_content(
        &self,
        ctx: &ContentDataContext<'_>,
        item: &mut serde_json::Value,
    ) -> Result<()> {
        let db = ctx.db_pool::<Arc<Database>>()?;
        let pool = db.pool()?;
        let content_id = ctx.content_id();

        // Fetch additional data
        let row = sqlx::query!(
            r#"
            SELECT
                slug, kind, source_id,
                COALESCE(after_reading_this, '[]'::jsonb) as "after_reading_this!",
                COALESCE(related_playbooks, '[]'::jsonb) as "related_playbooks!"
            FROM markdown_content
            WHERE id = $1
            "#,
            content_id
        )
        .fetch_one(&*pool)
        .await?;

        // Insert enriched fields
        if let Some(obj) = item.as_object_mut() {
            obj.insert("after_reading_this".to_string(), row.after_reading_this);
            obj.insert("related_playbooks".to_string(), row.related_playbooks);
        }

        // Add children for index pages
        if row.kind == "docs-index" {
            let children = self.get_children(&pool, &row.source_id, &row.slug).await;
            if let Some(obj) = item.as_object_mut() {
                obj.insert("children".to_string(), json!(children));
            }
        }

        Ok(())
    }
}
```

### Registration

**File**: `extensions/web/src/extension.rs`

```rust
impl Extension for WebExtension {
    fn content_data_providers(&self) -> Vec<Arc<dyn ContentDataProvider>> {
        vec![
            Arc::new(DocsContentDataProvider::new()),
            // Add more providers here
        ]
    }
}
```

---

## Database Schema

### markdown_content Table

| Column | Type | Description |
|--------|------|-------------|
| `id` | TEXT | Primary key (UUID) |
| `slug` | TEXT | URL-friendly identifier |
| `title` | TEXT | Content title |
| `description` | TEXT | SEO description |
| `body` | TEXT | Markdown content |
| `author` | TEXT | Author name |
| `published_at` | TIMESTAMPTZ | Publication date |
| `keywords` | TEXT | SEO keywords |
| `kind` | TEXT | Content type (article, guide, etc.) |
| `image` | TEXT | Featured image URL |
| `category_id` | TEXT | Source category |
| `source_id` | TEXT | Content source (blog, documentation) |
| `version_hash` | TEXT | Content hash for change detection |
| `public` | BOOLEAN | Published status |
| `links` | JSONB | External reference links |
| `after_reading_this` | JSONB | Learning objectives |
| `related_playbooks` | JSONB | Related playbook links |
| `related_code` | JSONB | Related code links |
| `related_docs` | JSONB | Related documentation links |
| `updated_at` | TIMESTAMPTZ | Last modification time |

---

## CLI Commands

```bash
# Run ingestion job
systemprompt infra jobs run blog_content_ingestion

# List content
systemprompt core content list --source blog

# Show content details
systemprompt core content show <slug> --source <source>

# Query database directly
systemprompt infra db query "SELECT slug, kind, category FROM markdown_content WHERE source_id = 'blog'"
```

---

## Troubleshooting

| Problem | Solution |
|---------|----------|
| Field not stored | Check ContentMetadata struct has the field |
| Column doesn't exist | Run database migration |
| Provider not called | Check `applies_to_sources()` matches content source |
| Field not in template | Check ContentDataProvider enriches the field |

---

## Quick Reference

| Task | Location |
|------|----------|
| Add frontmatter field | `extensions/web/src/models/content.rs` |
| Add database column | `extensions/web/schema/*.sql` |
| Store field in DB | `extensions/web/src/repository/content.rs` |
| Enrich at runtime | Create `ContentDataProvider` |
| Register provider | `extensions/web/src/extension.rs` |
