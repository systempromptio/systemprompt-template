# Performance -- extensions/ Code Review

**Severity**: P4
**Scope**: extensions/web, extensions/mcp/marketplace
**Estimated effort**: M (3-5 days)

## Summary

The codebase has three classes of performance waste: unnecessary memory allocations from excessive `.clone()` (741 instances) and `.to_string()` (846 instances), N+1 query patterns in the marketplace slug resolution layer, and collection-level inefficiencies where data is cloned for sorting instead of sorted in place. None of these are likely to cause production incidents at current scale, but they establish patterns that degrade performance linearly with growth.

## Findings

### PERF-01 N+1 query pattern in plugin enrichment

- **File(s)**:
  - `extensions/mcp/marketplace/src/tools/get_plugin/handler.rs`
  - `extensions/mcp/marketplace/src/tools/list_plugins/handler.rs`
- **Crate**: marketplace
- **Impact**: When listing plugins, each plugin's associations are resolved via separate slug resolution calls. For N plugins with skill/agent/MCP associations, this generates 3N additional queries (one per entity type per plugin).
- **Pattern**: `list_plugins` fetches all plugins, then for each plugin calls:
  ```rust
  resolve_skill_uuids_to_slugs(pool, &skill_uuids).await   // query 1
  resolve_agent_uuids_to_slugs(pool, &agent_uuids).await   // query 2
  resolve_mcp_server_uuids_to_slugs(pool, &mcp_uuids).await // query 3
  ```
  `get_plugin` does the same for a single plugin (3 queries instead of 1 joined query).
- **Fix**: Batch resolution. Collect all UUIDs across all plugins, resolve in 3 bulk queries, then distribute results:
  ```rust
  // Collect all UUIDs
  let all_skill_uuids: Vec<&str> = plugins.iter().flat_map(|p| &p.skill_ids).collect();
  // One bulk query
  let skill_slug_map = resolve_skill_uuids_to_slugs_batch(pool, &all_skill_uuids).await?;
  // Distribute
  for plugin in &mut plugins {
      plugin.skill_slugs = plugin.skill_ids.iter()
          .filter_map(|id| skill_slug_map.get(id).cloned())
          .collect();
  }
  ```

### PERF-02 Excessive .clone() on Arc<PgPool>

- **File(s)**: 20+ files in `extensions/web/src/` (43 occurrences across services, repositories, admin)
- **Crate**: web
- **Impact**: Every `Arc<PgPool>` clone increments an atomic counter. While cheap individually, 43 clone sites executed per-request add measurable overhead under load. More importantly, the cloning is often unnecessary -- the pool is borrowed for the duration of the function.
- **Pattern**:
  ```rust
  // Typical: clone Arc to pass to function that only borrows it
  let service = LinkService::new(pool.clone());
  service.do_something().await
  ```
- **Fix**: This is resolved by ARCH-01 (remove Arc<PgPool>). Once functions take `&PgPool`, the clones disappear.

### PERF-03 TierLimits struct cloned on every cache hit

- **File(s)**: `extensions/web/src/admin/` (cache-related files)
- **Crate**: web
- **Impact**: The tier enforcement cache returns `cached.limits.clone()` on every cache hit. If `TierLimits` is a large struct (multiple fields, nested collections), this is a significant allocation on every request that checks tier limits.
- **Pattern**:
  ```rust
  // Returns clone of entire struct on cache hit
  cached.limits.clone()
  ```
- **Fix**: Wrap the cached value in `Arc<TierLimits>` so cache hits return an `Arc::clone` (cheap pointer increment) instead of a deep clone:
  ```rust
  Arc::new(limits) // on cache insert
  cached_limits.clone() // Arc::clone -- cheap
  ```

### PERF-04 Collection cloning for sorting in marketplace

- **File(s)**: `extensions/web/src/homepage/` (marketplace-related files)
- **Crate**: web
- **Impact**: Marketplace ranking and filtering operations clone entire collections before sorting. The original collection is consumed by the sort, so the clone is wasted.
- **Pattern**:
  ```rust
  let mut sorted = items.clone(); // Unnecessary clone
  sorted.sort_by(|a, b| ...);
  ```
- **Fix**: Sort in place if the original order isn't needed:
  ```rust
  items.sort_by(|a, b| ...);
  ```
  Or use `sort_unstable_by` for better cache performance on large collections.

### PERF-05 Unnecessary String allocations

- **File(s)**: Distributed across all crates (846 `.to_string()` calls)
- **Crate**: all
- **Impact**: Many `.to_string()` calls are on string literals or values that could be `&str`. While each is cheap, the pattern encourages heap allocation where borrowing would suffice.
- **Specific examples**:
  ```rust
  // extensions/web/src/ -- could use &str or Cow<str>
  ids.push(entry.file_name().to_string_lossy().to_string());
  // Better:
  ids.push(entry.file_name().to_string_lossy().into_owned());
  ```
- **Fix**: Not all 846 instances need changing. Focus on:
  1. Hot paths (request handlers, per-request operations)
  2. Loop bodies where the same allocation happens per iteration
  3. Function signatures that take `String` but could take `&str` or `impl Into<String>`

### PERF-06 Sequential slug resolution could be concurrent

- **File(s)**: `extensions/mcp/marketplace/src/tools/create_plugin/handler.rs`, `update_plugin/handler.rs`
- **Crate**: marketplace
- **Impact**: Plugin creation/update resolves skill, agent, and MCP server slugs sequentially. These are independent database queries that could run concurrently.
- **Pattern**:
  ```rust
  let skill_ids = resolve_skill_slugs(pool, user_id, &skills).await?;
  let agent_ids = resolve_agent_slugs(pool, user_id, &agents).await?;  // waits for skills
  let mcp_ids = resolve_mcp_server_slugs(pool, user_id, &servers).await?; // waits for agents
  ```
- **Fix**: Use `tokio::try_join!` for concurrent resolution:
  ```rust
  let (skill_ids, agent_ids, mcp_ids) = tokio::try_join!(
      resolve_skill_slugs(pool, user_id, &skills),
      resolve_agent_slugs(pool, user_id, &agents),
      resolve_mcp_server_slugs(pool, user_id, &servers),
  )?;
  ```

## Recommended fix order

1. **PERF-06** -- Concurrent slug resolution (5 min, instant latency improvement)
2. **PERF-01** -- N+1 query batching (1 day, scales with data growth)
3. **PERF-03** -- Arc<TierLimits> cache (1 hour, per-request improvement)
4. **PERF-02** -- Remove Arc<PgPool> clones (done as part of ARCH-01)
5. **PERF-04** -- In-place sorting (30 min, minor)
6. **PERF-05** -- String allocation audit (ongoing, focus on hot paths)

## Verification

1. Before/after benchmarks using `just benchmark` (governance endpoint load testing)
2. `cargo clippy -- -W clippy::redundant_clone` to catch clones on values that are moved afterward
3. `grep -rn '\.clone()' extensions/ | wc -l` -- track reduction (target: <400)
4. Database query count: add `sqlx` query logging and count queries per `list_plugins` call (target: O(1) not O(N))
