# Phase 4: Testing & Validation

**Objective**: Comprehensive testing of the extension system and blog migration to ensure correctness, performance, and documentation quality.

**Prerequisites**:
- Phase 1 (Extension Framework Core) must be complete
- Phase 2 (Blog Extension Extraction) must be complete
- Phase 3 (Template Integration) must be complete

---

## 1. Testing Strategy

### 1.1 Test Categories

| Category | Location | Purpose |
|----------|----------|---------|
| Unit Tests | `crates/shared/extension/src/tests/` | Type system validation |
| Integration Tests | `extensions/blog/tests/` | Extension functionality |
| E2E Tests | `tests/e2e/` | Full application flow |
| Compile-Time Tests | `tests/compile_fail/` | Dependency errors |
| Performance Tests | `tests/bench/` | Latency/throughput |

### 1.2 Coverage Goals

| Area | Target |
|------|--------|
| Extension Framework | 90%+ |
| Blog Extension | 80%+ |
| API Endpoints | 100% |
| Error Paths | 80%+ |

---

## 2. Extension Framework Tests

### 2.1 Type System Tests

**File**: `/var/www/html/systemprompt-core/crates/shared/extension/src/tests/types_test.rs`

```rust
//! Tests for ExtensionType and Dependencies traits.

use super::*;

// Test fixtures
#[derive(Default)]
struct AuthExtension;

impl ExtensionType for AuthExtension {
    const ID: &'static str = "auth";
    const NAME: &'static str = "Authentication";
    const VERSION: &'static str = "1.0.0";
}
impl NoDependencies for AuthExtension {}

#[derive(Default)]
struct BlogExtension;

impl ExtensionType for BlogExtension {
    const ID: &'static str = "blog";
    const NAME: &'static str = "Blog";
    const VERSION: &'static str = "1.0.0";
}

impl Dependencies for BlogExtension {
    type Deps = (AuthExtension, ());
}

#[derive(Default)]
struct AnalyticsExtension;

impl ExtensionType for AnalyticsExtension {
    const ID: &'static str = "analytics";
    const NAME: &'static str = "Analytics";
    const VERSION: &'static str = "1.0.0";
}

impl Dependencies for AnalyticsExtension {
    type Deps = (BlogExtension, (AuthExtension, ()));
}

#[test]
fn test_extension_type_metadata() {
    assert_eq!(AuthExtension::ID, "auth");
    assert_eq!(AuthExtension::NAME, "Authentication");
    assert_eq!(AuthExtension::VERSION, "1.0.0");
    assert_eq!(AuthExtension::PRIORITY, 100);
}

#[test]
fn test_dependency_list_empty() {
    let ids = <() as DependencyList>::dependency_ids();
    assert!(ids.is_empty());
}

#[test]
fn test_dependency_list_single() {
    type Deps = (AuthExtension, ());
    let ids = <Deps as DependencyList>::dependency_ids();
    assert_eq!(ids, vec!["auth"]);
}

#[test]
fn test_dependency_list_multiple() {
    type Deps = (BlogExtension, (AuthExtension, ()));
    let ids = <Deps as DependencyList>::dependency_ids();
    assert_eq!(ids, vec!["blog", "auth"]);
}
```

### 2.2 HList Tests

**File**: `/var/www/html/systemprompt-core/crates/shared/extension/src/tests/hlist_test.rs`

```rust
//! Tests for type-level HList operations.

use super::*;
use std::any::TypeId;

struct A;
struct B;
struct C;

#[test]
fn test_type_list_contains_empty() {
    assert!(!<() as TypeList>::contains::<A>());
}

#[test]
fn test_type_list_contains_single() {
    type List = (A, ());
    assert!(<List as TypeList>::contains::<A>());
    assert!(!<List as TypeList>::contains::<B>());
}

#[test]
fn test_type_list_contains_multiple() {
    type List = (A, (B, ()));
    assert!(<List as TypeList>::contains::<A>());
    assert!(<List as TypeList>::contains::<B>());
    assert!(!<List as TypeList>::contains::<C>());
}

// Compile-time subset check tests
// These verify the trait bounds work correctly

fn assert_subset<S: Subset<T>, T: TypeList>() {}

#[test]
fn test_empty_subset_of_anything() {
    assert_subset::<(), ()>();
    assert_subset::<(), (A, ())>();
    assert_subset::<(), (A, (B, ()))>();
}

#[test]
fn test_subset_of_self() {
    assert_subset::<(A, ()), (A, ())>();
    assert_subset::<(A, (B, ())), (A, (B, ()))>();
}

#[test]
fn test_subset_of_superset() {
    assert_subset::<(A, ()), (A, (B, ()))>();
    assert_subset::<(B, ()), (A, (B, ()))>();
}
```

