# MCP Binary Resolution Fix

## Problem

The MCP server YAML config has `binary` and `path` fields that are **not used** for binary resolution.

### Current YAML Config
```yaml
mcp_servers:
  systemprompt-admin:
    binary: "systemprompt-admin"      # IGNORED
    path: "extensions/mcp/admin"      # IGNORED for binary resolution
    port: 5002
    enabled: true
```

### Current Code Flow (Broken)

**spawner.rs:23**
```rust
let binary_path = BinaryPaths::resolve_binary(&config.name)
```

Uses `config.name` (the server key, e.g., "systemprompt-admin"), completely ignoring:
- `binary` field from YAML
- `path` field from YAML

**BinaryPaths::resolve_binary** (paths.rs:9-44)
```rust
pub fn resolve_binary(binary_name: &str) -> Result<PathBuf> {
    let config = Config::global();

    // Only checks binary_dir from profile
    if let Some(binary_dir) = &config.binary_dir {
        let binary_path = PathBuf::from(binary_dir).join(binary_name);
        if binary_path.exists() {
            return Self::ensure_absolute(binary_path);
        }
    }

    // Falls back to cargo_target/{debug,release}
    let cargo_target_dir = PathBuf::from(&config.cargo_target_dir);
    let debug_path = cargo_target_dir.join("debug").join(binary_name);
    let release_path = cargo_target_dir.join("release").join(binary_name);
    // ...
}
```

## Why This Is Wrong

1. **`binary` field is pointless** - It exists in the struct but is never used
2. **`path` field only used for source** - Only used for building/manifests, not runtime
3. **Inconsistent naming** - Server key must match binary name exactly
4. **No flexibility** - Can't have server name differ from binary name

## Required Changes

### 1. Make `binary` Required (Not Optional)

**File:** `core/crates/shared/models/src/mcp/deployment.rs`

```rust
// BEFORE
pub struct Deployment {
    pub binary: Option<String>,  // Optional, ignored
    pub path: Option<String>,
    // ...
}

// AFTER
pub struct Deployment {
    pub binary: String,          // REQUIRED - the actual binary name
    pub path: String,            // REQUIRED - path to crate/binary location
    // ...
}
```

### 2. Use `binary` Field in Spawner

**File:** `core/crates/domain/mcp/src/services/process/spawner.rs`

```rust
// BEFORE (line 23)
let binary_path = BinaryPaths::resolve_binary(&config.name)

// AFTER
let binary_path = BinaryPaths::resolve_binary(&config.binary)
```

### 3. Add `binary` to McpServerConfig

**File:** `core/crates/shared/models/src/mcp/server.rs`

```rust
pub struct McpServerConfig {
    pub name: String,           // Server identifier (key in YAML)
    pub binary: String,         // ADD: Binary name to execute
    pub crate_path: PathBuf,
    // ...
}
```

Update `from_manifest_and_deployment`:
```rust
pub fn from_manifest_and_deployment(
    name: String,
    manifest: &super::registry::ServerManifest,
    deployment: &super::deployment::Deployment,
    crate_path: PathBuf,
) -> Self {
    Self {
        name,
        binary: deployment.binary.clone(),  // ADD
        crate_path,
        // ...
    }
}
```

### 4. Update BinaryPaths to Accept Path Hint

**File:** `core/crates/infra/config/src/paths.rs`

```rust
// Add new method that accepts optional path hint
pub fn resolve_binary_with_path(binary_name: &str, crate_path: Option<&Path>) -> Result<PathBuf> {
    let config = Config::global();

    // 1. First check binary_dir (production)
    if let Some(binary_dir) = &config.binary_dir {
        let binary_path = PathBuf::from(binary_dir).join(binary_name);
        if binary_path.exists() {
            return Self::ensure_absolute(binary_path);
        }
    }

    // 2. If crate_path provided, check its target directory (development)
    if let Some(crate_path) = crate_path {
        let debug_path = crate_path.join("target/debug").join(binary_name);
        let release_path = crate_path.join("target/release").join(binary_name);

        if release_path.exists() {
            return Self::ensure_absolute(release_path);
        }
        if debug_path.exists() {
            return Self::ensure_absolute(debug_path);
        }
    }

    // 3. Fall back to main cargo_target
    let cargo_target_dir = PathBuf::from(&config.cargo_target_dir);
    let debug_path = cargo_target_dir.join("debug").join(binary_name);
    let release_path = cargo_target_dir.join("release").join(binary_name);

    // ... existing fallback logic
}
```

### 5. Update Spawner to Use Path Hint

**File:** `core/crates/domain/mcp/src/services/process/spawner.rs`

```rust
pub async fn spawn_server(_manager: &ProcessManager, config: &McpServerConfig) -> Result<u32> {
    let binary_path = BinaryPaths::resolve_binary_with_path(
        &config.binary,
        Some(&config.crate_path),
    ).with_context(|| format!("Failed to find binary for {}", config.name))?;
    // ...
}
```

## Resolution Order (After Fix)

1. **Production** (`binary_dir` set in profile):
   - `/app/bin/{binary}` ✓

2. **Development** (no `binary_dir`, `path` set in YAML):
   - `{path}/target/release/{binary}`
   - `{path}/target/debug/{binary}`

3. **Fallback** (main workspace target):
   - `{cargo_target}/release/{binary}`
   - `{cargo_target}/debug/{binary}`

## YAML Config (No Changes Needed)

The existing YAML format is correct, it just needs to be actually used:

```yaml
mcp_servers:
  systemprompt-admin:
    binary: "systemprompt-admin"        # Binary executable name
    path: "extensions/mcp/admin"        # Crate path (for dev builds)
    port: 5002
    enabled: true
```

## Summary

| File | Change |
|------|--------|
| `deployment.rs` | Make `binary` and `path` required (not Optional) |
| `server.rs` | Add `binary: String` field to `McpServerConfig` |
| `server.rs` | Populate `binary` in `from_manifest_and_deployment` |
| `paths.rs` | Add `resolve_binary_with_path(name, crate_path)` |
| `spawner.rs` | Use `config.binary` and `config.crate_path` for resolution |
