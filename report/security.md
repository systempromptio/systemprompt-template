# Security -- extensions/ Code Review

**Severity**: P0
**Scope**: extensions/web, extensions/mcp/marketplace
**Estimated effort**: S (1-2 days)

## Summary

Three classes of security weakness exist in the extension codebase: dynamic SQL column/table names that bypass parameterized query protection, missing URL encoding on user-influenced query parameters, and absent input length validation on all MCP tool string inputs. None are trivially exploitable today because the dynamic SQL values are currently hardcoded at call sites, but any refactor that exposes them to user input creates a critical SQL injection surface.

## Findings

### SEC-01 Dynamic table/column names in SQL queries

- **File(s)**: `extensions/mcp/marketplace/src/tools/shared/slugs.rs:19-20, 54`
- **Crate**: marketplace
- **Impact**: SQL injection if table/column name parameters ever originate from user input. Currently safe only because callers hardcode the values (`"user_skills"`, `"skill_id"`, etc.).
- **Pattern**:
  ```rust
  // slugs.rs:19-20
  let query = format!(
      "SELECT {slug_col}, {id_col} FROM {table} WHERE user_id = $1 AND {slug_col} = ANY($2)"
  );
  ```
  The `table`, `slug_col`, and `id_col` parameters are interpolated directly into the query string. `sqlx::query_as` then binds `$1` and `$2`, but the structural parts of the query are unparameterized.
- **Fix**: Either:
  1. Replace the generic function with per-entity functions that use `sqlx::query_as!` with hardcoded SQL (preferred -- compile-time verification).
  2. Add an allowlist enum for valid table/column combinations and match on it, never accepting raw strings.

### SEC-02 Missing URL encoding on UTM parameters

- **File(s)**: `extensions/web/src/models/link.rs:130-147`
- **Crate**: web
- **Impact**: If UTM parameter values contain `&`, `=`, `#`, or space characters, the generated query string breaks. Malicious values could inject additional query parameters or fragment identifiers. In a redirect context, this enables open-redirect or reflected XSS via crafted campaign names.
- **Pattern**:
  ```rust
  // link.rs:132-134
  if let Some(ref source) = self.source {
      parts.push(format!("utm_source={source}")); // NOT URL-encoded
  }
  ```
  All five UTM fields (source, medium, campaign, term, content) are concatenated raw.
- **Fix**: URL-encode each value before interpolation. Use `urlencoding::encode()` or `percent_encoding` crate:
  ```rust
  parts.push(format!("utm_source={}", urlencoding::encode(source)));
  ```

### SEC-03 No input length validation on MCP tool parameters

- **File(s)**: All handler files in `extensions/mcp/marketplace/src/tools/*/handler.rs`
- **Crate**: marketplace
- **Impact**: Unbounded string inputs (name, description, content, instructions) are passed directly to database INSERT/UPDATE statements. An attacker or misbehaving client can:
  - Exhaust database storage with multi-megabyte skill content
  - Cause OOM in the MCP server during JSON serialization of oversized responses
  - Trigger database timeouts on large payload writes
- **Pattern**: Every `create_*` and `update_*` handler extracts string parameters from the MCP request and passes them through without length checks.
- **Fix**: Add a validation layer at handler entry that enforces maximum lengths:
  ```rust
  const MAX_NAME_LEN: usize = 256;
  const MAX_DESCRIPTION_LEN: usize = 4096;
  const MAX_CONTENT_LEN: usize = 65536;
  
  if name.len() > MAX_NAME_LEN {
      return Err(McpError::invalid_params("Name exceeds maximum length", None));
  }
  ```

### SEC-04 String interpolation in tracing::warn! not interpolated

- **File(s)**: `extensions/mcp/marketplace/src/tools/shared/plugin.rs:57, 85, 129`
- **Crate**: marketplace
- **Impact**: Low security impact but high operational impact. Log messages contain literal `{entity_kind}` instead of the actual value, making incident investigation blind to what entity type failed.
- **Pattern**:
  ```rust
  // plugin.rs:57
  tracing::warn!(error = %e, plugin_id = %plugin_id,
      "Failed to add {entity_kind} to target plugin");  // NOT interpolated
  ```
  `tracing::warn!` uses `format_args!` semantics, not `format!`. Named fields in the string literal are not interpolated from local variables.
- **Fix**: Use structured fields or explicit format:
  ```rust
  tracing::warn!(error = %e, plugin_id = %plugin_id, entity_kind = %entity_kind,
      "Failed to add entity to target plugin");
  ```

## Recommended fix order

1. **SEC-02** (URL encoding) -- Easiest fix, highest exploit potential in a web context
2. **SEC-03** (Input validation) -- Add validation constants and checks to handler entry points
3. **SEC-01** (Dynamic SQL) -- Replace generic function with per-entity typed queries
4. **SEC-04** (Log interpolation) -- Quick fix, improves incident response

## Verification

1. `cargo clippy -- -W clippy::format_in_format_args` to catch format string issues
2. `grep -rn 'format!.*SELECT\|format!.*INSERT\|format!.*UPDATE\|format!.*DELETE' extensions/` should return zero results after SEC-01 fix
3. Write a unit test for `UtmParams::to_query_string` with values containing `&`, `=`, `#`, and spaces
4. `grep -rn 'unwrap_or_else.*String::new' extensions/mcp/` to track silent serialization failures (related -- see error-handling report)
