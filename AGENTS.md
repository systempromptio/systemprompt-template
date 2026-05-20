# AGENTS.md — systemprompt-template

Clone-and-run evaluation template for [systemprompt.io](https://systemprompt.io). Stand up a complete AI governance binary with demo data in under 10 minutes. MIT licensed.

## What this is

An evaluation template, not a library. Clone, build, run. You get a self-hosted AI governance system with pre-configured agents, MCP servers, skills, scheduled jobs, and a static site. The binary is ~50 MB, needs only PostgreSQL, runs air-gapped.

It wraps [`systemprompt-core`](https://github.com/systempromptio/systemprompt-core) — the Rust crate that does the governance — with production-ready configuration, demo scripts, and a compile-time extension system.

## Try it

```bash
git clone https://github.com/systempromptio/systemprompt-template
cd systemprompt-template
just setup-local <your-anthropic-api-key>   # builds, runs migrations, starts Postgres
just start                                  # serves on http://localhost:8080
```

Then walk the scripts in `demo/` in order — they exercise the full governance pipeline via the CLI.

## Where things live

| Path | Contents |
|------|----------|
| `demo/` | Executable CLI demo scripts organised by domain |
| `services/` | Config-as-code (YAML + Markdown): agents, AI providers, MCP servers, plugins, skills, scheduler |
| `extensions/` | Compile-time Rust extensions (MCP servers + web rendering) |
| `storage/`, `content/` | Static assets and Markdown content |
| `docs/` | Public install + integration guides |
| `deploy/` | Deployment scenarios (airgap, scaled), platform templates (Coolify, Railway, Render), Nix flake |
| `src/main.rs` | Thin binary entry point — delegates to `core/` |

Full developer guidance for working in this repo lives in [`CLAUDE.md`](./CLAUDE.md).

## Governance pipeline

Every tool call passes through four synchronous stages before execution: **scope check → secret scan (35+ patterns) → blocklist → rate limit**. Every decision is logged as a structured JSON event with a `trace_id` linking identity → agent → tool call → result → cost.

## Key facts

- Single Rust binary, ~50 MB; PostgreSQL is the only runtime dependency.
- Provider-agnostic: Anthropic, OpenAI, Gemini, local models.
- MCP-native — governance is the transport, not a proxy.
- Identity: `users.roles` is a free-text `TEXT[]` array; `admin` is just a role string. System-originated work (scheduled jobs, hooks, MCP) runs under an explicit `owner:` declared per resource in `services/scheduler/config.yaml`. The owner is a real admin user — there is no separate "system" user. Every governance audit row carries `(user_id, actor_kind, actor_id)`: see `services/content/documentation/authentication.md`.
- Compile-time extension model via the [`inventory`](https://docs.rs/inventory) crate.
- Configuration is YAML under `services/`, not a database UI.
- Air-gap capable.

## Licensing

This template is **MIT licensed**. Fork it, modify it, ship it. The underlying `systemprompt-core` crate is BSL-1.1 (source-available, free for evaluation, commercial licence for production).

## Links

- Template: https://github.com/systempromptio/systemprompt-template
- Core: https://github.com/systempromptio/systemprompt-core
- Docs: https://systemprompt.io/documentation
- Agent-readable summary: https://systemprompt.io/llms.txt
- Feedback: open a `feedback`-labelled issue, or email `hello@systemprompt.io`