### 2.3 Builder Tests

**File**: `/var/www/html/systemprompt-core/crates/shared/extension/src/tests/builder_test.rs`

```rust
//! Tests for ExtensionBuilder.

use super::*;

#[test]
fn test_builder_empty() {
    let registry = ExtensionBuilder::new().build().unwrap();
    assert!(registry.is_empty());
}

#[test]
fn test_builder_single_extension() {
    let registry = ExtensionBuilder::new()
        .extension(AuthExtension::default())
        .build()
        .unwrap();

    assert_eq!(registry.len(), 1);
    assert!(registry.has("auth"));
    assert!(registry.has_type::<AuthExtension>());
}

#[test]
fn test_builder_with_dependencies() {
    // AuthExtension must come before BlogExtension
    let registry = ExtensionBuilder::new()
        .extension(AuthExtension::default())
        .extension(BlogExtension::default())
        .build()
        .unwrap();

    assert_eq!(registry.len(), 2);
    assert!(registry.has("auth"));
    assert!(registry.has("blog"));
}

#[test]
fn test_builder_duplicate_id_rejected() {
    let result = ExtensionBuilder::new()
        .extension(AuthExtension::default())
        .extension(AuthExtension::default())
        .build();

    assert!(matches!(result, Err(ExtensionError::DuplicateExtension(_))));
}

#[test]
fn test_builder_reserved_path_rejected() {
    #[derive(Default)]
    struct BadApiExtension;

    impl ExtensionType for BadApiExtension {
        const ID: &'static str = "bad-api";
        const NAME: &'static str = "Bad API";
        const VERSION: &'static str = "1.0.0";
    }
    impl NoDependencies for BadApiExtension {}

    impl ApiExtension for BadApiExtension {
        type Db = ();
        type Config = ();

        fn router(&self, _: &(), _: &()) -> Router {
            Router::new()
        }

        fn base_path(&self) -> &'static str {
            "/api/v1/oauth"  // Reserved!
        }
    }

    let result = ExtensionBuilder::new()
        .extension(BadApiExtension::default())
        .build();

    assert!(matches!(result, Err(ExtensionError::ReservedPathCollision { .. })));
}

// Compile-time dependency check (uncomment to verify compile failure):
// #[test]
// fn test_builder_missing_dependency_compile_error() {
//     // This should fail to compile because BlogExtension needs AuthExtension
//     let _ = ExtensionBuilder::new()
//         .extension(BlogExtension::default())  // ERROR: AuthExtension not registered
//         .build();
// }
```

### 2.4 Registry Tests

**File**: `/var/www/html/systemprompt-core/crates/shared/extension/src/tests/registry_test.rs`

