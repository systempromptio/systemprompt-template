# Plan: Clean Service Discovery & Configuration

## Root Cause Analysis

After investigating the code, here's what's happening:

### The Failure Flow

1. **Reconciliation starts** (`reconciler.rs:66-98`)
   - Publishes `McpEvent::ServiceStartRequested`
   - Waits **500ms**
   - Checks database for `status == "running"`

2. **Event handler starts server** (`lifecycle/startup.rs`)
   - Spawns binary via `process().spawn_server(config)`
   - Waits for health check (up to 15 attempts × 300-1500ms each)
   - **Only registers in database AFTER health check passes**

3. **Problem**: The 500ms wait in reconciliation is **way too short** for:
   - Binary to spawn
   - Health check to pass (30+ attempts taking several seconds)
   - Database registration

### Path Resolution Chain

```
SYSTEM_PATH = /var/www/html/systemprompt-template
         ↓
services_config = $SYSTEM_PATH/crates/services/config/config.yml
         ↓
includes: ../mcp/systemprompt-admin.yml
         ↓
mcp_servers.systemprompt-admin.path = "crates/services/mcp/systemprompt-admin"
         ↓
BinaryPaths::resolve_binary("systemprompt-admin")
         ↓
CARGO_TARGET_DIR = "target" (default, relative!)
         ↓
Looks for: target/debug/systemprompt-admin (relative to CWD)
```

### Key Issues Found

1. **Race condition**: 500ms timeout vs multi-second startup
2. **Relative paths**: `CARGO_TARGET_DIR` defaults to relative "target"
3. **No error propagation**: If startup fails, reconciler just marks as failed
4. **Fragmented config**: `config.yml` → includes → YAML files → path resolution

---

## Problem Statement

The current system has fragmented configuration and service discovery:
- MCP server configs in `crates/services/mcp/*.yml`
- Infrastructure configs in `infrastructure/`
- Core expects services in certain paths
- Binary lookup fails silently
- No clear source of truth for "what services exist and how to run them"

## Current Architecture (Broken)

```
systemprompt-template/
├── crates/services/
│   ├── mcp/
│   │   ├── systemprompt-admin/        # Rust source
│   │   └── systemprompt-admin.yml     # MCP config (name, port, tools)
│   ├── agents/                        # Agent YAML configs
│   ├── config/config.yml              # Services config (references modules)
│   └── ...
├── infrastructure/
│   └── environments/                  # Docker configs
├── config/
│   └── ai.yaml                        # AI provider config
└── target/debug/
    └── systemprompt-admin             # Built binary
```

**Issues:**
1. `just start` can't find MCP server binaries reliably
2. Config files scattered across multiple directories
3. No validation that config matches actual binaries
4. Path resolution is fragile (relative vs absolute)
5. Service discovery in core relies on hardcoded conventions

## Proposed Architecture

### Single Source of Truth: `services.yml`

Create one master config that defines all services:

```yaml
# crates/services/services.yml
version: "1.0"

mcp_servers:
  systemprompt-admin:
    enabled: true
    binary: systemprompt-admin          # Binary name in target/
    port: 5002
    oauth_required: true
    config: mcp/systemprompt-admin.yml  # Relative to this file

agents:
  # Future: agent definitions

api:
  port: 8080
  # API config
```

### Directory Structure

```
systemprompt-template/
├── crates/services/
│   ├── services.yml                   # MASTER CONFIG - single source of truth
│   ├── mcp/
│   │   ├── systemprompt-admin/
│   │   │   ├── Cargo.toml
│   │   │   ├── src/
│   │   │   └── module.yml             # MCP-specific config (tools, resources)
│   │   └── ...
│   ├── agents/
│   │   └── *.yml                      # Agent definitions
│   └── content/
│       └── ...
├── infrastructure/
│   ├── scripts/
│   │   └── service-manager.sh         # Reads services.yml, manages processes
│   └── environments/
│       └── docker/
└── target/debug/                       # Built binaries
```

### Implementation Steps

#### Phase 1: Create Master Config

**1.1 Create `crates/services/services.yml`**

```yaml
version: "1.0"

# Base paths (relative to repo root)
paths:
  binaries: target/debug              # Or target/release for prod
  mcp_configs: crates/services/mcp
  agent_configs: crates/services/agents

# MCP Servers
mcp_servers:
  systemprompt-admin:
    enabled: true
    port: 5002
    oauth_required: true
    description: "Admin dashboard and content management"

# Agents (loaded from agents/*.yml)
agents:
  enabled: true
  auto_discover: true                 # Scan agents/ directory

# API Server
api:
  enabled: true
  port: 8080
```

