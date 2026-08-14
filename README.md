<div align="center">

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="storage/files/images/logo-white.svg">
  <source media="(prefers-color-scheme: light)" srcset="storage/files/images/logo.svg">
  <img src="storage/files/images/logo.svg" alt="Astound Digital" width="380">
</picture>

# Transformation That Endures.

The Astound Digital branded AI governance platform. One self-hosted binary governs inference, auditing, and every tool call across your AI fleet. Any agent, any model, any provider.

[![Built on systemprompt-core](https://img.shields.io/badge/built%20on-systemprompt--core-2b6cb0?style=flat-square)](https://github.com/systempromptio/systemprompt-core)
[![Rust 1.94+](https://img.shields.io/badge/rust-1.94+-f97316?style=flat-square&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![PostgreSQL 18](https://img.shields.io/badge/postgres-18-336791?style=flat-square&logo=postgresql&logoColor=white)](https://www.postgresql.org/)

[**astounddigital.com**](https://astounddigital.com) · [**Platform documentation**](https://systemprompt.io/documentation/) · [**Guides**](https://systemprompt.io/guides) · [**Discord**](https://discord.gg/wkAbSuPWpr)

</div>

---

## Setup

From nothing to Claude Code running against your own governed gateway.

### 1. Prerequisites

| Requirement | Why | Install |
|---|---|---|
| **Rust 1.94+** | Compiles the binary | [rustup.rs](https://rustup.rs/) |
| **Docker** | Runs PostgreSQL 18 | [docs.docker.com](https://docs.docker.com/get-docker/) |
| **`just`** | Task runner — every command below | [just.systems](https://just.systems/) |
| **`jq`, `yq`** | Used by the setup scripts | `apt install jq yq` / `brew install jq yq` |
| **An AI API key** | Anthropic, OpenAI, or Gemini — one is enough | Provider dashboard |

Ports `8080` (HTTP) and `5432` (Postgres) must be free.

### 2. Clone

```bash
git clone https://github.com/Astound-Digital/systemprompt-astound
cd systemprompt-astound
```

The **server** needs this repository alone: the workspace resolves `systemprompt` from crates.io, and the `[patch.crates-io]` blocks in `Cargo.toml` and `tests/Cargo.toml` are commented out. Uncomment both — `[patch]` is per-workspace — to build the server against a sibling core checkout while a core change is unreleased.

The **client** is different. `bridge/` depends on `systemprompt-bridge` by relative path, and that crate is not published, so building it requires `systemprompt-core` checked out beside this repository. `just bridge-build` clones it for you; nothing else needs it.

### 3. Set up and start

```bash
just setup-local     # builds the binary, writes .systemprompt/profiles/local/,
                     # starts Docker Postgres, runs the publish pipeline
just bridge-build    # builds the Claude Code client; clones systemprompt-core
                     # beside this repo, which the client depends on by path
just start           # governance + agents + MCP + admin on :8080
```

`bridge-build` is part of setup rather than part of connecting: connect codes expire in ten minutes, and a first client build takes longer than that.

`setup-local` prompts for your provider and its key. Non-interactive instead — the first key given becomes the default provider:

```bash
just setup-local <anthropic_key> [openai_key] [gemini_key]
```

Defaults are `8080` and `5432`. A second clone on the same host overrides both: `just setup-local <key> "" "" 8081 5436`.

### 4. Connect Claude Code

Create an account at **http://localhost:8080/admin/login** — registration is passkey-based and gated on the configured email domain; there is no password. The code is bound to the signed-in identity, so this comes first.

Then open **/admin/profile** and copy the connect code:

```bash
just claude <code>
```

A new account has user permissions. Admin-only pages, the systemprompt MCP server, and the admin plugins stay hidden until it is promoted:

```bash
systemprompt admin users role promote <email>
```

Sign out and back in afterwards — the admin scope is minted when the token is issued, so an existing session keeps the old one.

Starts a container, redeems the code, execs `claude`. Host config is untouched: no installer runs, `~/.claude` and `~/.config` are not written.

A code is needed the first time only. The credential it is exchanged for persists, so afterwards the command is just `just claude`.

The container and its home are scoped to the clone and the gateway (`astound-claude-<repo>-<hash>-<gateway>`), so several checkouts pointing at different gateways coexist and never inherit each other's credential. `just claude-reset` signs this clone out; `just claude-reset ALL=1` signs out every clone on the host.

Every request lands in the audit table with user, session, trace, tokens, and cost.

Codes are 32 random bytes, stored hashed, 10-minute TTL, single use. The client redeems one for a durable PAT held on the machine it was issued to. `systemprompt admin bridge issue-code --user-id <email>` issues the same code without a browser.

To configure the host instead of a container: `just connect <code>`. That writes `~/.config/astound/`, a managed block in `~/.profile`, `~/.claude/managed-settings.json`, `~/.local/share/Claude/org-plugins/`, and two systemd user units.

### Verifying from a clean state

Run after any change to the connect path. The failure mode is silent — a machine holding a valid credential skips sign-in and still exits 0.

This section runs on `8081`/`5436` rather than the `8080`/`5432` defaults, so the test instance coexists with a gateway already running. Substitute the defaults if nothing else is up.

```bash
git clone https://github.com/systempromptio/systemprompt-astound fresh && cd fresh
just setup-local <provider-key> "" "" 8081 5436   # ports of its own
just build
just bridge-build
just start
```

The database has no users. Either register at `http://localhost:8081/admin/login` and take the code from the profile page, or stay headless — both write the same `bridge_exchange_codes` row:

```bash
systemprompt admin users create --name you --email you@example.com --if-not-exists
systemprompt admin users role promote you@example.com
systemprompt admin bridge issue-code --user-id you@example.com
```

`claude-reset` is load-bearing — without it a surviving credential carries the test:

```bash
just claude-reset
just claude <code> http://localhost:8081
```

Assert on the output, not the exit code. Pass: `signing in with the supplied code`. Fail: `already signed in — reusing the stored PAT` — sign-in never ran.

`astound-bridge doctor` runs last, one line per check. The hook-token warning is expected: OAuth client provisioning is lazy, on first plugin hook request, not during sync.

### 5. Day-to-day

```bash
just build            # debug build (--release for release)
just preflight        # the CI gate: static → lint → tests → coverage
just publish          # rebuild templates, CSS, JS, assets
systemprompt --help   # discover the CLI
```

## Upgrading core

Two ways to depend on `systemprompt-core`, chosen by the `[patch.crates-io]`
blocks in `Cargo.toml` and `tests/Cargo.toml`:

```bash
# Published release from crates.io — patch blocks commented out.
just core-bump X.Y.Z

# Local sibling checkout, for a core change that is not released yet —
# patch blocks uncommented in BOTH manifests, pins set to the core version.
just build && just prepare && just verify
```

Either way the core version in both manifests must match the version you are
building against; a mismatch drops the patch silently. Core ships its own
migrations, so run the new binary once against your database. Details and the
release procedure: [docs/RELEASING.md](docs/RELEASING.md).

---

<div align="center">

[![systemprompt.io](https://img.shields.io/badge/systemprompt.io-2b6cb0?style=for-the-badge)](https://systemprompt.io) &nbsp; [![Core](https://img.shields.io/badge/systemprompt--core-2b6cb0?style=for-the-badge)](https://github.com/systempromptio/systemprompt-core) &nbsp; [![Documentation](https://img.shields.io/badge/documentation-16a34a?style=for-the-badge)](https://systemprompt.io/documentation/) &nbsp; [![Guides](https://img.shields.io/badge/guides-f97316?style=for-the-badge)](https://systemprompt.io/guides) &nbsp; [![Discord](https://img.shields.io/badge/discord-5865F2?style=for-the-badge&logo=discord&logoColor=white)](https://discord.gg/wkAbSuPWpr)

<sub>Own how your organization uses AI. Every interaction governed and provable.</sub>

</div>