```rust
//! Tests for ExtensionRegistry.

use super::*;

#[test]
fn test_registry_get_by_id() {
    let registry = ExtensionBuilder::new()
        .extension(AuthExtension::default())
        .build()
        .unwrap();

    let ext = registry.get("auth");
    assert!(ext.is_some());
    assert_eq!(ext.unwrap().id(), "auth");
}

#[test]
fn test_registry_get_typed() {
    let registry = ExtensionBuilder::new()
        .extension(AuthExtension::default())
        .build()
        .unwrap();

    let ext: Option<&AuthExtension> = registry.get_typed();
    assert!(ext.is_some());
}

#[test]
fn test_registry_schema_extensions_ordered() {
    #[derive(Default)]
    struct EarlySchema;
    impl ExtensionType for EarlySchema {
        const ID: &'static str = "early";
        const NAME: &'static str = "Early";
        const VERSION: &'static str = "1.0.0";
    }
    impl NoDependencies for EarlySchema {}
    impl SchemaExtension for EarlySchema {
        fn schemas(&self) -> Vec<SchemaDefinition> { vec![] }
        fn migration_weight(&self) -> u32 { 10 }
    }

    #[derive(Default)]
    struct LateSchema;
    impl ExtensionType for LateSchema {
        const ID: &'static str = "late";
        const NAME: &'static str = "Late";
        const VERSION: &'static str = "1.0.0";
    }
    impl NoDependencies for LateSchema {}
    impl SchemaExtension for LateSchema {
        fn schemas(&self) -> Vec<SchemaDefinition> { vec![] }
        fn migration_weight(&self) -> u32 { 200 }
    }

    let registry = ExtensionBuilder::new()
        .extension(LateSchema::default())  // Registered first
        .extension(EarlySchema::default())
        .build()
        .unwrap();

    let schemas: Vec<_> = registry.schema_extensions().collect();
    assert_eq!(schemas[0].id(), "early");  // But sorted by weight
    assert_eq!(schemas[1].id(), "late");
}
```

---

## 3. Blog Extension Tests

### 3.1 Repository Tests

**File**: `/var/www/html/systemprompt-template/extensions/blog/tests/repository_test.rs`

```rust
//! Repository integration tests.

use sqlx::PgPool;
use systemprompt_blog_extension::{Content, ContentRepository};

#[sqlx::test]
async fn test_content_repository_upsert(pool: PgPool) {
    let repo = ContentRepository::new(&pool);

    let content = Content {
        id: uuid::Uuid::new_v4().to_string(),
        slug: "test-article".into(),
        title: "Test Article".into(),
        description: "A test article".into(),
        body: "# Test\n\nThis is a test.".into(),
        author: "Test Author".into(),
        published_at: chrono::Utc::now(),
        keywords: "test, article".into(),
        kind: "article".into(),
        image: None,
        category_id: Some("blog".into()),
        source_id: "blog".into(),
        version_hash: "abc123".into(),
        public: true,
        links: serde_json::json!([]),
        updated_at: chrono::Utc::now(),
    };

    repo.upsert(&content).await.unwrap();

    let retrieved = repo.get_by_slug("blog", "test-article").await.unwrap();
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().title, "Test Article");
}

#[sqlx::test]
async fn test_content_repository_list_by_source(pool: PgPool) {
    let repo = ContentRepository::new(&pool);

    // Insert multiple items
    for i in 0..5 {
        let content = Content {
            id: uuid::Uuid::new_v4().to_string(),
            slug: format!("article-{}", i),
            title: format!("Article {}", i),
            source_id: "blog".into(),
            public: true,
            // ... other fields
            ..Default::default()
        };
        repo.upsert(&content).await.unwrap();
    }

    let items = repo.list_by_source("blog").await.unwrap();
    assert_eq!(items.len(), 5);
}
```

### 3.2 Service Tests

**File**: `/var/www/html/systemprompt-template/extensions/blog/tests/service_test.rs`

```rust
//! Service layer tests.

use systemprompt_blog_extension::{
    ContentService, SearchRequest, SearchService,
};

#[sqlx::test]
async fn test_search_service_query(pool: PgPool) {
    // Insert test data
    let content_service = ContentService::new(&pool);
    content_service.create_test_content().await.unwrap();

    let search_service = SearchService::new(&pool);

    let request = SearchRequest {
        query: Some("test".into()),
        source_id: None,
        category_id: None,
        kind: None,
        limit: 10,
        offset: 0,
    };

    let response = search_service.search(request).await.unwrap();
    assert!(!response.results.is_empty());
}

#[sqlx::test]
async fn test_search_service_filters(pool: PgPool) {
    let search_service = SearchService::new(&pool);

    // Test filtering by source
    let request = SearchRequest {
        query: None,
        source_id: Some("blog".into()),
        category_id: None,
        kind: Some("article".into()),
        limit: 10,
        offset: 0,
    };

    let response = search_service.search(request).await.unwrap();
    for result in &response.results {
        assert_eq!(result.source_id, "blog");
        assert_eq!(result.kind, "article");
    }
}
```

