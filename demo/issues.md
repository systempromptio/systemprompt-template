# Demo Issues

All 42 category demos pass (exit 0). This file documents remaining CLI-level issues that affect demo quality.

---

## CLI Bugs

### 1. `plugins mcp list` — AppPaths not initialized
```
Error: Failed to list MCP servers
Caused by: AppPaths not initialized - call AppPaths::init() first
```
**Workaround:** Use `plugins mcp status` instead.

### 2. `plugins mcp logs <server>` — Log files not found
```
Error: Log file not found for service 'systemprompt'. Available: []
```
**Workaround:** None — MCP logs are invisible. Use `infra logs view` for general log access.

### 3. `admin agents registry` — JSON parse error
```
Error: Failed to parse registry response
Caused by: missing field `url` at line 1 column 13251
```
**Workaround:** Use `admin agents status` for running agent info. Registry demo has graceful fallback.

### 4. `plugins mcp validate` — Requires `--service` in non-interactive mode
```
Error: --service is required in non-interactive mode
```
**Workaround:** Not used in demos. Use `plugins mcp status` for validation info.

---

## Friction

### 5. ~500ms startup per CLI command
Every invocation validates cloud credentials via HTTPS roundtrip to `api.systemprompt.io`, loads secrets, and discovers 12 extensions. Demos with many steps accumulate noticeable latency.

**Suggestion:** Cache credential validation for local profiles, or add `--skip-cloud-check`.

### 6. `[profile: local ...]` prefix on every command
Visual noise in demo output. Suppressed by RUST_LOG=warn but still appears on some commands.

**Suggestion:** Only show in `--verbose` mode.
