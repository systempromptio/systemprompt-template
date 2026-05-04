# Cowork Desktop app

The Cowork Desktop app is the user-facing companion to a running [systemprompt-gateway](../install/ghcr.md). It is a single native binary — `systemprompt-bridge` — that does three jobs:

1. **Credential helper.** Emits Anthropic's `inferenceCredentialHelper` JSON envelope (`{ token, ttl, headers }`) on stdout when invoked by Claude Desktop.
2. **Local inference proxy.** Loopback HTTP listener on `127.0.0.1:48217` that swaps the long-lived loopback secret for a short-lived JWT before forwarding to the gateway. The JWT never leaves the host.
3. **Sync agent.** Pulls the user's signed plugin / skill / agent / MCP-allowlist manifest from the gateway into the synthetic plugin directory `<org-plugins>/systemprompt-managed/`.

On macOS and Windows it now ships with a native GUI (winit + wry); on Linux the same binary still runs headless from the shell.

> **Naming note.** The crate and binary were renamed from `cowork` to `bridge` in v0.7.0. Public-facing branding stays "Cowork" — the macOS bundle is `Systemprompt Cowork.app` and Claude Desktop's host integration is still labelled "Cowork". Internally the executable is `systemprompt-bridge` (Linux/Windows) or `systemprompt-cowork` inside the .app (macOS, renamed by the bundler for parity with prior installs). Existing release tags (`cowork-v*`) continue to publish artifacts.

---

## What you get when you launch the GUI

| Surface | What it does |
|---|---|
| **Setup wizard** | First-run flow. Two steps: paste a gateway URL + PAT (or pick session/mTLS), then enable each agent integration you want managed. |
| **Agents tab** | One card per registered host (Claude Desktop, Codex CLI, …). Per-agent **Enable / Disable** toggle persisted to `~/.config/systemprompt/agents.json`. Disabled hosts are not probed and reject `host.probe`, `host.profile.generate`, `host.profile.install`, `agent.uninstall`, `agent.openConfig` with `Conflict`. Hosts default to **disabled** on a fresh install — nothing is silently probed. |
| **Marketplace tab** | Browse plugins, skills, agents, and MCP servers materialised by sync. Install/uninstall is implicit: cloud sync (`bridge sync`) is the install mechanism — there are no per-item install buttons. |
| **Settings panel** | Theme, gateway URL, PAT rotation, Re-run setup wizard, signature pubkey pinning. |
| **Status pill** | Overall health badge: gateway reachable, proxy listening, sync state, manifest version. |
| **Activity drawer** | Live log of proxy requests, sync events, host probes, and structured errors. **Help & Support** card has two actions:<br>· **Open log folder** — reveals the rotating logs dir in the OS file manager.<br>· **Export diagnostic bundle** — zips logs + activity JSONL + crash dumps + redacted config + `diagnostics.txt` to the Desktop, then reveals it. |
| **Native menu bar** | macOS app-wide menus and Windows window-attached menus via `muda`. All entries are translated through Fluent (`web/i18n/<locale>/bridge.ftl`); drop a locale file to retranslate the entire UI. |
| **Cancel buttons** | The sync pill exposes a cancel button when a sync, login, gateway probe, or logout is in flight. Wired to `bridge.cancel(scope)`. |

The control plane between the WebView and the Rust side is IPC (custom protocol `sp://app/`); the legacy HTTP control server has been removed except for a single-instance focus endpoint on a loopback TCP listener. Static assets are served via `sp://app/` so no CSRF token round-trip is needed.

---

## Three roles, one binary

```
┌─────────────────────────────────────────────────────────────┐
│                  systemprompt-bridge                        │
├─────────────────────────────────────────────────────────────┤
│  bridge gui                  → native settings window       │
│  bridge                      → emit credential JSON         │
│  bridge login <sp-live-…>    → store PAT (or use the GUI)   │
│  bridge sync [--watch]       → pull signed manifest         │
│  bridge install [--apply]    → register on launchd / task   │
│                                  scheduler / systemd-user;  │
│                                  pin manifest pubkey        │
│  bridge validate             → end-to-end self-check        │
│  bridge status / whoami      → health + identity            │
│  bridge logout / clean       → revoke + wipe state          │
│  bridge uninstall [--purge]  → reverse install              │
└─────────────────────────────────────────────────────────────┘
```

