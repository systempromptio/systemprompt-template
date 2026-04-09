# Error Handling -- extensions/ Code Review

**Severity**: P1
**Scope**: extensions/web, extensions/mcp/systemprompt, extensions/mcp/marketplace
**Estimated effort**: L (1-2 weeks)

## Summary

The extension codebase contains approximately 785 uses of `unwrap()`, `expect()`, or `panic!()` across an async server runtime where any panic kills the task (and potentially the process). Additionally, a pervasive pattern of `unwrap_or_else(|_| ...)` silently swallows errors, returning empty strings or vectors that mask bugs in production. Error types are defined per-module but inconsistently applied, with some handlers using `map_err` properly while others log-and-discard.

## Findings

### ERR-01 Silent serialization failures in MCP handlers

- **File(s)**:
  - `extensions/mcp/marketplace/src/tools/update_skill/handler.rs:92`
  - `extensions/mcp/marketplace/src/tools/update_agent/handler.rs:89`
  - `extensions/mcp/marketplace/src/tools/update_mcp_server/handler.rs:105`
  - `extensions/mcp/marketplace/src/tools/list_plugins/handler.rs:113-115`
  - `extensions/mcp/marketplace/src/tools/manage_secrets/handler.rs:136, 176, 213`
- **Crate**: marketplace
- **Impact**: When `serde_json::to_string()` fails, the handler returns an empty string to the client. The client receives a "success" response with truncated or missing data. No error is logged. The user sees a blank result with no indication of failure.
- **Pattern**:
  ```rust
  // update_skill/handler.rs:92
  serde_json::to_string_pretty(&response)
      .unwrap_or_else(|_| String::new());
  ```
  This appears in 9 locations across 7 handler files.
- **Fix**: Propagate the error:
  ```rust
  let body = serde_json::to_string_pretty(&response)
      .map_err(|e| McpError::internal_error(format!("Serialization failed: {e}"), None))?;
  ```

### ERR-02 Silent database query failures in slug resolution

- **File(s)**: `extensions/mcp/marketplace/src/tools/shared/slugs.rs:56-60`
- **Crate**: marketplace
- **Impact**: `resolve_uuids_to_slugs_generic` silently returns an empty Vec when the database query fails. Callers (get_plugin, list_plugins) display entities as having zero associations instead of reporting the error. Data appears missing rather than broken.
- **Pattern**:
  ```rust
  // slugs.rs:56-60
  let rows: Vec<(String, String)> = sqlx::query_as(&query)
      .bind(uuids)
      .fetch_all(pool.as_ref())
      .await
      .unwrap_or_else(|_| Vec::new()); // DB error silently eaten
  ```
- **Fix**: Change return type to `Result<Vec<String>, McpError>` and propagate:
  ```rust
  .await
  .map_err(|e| McpError::internal_error(format!("Failed to resolve slugs: {e}"), None))?;
  ```

### ERR-03 Excessive unwrap/expect in web extension

- **File(s)**: Distributed across 414 files in `extensions/web/src/`
- **Crate**: web
- **Impact**: ~785 instances of `unwrap()`, `expect()`, or `panic!()`. In an async Tokio runtime, a panic in a spawned task aborts that task. In a non-spawned handler, it propagates up and returns a 500 to the client. Repeated panics can degrade service availability.
- **Top offenders** (by count):
  - `admin/handlers/hooks_track/helpers.rs` -- 8 instances
  - `admin/handlers/hooks_track/ai_context.rs` -- 6 instances
  - `homepage/provider.rs` -- 5 instances (marketplace ranking math)
  - `admin/handlers/marketplace_upload/api.rs` -- 5 instances
- **Pattern categories**:
  1. **Numeric conversions**: `i32::try_from(x).unwrap_or(0)` -- safe but masks data issues
  2. **JSON parsing**: `serde_json::from_str(&s).unwrap()` -- panics on malformed data
  3. **Option access**: `.unwrap()` on HashMap lookups -- panics if key missing
  4. **Regex compilation**: `Regex::new(...).unwrap()` -- safe for static patterns, but should use `LazyLock`
