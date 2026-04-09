# Architecture -- extensions/ Code Review

**Severity**: P2
**Scope**: extensions/web, extensions/mcp/systemprompt, extensions/mcp/marketplace
**Estimated effort**: L (2-3 weeks)

## Summary

The extension codebase suffers from five structural problems: a redundant `Arc<PgPool>` wrapping pattern used 43+ times across 20 files, significant code duplication between the two MCP server crates, missing transactional guarantees for multi-step database operations, a monolithic web extension that bundles unrelated concerns into a single 57K-LOC crate, and near-identical entity management code repeated for skills/agents/MCP servers with no shared abstraction.

## Findings

### ARCH-01 Arc<PgPool> double-wrapping anti-pattern

- **File(s)**: 20+ files in `extensions/web/src/`, all files in `extensions/mcp/marketplace/src/tools/shared/slugs.rs`
- **Crate**: web, marketplace
- **Impact**: `sqlx::PgPool` is internally reference-counted (`Pool` contains an `Arc<SharedPool>`). Wrapping it in another `Arc` adds unnecessary indirection, an extra atomic reference count, and confusing API signatures. Every function that takes `Arc<PgPool>` forces callers to clone the Arc unnecessarily.
- **Pattern**:
  ```rust
  // extensions/web/src/services/link.rs:17
  pub fn new(pool: Arc<PgPool>) -> Self { ... }
  
  // extensions/mcp/marketplace/src/tools/shared/slugs.rs:7
  async fn resolve_slugs_generic(
      pool: &Arc<PgPool>,  // &Arc<PgPool> -- double indirection
      ...
  ```
  43 occurrences across 20 files in the web crate alone, plus all slug/handler functions in marketplace.
- **Fix**: Replace `Arc<PgPool>` with `PgPool` (owned, cheap to clone) or `&PgPool` (borrowed). This is a mechanical refactor:
  ```rust
  pub fn new(pool: PgPool) -> Self { ... }
  // or
  async fn resolve_slugs_generic(pool: &PgPool, ...) { ... }
  ```

### ARCH-02 Code duplication between MCP crates

- **File(s)**:
  - `extensions/mcp/marketplace/src/server/mod.rs:213-283` (logging functions)
  - `extensions/mcp/systemprompt/src/server_logging.rs` (same logging functions)
  - `extensions/mcp/marketplace/src/server/mod.rs:126-162` (spawn pattern)
  - `extensions/mcp/systemprompt/src/server.rs:179-215` (identical spawn pattern)
  - `extensions/mcp/marketplace/src/main.rs` (bootstrap)
  - `extensions/mcp/systemprompt/src/main.rs` (identical bootstrap)
- **Crate**: marketplace, systemprompt
- **Impact**: Bug fixes or security patches to logging, RBAC enforcement, or bootstrap must be applied in two places. The `record_mcp_access` and `record_mcp_access_rejected` functions are copy-pasted with identical SQL and logic. The 4-spawn authentication/authorization pattern is duplicated verbatim.
- **Pattern**:
  ```rust
  // Identical in both crates:
  async fn record_mcp_access(pool: &DbPool, user_id: &str, server: &str, tool: &str, action: &str) {
      let Some(pg_pool) = pool.pool() else { return; };
      // ... identical SQL INSERT ...
  }
  ```
- **Fix**: Extract shared MCP server infrastructure into a shared module:
  1. Create `extensions/mcp/shared/` (or a `systemprompt-mcp-common` crate)
  2. Move `record_mcp_access`, `record_mcp_access_rejected`, and the auth spawn pattern
  3. Both crates depend on the shared module

### ARCH-03 Missing transactional guarantees for multi-step operations

- **File(s)**:
  - `extensions/mcp/marketplace/src/tools/shared/plugin.rs:7-66` (create entity + add to plugin)
  - `extensions/mcp/marketplace/src/tools/create_plugin/handler.rs` (create plugin + set associations)
  - `extensions/mcp/marketplace/src/tools/update_plugin/handler.rs` (update plugin + set associations)
- **Crate**: marketplace
- **Impact**: `create_skill` performs two operations: (1) insert the skill, (2) add it to the user's plugin. These are separate database calls with no transaction boundary. If step 2 fails, the skill exists in the database but is not associated with any plugin -- an orphan record the user cannot manage.
- **Pattern**:
  ```rust
  // plugin.rs -- add_to_plugin is called AFTER the skill/agent is created
  // If this fails, the entity exists but isn't in any plugin
  pub async fn add_to_plugin(db_pool: &DbPool, user_id: &UserId, entity_id: &str, ...) -> Option<String> {
      // ... multiple DB calls without transaction ...
  }
  ```
  The `create_plugin` handler similarly performs 4 sequential operations (create plugin, set skills, set agents, set MCP servers) without a transaction.
- **Fix**: Wrap multi-step operations in a transaction:
  ```rust
  let mut tx = pool.begin().await?;
  // step 1: create entity
  // step 2: add to plugin
  tx.commit().await?;
  ```

### ARCH-04 Monolithic web extension