### 3.3 API Tests

**File**: `/var/www/html/systemprompt-template/extensions/blog/tests/api_test.rs`

```rust
//! API endpoint tests.

use axum::http::StatusCode;
use axum_test::TestServer;
use systemprompt_blog_extension::{api, BlogConfig, BlogExtension};

async fn setup_test_server() -> TestServer {
    let db = setup_test_db().await;
    let config = BlogConfig::default();
    let router = api::router(db, config);
    TestServer::new(router).unwrap()
}

#[tokio::test]
async fn test_query_endpoint() {
    let server = setup_test_server().await;

    let response = server
        .post("/query")
        .json(&serde_json::json!({
            "query": "test",
            "limit": 10
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);

    let body: serde_json::Value = response.json();
    assert!(body.get("results").is_some());
    assert!(body.get("total").is_some());
}

#[tokio::test]
async fn test_get_content_not_found() {
    let server = setup_test_server().await;

    let response = server.get("/blog/nonexistent-slug").await;

    assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_list_content_by_source() {
    let server = setup_test_server().await;

    let response = server.get("/blog").await;

    assert_eq!(response.status_code(), StatusCode::OK);

    let body: Vec<serde_json::Value> = response.json();
    // Initially empty
    assert!(body.is_empty());
}

#[tokio::test]
async fn test_generate_link() {
    let server = setup_test_server().await;

    let response = server
        .post("/links/generate")
        .json(&serde_json::json!({
            "target_url": "https://example.com/article",
            "campaign_id": "test-campaign",
            "source": "email"
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);

    let body: serde_json::Value = response.json();
    assert!(body.get("short_code").is_some());
    assert!(body.get("tracking_url").is_some());
}
```

### 3.4 Job Tests

**File**: `/var/www/html/systemprompt-template/extensions/blog/tests/job_test.rs`

```rust
//! Background job tests.

use systemprompt_blog_extension::{BlogConfig, ContentIngestionJob, ContentSource};
use systemprompt_traits::{Job, JobContext};
use tempfile::TempDir;

#[tokio::test]
async fn test_ingestion_job_empty_sources() {
    let pool = setup_test_db().await;
    let config = BlogConfig {
        content_sources: vec![],
        ..Default::default()
    };

    let ctx = JobContext::new()
        .with_db_pool(pool)
        .with_config(config);

    let job = ContentIngestionJob;
    let result = job.execute(&ctx).await.unwrap();

    assert!(result.success);
    assert_eq!(result.processed, 0);
}

#[tokio::test]
async fn test_ingestion_job_with_content() {
    let pool = setup_test_db().await;

    // Create temp directory with test content
    let temp_dir = TempDir::new().unwrap();
    let content_path = temp_dir.path().join("test-article.md");
    std::fs::write(&content_path, r#"---
title: "Test Article"
description: "A test"
author: "Test"
published_at: "2024-01-01T00:00:00Z"
keywords: "test"
kind: "article"
public: true
---

# Test Content
"#).unwrap();

    let config = BlogConfig {
        content_sources: vec![ContentSource {
            source_id: "test".into(),
            category_id: "test".into(),
            path: temp_dir.path().to_path_buf(),
            allowed_content_types: vec!["article".into()],
            enabled: true,
            override_existing: false,
        }],
        ..Default::default()
    };

    let ctx = JobContext::new()
        .with_db_pool(pool.clone())
        .with_config(config);

    let job = ContentIngestionJob;
    let result = job.execute(&ctx).await.unwrap();

    assert!(result.success);
    assert_eq!(result.processed, 1);

    // Verify content was ingested
    let content = ContentRepository::new(&pool)
        .get_by_slug("test", "test-article")
        .await
        .unwrap();
    assert!(content.is_some());
}
```

