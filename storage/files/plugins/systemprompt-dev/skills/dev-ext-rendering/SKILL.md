---
name: "Extension: Rendering"
description: "ComponentRenderer, TemplateProvider, TemplateDataExtender, and PagePrerenderer trait implementations"
---

# Extension: Rendering

Rendering traits control how pages are composed, how templates are provided, how template context is extended, and how static pages are pre-generated. All traits live in `crates/shared/provider-contracts/src/`.

---

## 1. ComponentRenderer

Renders reusable UI components (header, footer, sidebar, partials) into page templates.

### Trait

```rust
#[async_trait]
pub trait ComponentRenderer: Send + Sync {
    fn component_id(&self) -> &str;
    fn applies_to(&self) -> Vec<String>;
    fn partial_template(&self) -> Option<PartialTemplate>;
    async fn render(&self, ctx: &dyn ComponentRenderContext) -> Result<RenderedComponent>;
}
```

### Existing Components

| Component | Renderer | Pages |
|-----------|----------|-------|
| Head assets | `HeadAssetsPartialRenderer` | All |
| Header | `HeaderPartialRenderer` | All |
| Footer | `FooterPartialRenderer` | All |
| Scripts | `ScriptsPartialRenderer` | All |
| CLI animation | `CliRemoteAnimationPartialRenderer` | Homepage |

### Registration

```rust
impl Extension for MyExtension {
    fn component_renderers(&self) -> Vec<Arc<dyn ComponentRenderer>> {
        vec![Arc::new(MyComponent)]
    }
}
```

---

## 2. TemplateProvider

Supplies page templates (embedded or file-based) for rendering.

### Trait

```rust
pub trait TemplateProvider: Send + Sync {
    fn templates(&self) -> Vec<TemplateDefinition>;
    fn provider_id(&self) -> &'static str;
    fn priority(&self) -> u32;
}
```

### Priority System

| Priority | Source |
|----------|-------|
| 100 | Extension templates (override core) |
| 1000 | Core default templates (fallback) |

Lower priority wins. Extension templates override core.

---

## 3. TemplateDataExtender

Injects variables into template rendering context. Runs before template compilation.

### Trait

```rust
#[async_trait]
pub trait TemplateDataExtender: Send + Sync {
    fn applies_to(&self) -> Vec<String>;
    async fn extend(&self, ctx: &dyn ExtenderContext, data: &mut Value) -> Result<()>;
    fn priority(&self) -> u32;
}
```

### Standard Template Variables

| Variable | Type | Source |
|----------|------|--------|
| `site` | Object | Full site configuration |
| `title` | String | Page title |
| `CONTENT_HTML` | String | Rendered markdown |
| `ORG_NAME` | String | Organization name |
| `JS_BASE_PATH` | String | JavaScript directory |
| `CSS_BASE_PATH` | String | CSS directory |
| `FOOTER_NAV` | String | Footer navigation HTML |

---

## 4. PagePrerenderer

Generates static HTML pages at build time. Runs during `content_prerender` job.

### Trait

```rust
#[async_trait]
pub trait PagePrerenderer: Send + Sync {
    fn page_type(&self) -> &str;
    async fn prepare(&self, ctx: &dyn PrerendererContext) -> Result<Option<PageRenderSpec>>;
    fn priority(&self) -> u32;
}
```

### Existing Prerenderers

| Prerenderer | Output |
|-------------|--------|
| `HomepagePrerenderer` | `/index.html` |
| `FeaturePagePrerenderer` | `/features/*.html` |

---

## 5. Rules

| Rule | Rationale |
|------|-----------|
| Templates use Handlebars syntax | `{{variable}}`, `{{#if}}`, `{{> partial}}` |
| Partials embedded via `include_str!()` | Compile-time inclusion |
| Extension priority starts at 100 | Core uses 1-50 |
| Escape all user content | `{{variable}}` auto-escapes. `{{{raw}}}` only for trusted HTML. |
| CSS/JS paths via template variables | Never hardcode paths |
| All renderers are `Send + Sync` | Multi-threaded async context |