- **File(s)**: `extensions/web/src/` -- 414 files, ~57,400 LOC
- **Crate**: web
- **Impact**: A single crate contains:
  - Homepage rendering and navigation
  - Blog/content management
  - Admin dashboard (SSR, SSE, handlers)
  - Link analytics and campaign tracking
  - Marketplace upload and sync
  - Tier enforcement and gamification
  - Webhook processing
  - Content ingestion and search
  - Repository layer for 10+ entity types
  
  This means:
  - Compile times scale with the entire crate (no incremental benefit from changing one subsystem)
  - Feature boundaries are blurred (gamification code can reach into blog internals)
  - Testing in isolation is impossible
  - A bug in any subsystem can break the entire extension
- **Fix**: This is a large refactor. Recommended approach:
  1. Identify natural module boundaries (admin, content, analytics, marketplace)
  2. Extract each into its own extension crate
  3. Use the extension trait system to compose them
  4. Start with the least-coupled module (e.g., link analytics)

### ARCH-05 Entity management code triplication

- **File(s)**: `extensions/mcp/marketplace/src/tools/shared/plugin.rs:7-133`
- **Crate**: marketplace
- **Impact**: The `add_to_plugin` and `auto_add_to_default_plugin` functions contain three nearly identical match arms for skills, agents, and MCP servers. Each arm does the same thing: get current IDs, check for duplicates, push the new ID, call the setter. The only difference is the type (`SkillId` vs `AgentId` vs `McpServerId`) and the setter function.
- **Pattern**:
  ```rust
  // Repeated 3 times in add_to_plugin AND 3 times in auto_add_to_default_plugin (6 total):
  "skill" => {
      let mut ids: Vec<SkillId> = assoc.skill_ids;
      let new_id = SkillId::new(entity_id);
      if !ids.iter().any(|id| id.as_ref() == entity_id) { ids.push(new_id); }
      set_plugin_skills(&pool, &assoc.plugin.id, &ids).await
  }
  "agent" => { /* identical logic, different types */ }
  "mcp_server" => { /* identical logic, different types */ }
  ```
- **Fix**: Extract a generic helper or use a trait:
  ```rust
  async fn add_entity_to_association<Id: From<String> + AsRef<str>>(
      pool: &PgPool, plugin_id: &str, existing: Vec<Id>, new_id: &str,
      setter: impl AsyncFn(&PgPool, &str, &[Id]) -> Result<()>,
  ) -> Result<()> { ... }
  ```

### ARCH-06 Database pool availability check repeated 40+ times

- **File(s)**: Every handler in `extensions/mcp/marketplace/src/tools/*/handler.rs`
- **Crate**: marketplace
- **Impact**: Every MCP tool handler starts with:
  ```rust
  let pool = self.pool.as_ref()
      .ok_or_else(|| McpError::internal_error("Database pool not available", None))?;
  ```
  This is pure boilerplate that obscures the actual handler logic. If the error message or behavior needs to change, 20+ files must be updated.
- **Fix**: Move the check to the dispatch layer or provide a helper:
  ```rust
  impl MarketplaceServer {
      fn require_pool(&self) -> Result<&Arc<PgPool>, McpError> {
          self.pool.as_ref()
              .ok_or_else(|| McpError::internal_error("Database pool not available", None))
      }
  }
  ```

### ARCH-07 Variable naming contradiction: `first_plugin` uses `.last()`

- **File(s)**: `extensions/mcp/marketplace/src/tools/shared/plugin.rs:89`
- **Crate**: marketplace
- **Impact**: Code readability. Variable named `first_plugin` is assigned `plugins.last()`. This is either a bug (should be `.first()`) or a misleading name.
- **Pattern**:
  ```rust
  let first_plugin = plugins.last()?; // Named "first" but takes "last"
  ```
- **Fix**: Clarify intent. If the most recently created plugin is desired, rename to `default_plugin` or `latest_plugin`. If the first plugin is desired, change to `.first()`.

## Recommended fix order

1. **ARCH-06** -- Pool availability helper (30 min, reduces noise for all other fixes)
2. **ARCH-01** -- Arc<PgPool> removal (mechanical, 1-2 days, cleans up all function signatures)
3. **ARCH-02** -- MCP crate deduplication (1-2 days, prevents future drift)
4. **ARCH-03** -- Transaction boundaries (1 day, data integrity)
5. **ARCH-05** -- Entity triplication (1 day, reduces plugin.rs by ~60%)
6. **ARCH-07** -- Variable naming (5 min)
7. **ARCH-04** -- Monolith decomposition (multi-week, plan separately)

## Verification

1. `grep -rn 'Arc<PgPool>' extensions/` should return 0 after ARCH-01
2. `diff extensions/mcp/marketplace/src/server/mod.rs extensions/mcp/systemprompt/src/server_logging.rs` should show no shared logic after ARCH-02
3. `grep -rn 'ok_or_else.*Database pool' extensions/mcp/marketplace/` should return exactly 1 result (the helper) after ARCH-06
4. `cargo build` with all changes -- verify no regressions