---

## 4. End-to-End Tests

### 4.1 Full Application Test

**File**: `/var/www/html/systemprompt-template/tests/e2e/full_flow_test.rs`

```rust
//! End-to-end tests for the complete application flow.

use reqwest::Client;
use std::time::Duration;
use tokio::time::sleep;

const BASE_URL: &str = "http://localhost:3000";

#[tokio::test]
#[ignore]  // Requires running server
async fn test_full_content_flow() {
    let client = Client::new();

    // 1. Health check
    let resp = client.get(format!("{}/health", BASE_URL))
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success());

    // 2. Query content (initially empty)
    let resp = client.post(format!("{}/api/v1/content/query", BASE_URL))
        .json(&serde_json::json!({
            "limit": 10
        }))
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success());

    // 3. Trigger ingestion job manually
    let resp = client.post(format!("{}/api/v1/scheduler/jobs/blog_content_ingestion/run", BASE_URL))
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success());

    // Wait for ingestion
    sleep(Duration::from_secs(2)).await;

    // 4. Query content (should have items now)
    let resp = client.post(format!("{}/api/v1/content/query", BASE_URL))
        .json(&serde_json::json!({
            "limit": 10
        }))
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success());

    let body: serde_json::Value = resp.json().await.unwrap();
    assert!(body["total"].as_u64().unwrap() > 0);
}

#[tokio::test]
#[ignore]
async fn test_link_tracking_flow() {
    let client = Client::new();

    // 1. Generate a tracking link
    let resp = client.post(format!("{}/api/v1/content/links/generate", BASE_URL))
        .json(&serde_json::json!({
            "target_url": "https://example.com/test",
            "campaign_id": "e2e-test",
            "source": "test"
        }))
        .send()
        .await
        .unwrap();

    let link: serde_json::Value = resp.json().await.unwrap();
    let short_code = link["short_code"].as_str().unwrap();

    // 2. Click the link
    let resp = client.get(format!("{}/r/{}", BASE_URL, short_code))
        .send()
        .await
        .unwrap();
    // Should redirect
    assert!(resp.status().is_redirection() || resp.status().is_success());

    // 3. Check link performance
    let link_id = link["id"].as_str().unwrap();
    let resp = client.get(format!("{}/api/v1/content/links/{}/performance", BASE_URL, link_id))
        .send()
        .await
        .unwrap();

    let performance: serde_json::Value = resp.json().await.unwrap();
    assert!(performance["clicks"].as_u64().unwrap() >= 1);
}
```

---

## 5. Compile-Time Tests

### 5.1 trybuild Tests

**File**: `/var/www/html/systemprompt-core/crates/shared/extension/tests/compile_fail/missing_dependency.rs`

```rust
// This file should fail to compile, verifying our type system works

use systemprompt_extension::*;

#[derive(Default)]
struct AuthExtension;
impl ExtensionType for AuthExtension {
    const ID: &'static str = "auth";
    const NAME: &'static str = "Auth";
    const VERSION: &'static str = "1.0.0";
}
impl NoDependencies for AuthExtension {}

#[derive(Default)]
struct BlogExtension;
impl ExtensionType for BlogExtension {
    const ID: &'static str = "blog";
    const NAME: &'static str = "Blog";
    const VERSION: &'static str = "1.0.0";
}
impl Dependencies for BlogExtension {
    type Deps = (AuthExtension, ());
}

fn main() {
    // This should fail to compile because AuthExtension is not registered
    let _ = ExtensionBuilder::new()
        .extension(BlogExtension::default())
        .build();
}
```

**File**: `/var/www/html/systemprompt-core/crates/shared/extension/tests/compile_tests.rs`

```rust
#[test]
fn compile_fail_tests() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile_fail/*.rs");
}
```

---

## 6. Performance Tests

### 6.1 Search Performance

**File**: `/var/www/html/systemprompt-template/extensions/blog/benches/search_bench.rs`

