# Demo Issues

All 44 category demos pass (exit 0). This file documents remaining CLI-level issues that affect demo quality.

---

## Open

### web validate — content config wrapper bug in core (4 residual warnings)
`systemprompt web validate` reports 4 warnings (down from 14 after the
2026-05-21 template-side cleanup), still rc=1. Root cause is a core-side
shape mismatch, not template config:

- **3× template warnings** ("Template 'X' references unknown content type
  'Y'"). `services/web/templates/templates.yaml` was rewritten to bind by
  source name (`blog`, `documentation`) per the validator's contract at
  `crates/entry/cli/src/commands/web/validate/template_validation.rs:67`.
  But the validator parses `services/content/config.yaml` directly as
  `ContentConfigRaw` (no `content:` wrapper), while the file ships with a
  top-level `content:` key that the runtime aggregator
  (`ServicesConfig.content: ContentConfig`) requires. Result: validator
  sees an empty `content_sources` map and warns on every binding. The same
  bug makes `systemprompt web sitemap show` return 0 routes locally. Fix
  belongs in core — either read content config through the aggregator or
  tolerate the `content:` wrapper in direct readers.

- **1× missing-directory warning** for `services/web/assets/`. Creating
  the directory triggers a worse failure: `asset_validation.rs` then
  literal-checks for `services/web/assets/logos/logo.{svg,png,webp}` and
  `favicon.ico`, which the template intentionally serves from
  `storage/files/images/` per CLAUDE.md Rule #4 (assets live in
  `storage/files/`, not `services/web/assets/`). The validator's
  hardcoded layout doesn't match the template's actual asset home; fix
  also belongs in core (consult `web/config.yaml`'s configured asset
  paths instead of a hardcoded `logos/` subdir).

`demo/web/02-sitemap-validate.sh` continues to tolerate rc=1 via the
shared `run_cli_reshape_json` helper.

---

## Resolved in systemprompt-core 0.11

Verified 2026-05-21 against template clone running `systemprompt 0.11.0`. Each command exits 0 with sensible output.

### 1. `plugins mcp list` — AppPaths not initialized ✅ resolved
Previously errored with `AppPaths not initialized`; now returns the configured MCP servers as JSON.

### 2. `plugins mcp logs <server>` — Log files not found ✅ resolved
Previously errored with `Log file not found`; now reads from the database log source and returns recent MCP service log lines.

### 3. `admin agents registry` — JSON parse error ✅ resolved
Previously errored with `missing field 'url'`; now returns both `developer_agent` and `associate_agent` with full `url`, `version`, `status`, `streaming`, and `skills_count` fields.

### 4. `plugins mcp validate` — Requires `--service` in non-interactive mode ✅ resolved
Previously refused without `--service`; now auto-detects the `systemprompt` server and emits a validation summary (`total/valid/invalid/healthy/unhealthy`).

---

## Friction

### 5. ~500ms startup per CLI command
Every invocation validates cloud credentials via HTTPS roundtrip to `api.systemprompt.io`, loads secrets, and discovers 12 extensions. Demos with many steps accumulate noticeable latency.

**Suggestion:** Cache credential validation for local profiles, or add `--skip-cloud-check`.

### 6. `[profile: local ...]` prefix on every command
Visual noise in demo output. Suppressed by RUST_LOG=warn but still appears on some commands.

**Suggestion:** Only show in `--verbose` mode.