**1.2 Update core to read this config**

The core `systemprompt` binary should:
1. Look for `services.yml` in `SYSTEMPROMPT_SERVICES_CONFIG` or default path
2. Resolve all paths relative to the config file location
3. Use this config for service discovery instead of scanning directories

#### Phase 2: Update Service Manager

**2.1 Create `infrastructure/scripts/service-manager.sh`**

```bash
#!/bin/bash
# Reads services.yml and manages service lifecycle

SERVICES_CONFIG="${SYSTEMPROMPT_SERVICES_CONFIG:-crates/services/services.yml}"

case "$1" in
  list)
    # Parse services.yml, list all services with status
    ;;
  start)
    # Start specified service or all
    ;;
  stop)
    # Stop specified service or all
    ;;
  status)
    # Show running status of all services
    ;;
esac
```

**2.2 Update `justfile`**

```just
# Service management
services-list:
    ./infrastructure/scripts/service-manager.sh list

services-start service="all":
    ./infrastructure/scripts/service-manager.sh start {{service}}

services-stop service="all":
    ./infrastructure/scripts/service-manager.sh stop {{service}}

services-status:
    ./infrastructure/scripts/service-manager.sh status
```

#### Phase 3: Update Core Integration

**3.1 Modify core service discovery**

In `systemprompt-core`, update the service discovery to:

```rust
// Read services.yml
let config = ServicesConfig::load(config_path)?;

// For each MCP server
for (name, server) in config.mcp_servers {
    if !server.enabled { continue; }

    let binary_path = config.paths.binaries.join(&name);
    if !binary_path.exists() {
        warn!("Binary not found: {}", binary_path.display());
        continue;
    }

    // Start server with correct config
    start_mcp_server(&name, &binary_path, server.port)?;
}
```

**3.2 Environment variable overrides**

```bash
# Override paths for different environments
SYSTEMPROMPT_SERVICES_CONFIG=/path/to/services.yml
SYSTEMPROMPT_BINARY_PATH=/custom/bin/path
```

#### Phase 4: Validation & Health Checks

**4.1 Add validation command**

```bash
just services-validate
```

Checks:
- All enabled services have binaries built
- All ports are unique
- Config files are valid YAML
- Required dependencies exist

**4.2 Add health check endpoint**

Each MCP server exposes `/health` that the orchestrator can poll.

---

## File Changes Summary

| File | Action | Purpose |
|------|--------|---------|
| `crates/services/services.yml` | Create | Master service configuration |
| `infrastructure/scripts/service-manager.sh` | Create | Service lifecycle management |
| `justfile` | Modify | Add service management commands |
| `core/.../service_discovery.rs` | Modify | Read services.yml instead of scanning |
| `crates/services/mcp/*/module.yml` | Keep | MCP-specific config (tools, etc.) |

---

## Migration Path

1. Create `services.yml` with current service definitions
2. Update `just start` to use new service manager
3. Update core to read `services.yml`
4. Remove scattered config lookups
5. Add validation to CI/CD

---

## Immediate Fixes Required (in systemprompt-core)

### Fix 1: Race Condition in Reconciliation

File: `crates/modules/mcp/src/services/orchestrator/reconciliation.rs`

The 500ms wait is insufficient. Options:
1. **Synchronous startup**: Don't use event bus, call `start_server()` directly and wait
2. **Longer timeout**: Increase to 30s with polling
3. **Event-based completion**: Wait for `ServiceStarted` event instead of fixed timeout

Recommended: Option 1 (synchronous startup during reconciliation)

```rust
// Instead of:
event_bus.publish(McpEvent::ServiceStartRequested { ... }).await?;
tokio::time::sleep(Duration::from_millis(500)).await;

// Do:
lifecycle.start_server(&server).await?;
// Registration happens inside start_server after health check
```

### Fix 2: Absolute Path Resolution

File: `crates/modules/core/src/services/shared/paths.rs`

Make `CARGO_TARGET_DIR` absolute when not explicitly set:

```rust
let cargo_target_dir = if config.cargo_target_dir == "target" {
    // Make relative path absolute using SYSTEM_PATH
    PathBuf::from(&config.system_path).join("target")
} else {
    PathBuf::from(&config.cargo_target_dir)
};
```

### Fix 3: Better Error Messages

The build script should show full cargo errors, not truncate them.

---

## Benefits

1. **Single source of truth** - One file defines all services
2. **Clear paths** - No more guessing where binaries/configs are
3. **Environment flexibility** - Easy overrides for dev/prod/docker
4. **Validation** - Can verify config before runtime
5. **Discoverability** - Easy to see what services exist
6. **Consistency** - Same config format for all service types
