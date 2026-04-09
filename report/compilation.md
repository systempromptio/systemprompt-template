# Compilation -- extensions/ Code Review

**Severity**: P3
**Scope**: extensions/web
**Estimated effort**: S (< 1 day)

## Summary

The web extension has visibility mismatches between module declarations and re-exports, unused imports that trigger compiler warnings, and dead code functions that inflate the binary and confuse maintainers. These are the easiest findings to fix and should be addressed first to clean up the compiler output so real warnings become visible.

## Findings

### COMP-01 Visibility mismatch on SSR router exports

- **File(s)**: `extensions/web/src/admin/routes/mod.rs:5-6`
- **Crate**: web
- **Impact**: The module re-exports `admin_ssr_router` and `workspace_ssr_router` as `pub` while the adjacent line exports other functions as `pub(super)`. The parent module (`admin/mod.rs:24`) then re-exports these as `pub`. This works but creates an inconsistent visibility contract -- some route builders are public API, others are module-private, with no documented reason for the distinction.
- **Pattern**:
  ```rust
  // admin/routes/mod.rs
  pub(super) use admin::{build_admin_only_routes, build_auth_read_routes};
  pub use ssr::{admin_ssr_router, workspace_ssr_router};     // pub (wider)
  pub(super) use user::build_auth_write_routes;               // pub(super) (narrower)
  ```
- **Fix**: Align visibility. If `admin_ssr_router` and `workspace_ssr_router` need to be used from `extension_impl.rs`, they should be `pub(crate)` at minimum. If not, they should be `pub(super)` like the others. Audit usage and apply the narrowest visibility that compiles.

### COMP-02 Unused import in hooks_track processing

- **File(s)**: `extensions/web/src/admin/handlers/hooks_track/processing.rs:9`
- **Crate**: web
- **Impact**: `hooks_track` is imported from `repositories` but never used. Compiler warning noise.
- **Pattern**:
  ```rust
  use crate::admin::repositories::{conversation_analytics, hooks_track, usage_aggregations};
  //                                                        ^^^^^^^^^^^ unused
  ```
- **Fix**: Remove `hooks_track` from the import list.

### COMP-03 Dead code in SSR routes

- **File(s)**: `extensions/web/src/admin/routes/ssr.rs`
- **Crate**: web
- **Impact**: Multiple route builder functions appear to be defined but never called from outside the module. Functions identified:
  - `workspace_routes()`
  - `public_routes()`
  - `dashboard_routes()`
  - `user_page_routes()`
  - `my_routes()`
  - `entity_routes()`
  - `org_routes()`
  
  These may be called internally by `admin_ssr_router` or `workspace_ssr_router`, which would make them used. However, if the top-level router functions themselves are dead, the entire tree is dead code.
- **Fix**: 
  1. Verify whether `admin_ssr_router` and `workspace_ssr_router` are actually called from `extension_impl.rs`
  2. If they are: the internal functions are fine (just not `pub`)
  3. If not: remove the entire SSR route tree and the `ssr` module
  4. Run `cargo build` with `#[deny(dead_code)]` to catch any remaining dead code

### COMP-04 Deleted playbooks module still referenced

- **File(s)**: Git status shows deleted files:
  - `extensions/web/src/playbooks/content_provider.rs`
  - `extensions/web/src/playbooks/mod.rs`
  - `extensions/web/src/playbooks/page_provider.rs`
  - `extensions/web/src/playbooks/provider.rs`
- **Crate**: web
- **Impact**: If `lib.rs` still has `mod playbooks;`, compilation fails. If the module reference has been removed, this is clean. Needs verification.
- **Fix**: Ensure `mod playbooks` is removed from `lib.rs` and any `use crate::playbooks::*` imports are removed.

## Recommended fix order

1. **COMP-04** -- Verify deleted playbooks module doesn't break compilation
2. **COMP-02** -- Remove unused import (1 line)
3. **COMP-01** -- Align visibility modifiers
4. **COMP-03** -- Audit and remove dead SSR route code

## Verification

1. `cargo build 2>&1 | grep -c warning` -- should be 0
2. `cargo clippy -- -D warnings -D dead_code` -- should pass clean
3. `cargo clippy -- -W unused-imports` -- should report no unused imports
