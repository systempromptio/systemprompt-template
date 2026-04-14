<!-- Thanks for the PR! Keep this short — the checklist is the point. -->

## What & why

<!-- One paragraph. Link any issue. -->

## Checklist

- [ ] `just build` is green
- [ ] `just clippy` is green (pedantic, deny-all — no new `#[allow]` suppressions)
- [ ] Touched files respect `CLAUDE.md`:
  - No edits to `core/` (read-only submodule)
  - Rust code lives in `extensions/`
  - Config-only changes live in `services/` (YAML/Markdown)
  - CSS lives in `storage/files/css/` and is registered in `extensions/web/src/extension.rs`
- [ ] If templates/CSS/JS/assets changed: ran `just publish`
- [ ] Demo scripts still run against a fresh `just setup-local`
- [ ] README updated if user-facing behaviour changed

## Governance pillar

<!-- Governance Pipeline · Secrets Management · Audit & SIEM · MCP Governance · Skill Marketplace · Unified Control Plane · Tooling · Docs -->
