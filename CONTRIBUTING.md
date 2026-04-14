# Contributing

Thanks for kicking the tires on the **systemprompt.io local evaluation template**.

This repo is the public, MIT-licensed on-ramp to systemprompt.io. Contributions are
welcome for anything that makes the evaluation experience sharper:

- `extensions/` — your Rust code (web, MCP servers, jobs)
- `services/` — YAML/Markdown config (agents, skills, plugins, AI providers, web)
- `demo/` — the runnable evaluation scripts
- `docs/`, `README.md` — documentation and positioning

Feature work on the underlying `systemprompt-core` library happens in the closed-source
core repo. Bugs you can reproduce against the evaluation template are in scope here —
we'll triage and, when appropriate, fix upstream and bump the facade version.

## Ground rules

These come straight from `CLAUDE.md`; please read it once before you start.

1. **`core/` is read-only.** It's a git submodule. Never edit it.
2. **Rust code lives in `extensions/`.** No `.rs` files outside it.
3. **Config lives in `services/`.** YAML/Markdown only.
4. **CSS lives in `storage/files/css/`** and must be registered in
   `extensions/web/src/extension.rs` under `required_assets()`. Never put CSS in
   `extensions/*/assets/css/`.
5. **`just clippy` is pedantic, deny-all.** Fix the root cause; don't add
   `#[allow(...)]` suppressions.
6. **Never use `git stash` or `SQLX_OFFLINE`.** Always work against the live Postgres
   brought up by `just db-up` / `just setup-local`.

## Local setup

```bash
# 1. Clone with submodules
git clone --recurse-submodules https://github.com/systempromptio/systemprompt-template
cd systemprompt-template

# 2. Build
just build

# 3. Seed profile + Postgres
just setup-local <anthropic_key> <openai_key> <gemini_key>

# 4. Run the stack
just start
```

Then use the CLI to find your way around: `systemprompt --help`.

## Before you open a PR

- [ ] `just build` is green
- [ ] `just clippy` is green (no new `#[allow]`)
- [ ] If you touched templates / CSS / JS / static assets: `just publish`
- [ ] Relevant demo scripts still run against a fresh `just setup-local`
- [ ] `README.md` updated if user-facing behaviour changed
- [ ] Commits follow the existing style (`feat:`, `fix:`, `refactor:`, `chore:`, `docs:`)

The PR template will remind you of most of this.

## Reporting bugs

Open a [bug report](.github/ISSUE_TEMPLATE/bug_report.yml). Please include:

- The exact `just` / `systemprompt` commands that reproduce it
- Output of `systemprompt infra logs view --level error --since 1h`
- OS, `rustc --version`, PostgreSQL version, and the template commit SHA

## Security disclosures

Please **do not** file public issues for security vulnerabilities. Email
[security@systemprompt.io](mailto:security@systemprompt.io) instead.

## Commercial / production support

This repo is evaluation-only. For production licensing, deployment support, or
enterprise SLAs: [book a meeting](https://systemprompt.io/contact) or email
[ed@systemprompt.io](mailto:ed@systemprompt.io).