- **Fix**: Prioritize by crash severity:
  1. Replace JSON `.unwrap()` with `?` or `.ok()` with logging
  2. Replace HashMap `.unwrap()` with `.get()` + error handling
  3. Move static regex to `LazyLock<Regex>` (compile once, can't fail at runtime)
  4. Leave `unwrap_or(0)` for numeric casts (acceptable fallback)

### ERR-04 Inconsistent error types across modules

- **File(s)**:
  - `extensions/web/src/error.rs` -- defines `BlogError`, `MarketplaceError`
  - `extensions/mcp/marketplace/src/error.rs` -- defines MCP-specific errors
  - `extensions/mcp/systemprompt/src/error.rs` -- defines MCP-specific errors
- **Crate**: all
- **Impact**: No unified error type means each handler makes ad-hoc decisions about how to convert internal errors to HTTP/MCP responses. Some modules use `map_err` chains, others use `unwrap_or_else` with logging, and others use raw `?` with implicit conversions.
- **Pattern**: The web crate defines error enums but doesn't implement `IntoResponse` consistently. The MCP crates each define their own error types but share no common base.
- **Fix**: Not proposing a full error rewrite, but:
  1. Audit all `unwrap_or_else(|e| { tracing::warn!(...); ... })` patterns -- if the operation is fallible, the return type should reflect it
  2. Ensure all public handler functions return `Result<T, E>` where `E` implements the appropriate response conversion
  3. Consider a shared error module for the two MCP crates

### ERR-05 Fire-and-forget logging loses audit events on crash

- **File(s)**:
  - `extensions/mcp/marketplace/src/server/mod.rs:126, 136, 148, 162`
  - `extensions/mcp/systemprompt/src/server.rs:179, 189, 201, 215`
- **Crate**: marketplace, systemprompt
- **Impact**: Audit log writes (RBAC enforcement, tool access events) are dispatched via `tokio::spawn` without tracking the JoinHandle. If the server shuts down or the task panics, the audit record is silently lost. For compliance-sensitive deployments, this means access events may not be recorded.
- **Pattern**:
  ```rust
  // server/mod.rs:126
  tokio::spawn(async move {
      record_mcp_access(&pool, &user_id, &server, &tool, "authenticated").await;
  });
  ```
  8 total fire-and-forget spawns across both MCP crates.
- **Fix**:
  1. For critical audit events: await the write inline (acceptable latency cost for security)
  2. For non-critical telemetry: use a bounded channel + background flush task that drains on shutdown
  3. At minimum: capture JoinHandles and join them during graceful shutdown

### ERR-06 Config initialization errors silently discarded

- **File(s)**: `extensions/web/src/extension.rs` (via `log_and_discard_err` helper)
- **Crate**: web
- **Impact**: After the first call, `OnceLock` caches the error. Subsequent calls to the same config function log nothing and return `None`. Operators see one error log on startup and then silent feature degradation.
- **Pattern**:
  ```rust
  fn log_and_discard_err<T: Clone>(
      lock: &OnceLock<Result<Option<T>, ConfigError>>,
      init: fn() -> Result<Option<T>, ConfigError>,
      msg: &str,
  ) -> Option<T> {
      match lock.get_or_init(init) {
          Ok(val) => val.clone(),
          Err(e) => {
              tracing::error!(error = %e, "{msg}");
              None // Logged once, then silently None forever
          }
      }
  }
  ```
- **Fix**: Cache `Option<T>` instead of `Result<Option<T>, _>`. Log on init failure but store `None` as the cached value, not the error.

## Recommended fix order

1. **ERR-01** -- Silent serialization failures (9 instances, mechanical fix, high user impact)
2. **ERR-02** -- Silent slug resolution failures (1 function, affects list/get plugin responses)
3. **ERR-05** -- Fire-and-forget audit logging (compliance risk)
4. **ERR-03** -- Unwrap/expect reduction (large effort, prioritize JSON and HashMap unwraps first)
5. **ERR-06** -- Config caching (low frequency, startup-only)
6. **ERR-04** -- Error type unification (design work, do alongside architecture improvements)

## Verification

1. `grep -rn 'unwrap_or_else.*String::new\|unwrap_or_else.*Vec::new' extensions/` -- should return 0 after ERR-01/ERR-02
2. `grep -rn '\.unwrap()' extensions/ | wc -l` -- track reduction over time (target: <100)
3. `grep -rn 'tokio::spawn' extensions/mcp/` -- each spawn should either be awaited or have its JoinHandle tracked
4. Integration test: send malformed JSON to MCP tool handlers and verify error response (not empty string)