```rust
//! Search performance benchmarks.

use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn search_benchmark(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("search_query_simple", |b| {
        b.to_async(&rt).iter(|| async {
            let request = SearchRequest {
                query: Some("test".into()),
                limit: 10,
                ..Default::default()
            };
            black_box(search_service.search(request).await)
        })
    });

    c.bench_function("search_query_complex", |b| {
        b.to_async(&rt).iter(|| async {
            let request = SearchRequest {
                query: Some("rust programming tutorial".into()),
                source_id: Some("blog".into()),
                kind: Some("article".into()),
                limit: 50,
                offset: 0,
            };
            black_box(search_service.search(request).await)
        })
    });
}

criterion_group!(benches, search_benchmark);
criterion_main!(benches);
```

---

## 7. Validation Checklist

### 7.1 Extension Framework

- [ ] `ExtensionType` trait works with const metadata
- [ ] `Dependencies` trait enables compile-time checking
- [ ] `ExtensionBuilder` rejects missing dependencies at compile time
- [ ] `ExtensionBuilder` rejects duplicate IDs at runtime
- [ ] `ExtensionBuilder` rejects reserved paths
- [ ] `ExtensionRegistry` provides typed lookups
- [ ] Schema extensions ordered by migration weight
- [ ] API extensions mounted at correct paths

### 7.2 Blog Extension

- [ ] All 7 schemas install correctly
- [ ] Content CRUD operations work
- [ ] Search returns correct results
- [ ] Link generation creates valid short codes
- [ ] Link clicks are tracked
- [ ] Analytics aggregation works
- [ ] Ingestion job processes content
- [ ] Error handling is comprehensive

### 7.3 Integration

- [ ] Template compiles with blog extension
- [ ] Server starts and mounts routes
- [ ] Configuration loads correctly
- [ ] Database schemas install on startup
- [ ] Scheduler registers blog jobs
- [ ] All API endpoints respond

---

## 8. Documentation Validation

### 8.1 Check rustdoc

```bash
cd /var/www/html/systemprompt-core
cargo doc --no-deps -p systemprompt-extension

cd /var/www/html/systemprompt-template
cargo doc --no-deps -p systemprompt-blog-extension
```

### 8.2 Verify Examples Compile

```bash
# In extension framework
cargo test --doc -p systemprompt-extension

# In blog extension
cargo test --doc -p systemprompt-blog-extension
```

---

## 9. Execution Checklist

### Phase 4a: Extension Framework Tests
- [ ] Create unit tests for types.rs
- [ ] Create unit tests for hlist.rs
- [ ] Create unit tests for builder.rs
- [ ] Create unit tests for registry.rs
- [ ] Create compile-fail tests with trybuild
- [ ] Achieve 90%+ coverage

### Phase 4b: Blog Extension Tests
- [ ] Create repository tests
- [ ] Create service tests
- [ ] Create API handler tests
- [ ] Create job tests
- [ ] Achieve 80%+ coverage

### Phase 4c: Integration Tests
- [ ] Create E2E test for content flow
- [ ] Create E2E test for link tracking
- [ ] Test extension mounting
- [ ] Test schema installation

### Phase 4d: Performance Tests
- [ ] Create search benchmarks
- [ ] Create ingestion benchmarks
- [ ] Establish baseline metrics

### Phase 4e: Documentation
- [ ] Verify rustdoc builds without warnings
- [ ] Verify all doc examples compile
- [ ] Update README files

---

## 10. Success Criteria

| Criterion | Target | How to Verify |
|-----------|--------|---------------|
| All tests pass | 100% | `cargo test --workspace` |
| Coverage | 80%+ | `cargo tarpaulin` |
| No compile warnings | 0 | `cargo build --workspace 2>&1 | grep warning` |
| Doc examples compile | 100% | `cargo test --doc` |
| Performance baseline | <100ms search | `cargo bench` |
| E2E tests pass | 100% | Manual or CI |

---

## 11. Output Artifacts

After executing this phase:

1. **Comprehensive test suite** for extension framework
2. **Integration tests** for blog extension
3. **E2E tests** for template application
4. **Performance benchmarks** with baselines
5. **Documentation** validated and complete
6. **CI configuration** for automated testing