Exit codes: `0` success · `2` emit error · `3` whoami error · `5` no credential source succeeded · `8` pubkey not pinned · `10` transient failure on preferred provider.

---

## Security posture

- **Out-of-band manifest pubkey pinning.** `bridge install --apply --pubkey <base64>` writes the pin to `HKCU\SOFTWARE\Policies\Claude` (Windows) or `com.anthropic.claudefordesktop` Managed Preferences (macOS). `bridge sync` is fail-closed without a pin unless `--allow-tofu`. The pin is read via in-process FFI through the same managed-policy code path on every platform.
- **Distinct JWT audience.** Tokens minted with `audience: Cowork` cannot call generic gateway API endpoints if stolen.
- **Replay protection.** Manifests carry a signed `not_before`; sync rejects `manifest_version ≤ last_applied` or `not_before` outside ±5 min skew.
- **RFC 8785 (JCS) canonical JSON** for signature input.
- **Loopback proxy** uses a single-source-of-truth secret (proxy startup is the sole minter; profile generation is read-only). Constant-time comparison; non-loopback `Host` headers are rejected.
- **mTLS-preferred chain.** Transient mTLS failures exit `10` rather than silently downgrading to PAT.
- **Live config reload.** Gateway URL and PAT changes propagate via `arc-swap` — no restart required.
- **Redacted diagnostics.** The exported bundle redacts values under `secret`, `credential`, `auth`, `pat`, `token`, `password`, `key`, `pubkey`, `session` keys before zipping.

---

## What's new in v0.7.0

- Native Desktop GUI on macOS + Windows (Linux still headless).
- Per-agent enable/disable, persisted across runs.
- Codex CLI host integration (probe, config, install) alongside Claude Desktop.
- Synthetic plugin layout — managed skills/agents/MCP allowlist materialise into a single `<org-plugins>/systemprompt-managed/` directory instead of separate `.systemprompt-bridge/` fragments.
- Diagnostics surface — daily log rotation (7 files, `tracing-appender`), persistent activity JSONL, panic dumps, version subcommand with embedded git SHA + build timestamp.
- Fluent i18n — every visible string translated through `web/i18n/<locale>/bridge.ftl`. Drop a new locale file and the entire UI switches.
- Cancel-in-flight for sync, login, gateway probe, and logout.
- Native menu bar on both platforms (`muda`).
- Vanilla Web Components frontend — Lit removed; every component extends a 110-line `SpElement` base with reactive setters and `data-action` event delegation.
- IPC types are ts-rs–generated — Rust is the source of truth.

Full release notes: see [`bin/bridge/CHANGELOG.md`](https://github.com/systempromptio/systemprompt-core/blob/main/bin/bridge/CHANGELOG.md) in `systemprompt-core`.

---

## Install

| Platform | Doc |
|---|---|
| macOS — `.app` bundle | [install-macos.md](install-macos.md) |
| Windows — `.exe` | [install-windows.md](install-windows.md) |
| Windows — Scoop | [scoop.md](scoop.md) |
| Linux | Build from source (`just build-bridge` in the [systemprompt-core](https://github.com/systempromptio/systemprompt-core) checkout). |

After install, see [device-auth.md](device-auth.md) for credential modes and [windows-minimax-demo.md](windows-minimax-demo.md) for an end-to-end runbook.

---

## Links

- [systemprompt.io](https://systemprompt.io/?utm_source=cowork-desktop&utm_medium=docs)
- [systemprompt-core on GitHub](https://github.com/systempromptio/systemprompt-core)
- [Bridge changelog](https://github.com/systempromptio/systemprompt-core/blob/main/bin/bridge/CHANGELOG.md)
- [LICENSE](../../LICENSE)
