---
name: "Extension: Data Providers"
description: "PageDataProvider, ContentDataProvider, and FrontmatterProcessor trait implementations"
---

# Extension: Data Providers

Data providers inject dynamic content into pages, enrich content items, and process frontmatter metadata. All traits live in `crates/shared/provider-contracts/src/`.

---

## 1. PageDataProvider

Injects JSON data into page templates during rendering.

### Trait

```rust
#[async_trait]
pub trait PageDataProvider: Send + Sync {
    fn applies_to_pages(&self) -> Vec<String>;
    async fn provide_page_data(&self, ctx: &dyn PageDataContext) -> Result<Value>;
    fn priority(&self) -> u32;
}
```

### Implementation

```rust
pub struct MyPageDataProvider;

#[async_trait]
impl PageDataProvider for MyPageDataProvider {
    fn applies_to_pages(&self) -> Vec<String> {
        vec!["homepage".into(), "blog".into()]
    }

    async fn provide_page_data(&self, ctx: &dyn PageDataContext) -> Result<Value> {
        let site = ctx.site_config();
        Ok(serde_json::json!({
            "hero_title": site.name,
            "featured_count": 5
        }))
    }

    fn priority(&self) -> u32 { 100 }
}
```

### Registration

```rust
impl Extension for MyExtension {
    fn page_data_providers(&self) -> Vec<Arc<dyn PageDataProvider>> {
        vec![Arc::new(MyPageDataProvider)]
    }
}
```

### Priority

Lower values execute first. Multiple providers for the same page merge their data. Later providers can override earlier keys.

---

## 2. ContentDataProvider

Enriches content items with computed fields during ingestion.

### Trait

```rust
#[async_trait]
pub trait ContentDataProvider: Send + Sync {
    fn applies_to_sources(&self) -> Vec<String>;
    async fn enrich_content(&self, ctx: &dyn ContentDataContext, item: &mut ContentItem) -> Result<()>;
    fn priority(&self) -> u32;
}
```

### Implementation

```rust
pub struct ReadingTimeProvider;

#[async_trait]
impl ContentDataProvider for ReadingTimeProvider {
    fn applies_to_sources(&self) -> Vec<String> {
        vec!["blog".into(), "docs".into()]
    }

    async fn enrich_content(&self, _ctx: &dyn ContentDataContext, item: &mut ContentItem) -> Result<()> {
        let word_count = item.body.split_whitespace().count();
        let reading_time = (word_count / 200).max(1);
        item.metadata.insert("reading_time_minutes".into(), reading_time.into());
        Ok(())
    }

    fn priority(&self) -> u32 { 100 }
}
```

### Registration

```rust
impl Extension for MyExtension {
    fn content_data_providers(&self) -> Vec<Arc<dyn ContentDataProvider>> {
        vec![Arc::new(ReadingTimeProvider)]
    }
}
```

---

## 3. FrontmatterProcessor

Validates and transforms YAML frontmatter before content ingestion.

### Trait

```rust
#[async_trait]
pub trait FrontmatterProcessor: Send + Sync {
    fn applies_to_sources(&self) -> Vec<String>;
    async fn process_frontmatter(&self, ctx: &dyn FrontmatterContext) -> Result<()>;
    fn priority(&self) -> u32;
}
```

### Standard Frontmatter Fields

```yaml
---
title: "Article Title"
slug: "article-title"
description: "Short description"
author: "systemprompt"
published_at: "2024-01-01"
type: "skill"
category: "skills"
keywords: "comma, separated, keywords"
kind: article
---
```

### Processing Order

1. FrontmatterProcessors run in priority order (lowest first)
2. ContentDataProviders enrich after frontmatter is finalized
3. PageDataProviders inject data at render time

---

## 4. Rules

| Rule | Rationale |
|------|-----------|
| All providers must be `Send + Sync` | Providers run in async contexts across threads |
| Return `Result`, never panic | Ingestion must continue even if one provider fails |
| Use typed identifiers | `SourceId`, `CategoryId` from `systemprompt_identifiers` |
| Log with `tracing`, never `println!` | Consistent with Rust standards |
| Priority 100 for extensions | Core providers use 1-50. Extensions start at 100. |
