# Core Fix Required: Send Bound Issue

## Problem

The `enforce_rbac_from_registry` function in core holds a tracing span guard across await points, making the returned future not `Send`.

```
error[E0277]: `*mut ()` cannot be sent between threads safely
  --> src/server/handlers/tools.rs
   |
   | `*mut ()` cannot be sent between threads safely
   | within this `impl Future<Output = Result<...>>`
```

## Location

`/var/www/html/systemprompt-core/crates/domain/mcp/src/middleware/rbac.rs:74`

```rust
// Line 74 - problematic:
let _guard = systemprompt_core_logging::SystemSpan::new("mcp_rbac").enter();
// ... await calls on lines 76+ ...
```

## Fix Options

### Option 1: Use `#[tracing::instrument]` (Recommended)

```rust
#[tracing::instrument(name = "mcp_rbac", skip_all)]
pub async fn enforce_rbac_from_registry(
    mcp_context: &McpContext<RoleServer>,
    server_name: &str,
) -> Result<AuthResult, McpError> {
    // Remove: let _guard = systemprompt_core_logging::SystemSpan::new("mcp_rbac").enter();

    let services_config = ConfigLoader::load().await.map_err(|e| {
        // ...
```

### Option 2: Drop Guard Before Await

```rust
pub async fn enforce_rbac_from_registry(
    mcp_context: &McpContext<RoleServer>,
    server_name: &str,
) -> Result<AuthResult, McpError> {
    {
        let _guard = systemprompt_core_logging::SystemSpan::new("mcp_rbac").enter();
        // sync logging only
    } // guard dropped here

    let services_config = ConfigLoader::load().await.map_err(|e| {
        // ...
```

### Option 3: Use `Span::in_scope` for sync blocks only

```rust
let span = systemprompt_core_logging::SystemSpan::new("mcp_rbac");
span.in_scope(|| {
    // sync-only code
});
// await calls here
```

## After Fix

```bash
cd /var/www/html/systemprompt-mcp-infrastructure
cargo update -p systemprompt
cargo check
```
