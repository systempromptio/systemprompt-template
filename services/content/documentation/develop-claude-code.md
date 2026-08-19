---
title: "Develop Against a Local Gateway"
description: "Developer setup: clone, build, start the gateway, and connect Claude Code with a one-shot code. Includes the clean-state verification procedure for the connect path."
author: "Astound Digital"
slug: "develop-claude-code"
keywords: "claude code, developer setup, just claude, setup local, clean state, bridge build, local gateway"
kind: "guide"
public: true
tags: ["documentation", "development", "claude-code"]
published_at: "2026-08-19"
updated_at: "2026-08-19"
after_reading_this:
  - "Stand up the gateway from an empty clone"
  - "Connect Claude Code to it with one command"
  - "Verify the connect path from a clean state"
related_playbooks:
  - title: "Connect Claude Code"
    url: "/documentation/connect-claude-code"
  - title: "Expose Your Instance Remotely"
    url: "/documentation/remote-access"
---

# Develop Against a Local Gateway

Advanced developer setup — building the binary from source: an empty machine to a Claude Code session routed through a
gateway you run yourself and recorded in its audit trail. The connect step is
one command; everything before it stands up the server that command talks to.

Connecting to the hosted instance — or any gateway someone else already runs?
You don't need this page: [Connect Claude Code](/documentation/connect-claude-code)
is one curl command with no checkout. And to make a local gateway reachable
from other machines, see
[Expose Your Instance Remotely](/documentation/remote-access).

## Prerequisites

Docker, [`just`](https://github.com/casey/just), a Rust toolchain, and one
provider API key.

The server needs this repository alone — the workspace resolves `systemprompt`
from crates.io. The client does not: `bridge/` depends on `systemprompt-bridge`
by relative path and that crate is unpublished, so building it requires
`systemprompt-core` checked out beside this repository. `just bridge-build`
clones it.

## Setup

```bash
git clone https://github.com/systempromptio/systemprompt-astound.git && cd systemprompt-astound
just setup-local     # profile, Docker Postgres, migrations, publish pipeline
just build
just bridge-build    # Claude Code client; clones systemprompt-core beside this repo
just start           # :8080
```

`bridge-build` belongs in setup rather than in the connect step: codes expire in
ten minutes and a first client build takes longer than that.

`setup-local` prompts for the provider when called with no key. Passing keys is
non-interactive; the first becomes the default provider. Override the ports for
a second clone on one host: `just setup-local <key> "" "" 8081 5436`.

The first build compiles the full dependency graph. Later builds are
incremental.

## Sign in

Registration is web-based and passkey-backed — no password, and no account
exists until it is created:

1. Open `/admin/login`.
2. Register. Self-registration is gated on the configured email domain.
3. Complete the passkey prompt.

The connect code is bound to the signed-in identity, so this precedes it.

A new account has user permissions. Admin-only pages, the systemprompt MCP
server, and the admin plugins remain hidden until it is promoted:

```bash
systemprompt admin users role promote <email>
```

The admin scope is minted at token-issue time, so an existing session keeps the
old one — sign out and back in. Promotion is optional for connecting Claude
Code; it is required to see the full dashboard.

## Connect

With an account signed in, the **Profile** page mints a one-shot code and prints
the command with it filled in:

```bash
just claude <code>
```

Starts a container, redeems the code, execs `claude`. Host config is untouched.

The code is needed on the first run only — the credential it is exchanged for
persists, so later runs are just:

```bash
just claude
```

Container and home are scoped to the clone and its gateway, so several
checkouts pointing at different gateways coexist without inheriting each
other's credential. `just claude-reset` signs this clone out;
`just claude-reset ALL=1` signs out every clone on the host.

If the client is missing, this builds it first — and that build can outlast the
code. Run `just bridge-build` during setup and the connect step is immediate.

### The code

32 random bytes, stored only as a SHA-256 hash, 10-minute TTL, single use. The
client redeems it for a durable PAT that stays on the machine it was issued to
and never passes through the browser. A leaked code is dead within minutes.

Codes are also issuable from the CLI, which is how headless setups work:

```bash
systemprompt admin bridge issue-code --user-id <email-or-uuid>
```

## Host install

For daily work on an owned machine, `just connect <code>` configures the host
rather than a container. It writes:

| Path | Contents |
|------|----------|
| `~/.config/astound/` | Client config, PAT (0600), loopback key |
| `~/.profile` | Managed block setting `ANTHROPIC_BASE_URL` and the auth token |
| `~/.claude/managed-settings.json` | Base URL, `apiKeyHelper`, model discovery |
| `~/.local/share/Claude/org-plugins/` | Organization plugins, skills, MCP servers |
| systemd user units | 30-minute sync timer, loopback inference proxy |

Open a new login shell (or `. ~/.profile`), then run `claude`.

Without a checkout, the installer does the same directly — this is the path
[Connect Claude Code](/documentation/connect-claude-code) documents for end
users:

```bash
curl -fsSL https://your-gateway/files/downloads/install.sh | sh -s -- \
  --download-base https://your-gateway/files/downloads --code <code>
```

The published copy of `install.sh` has its own download base baked in by
`scripts/package-bridge-linux.sh` (`INSTALL_BASE_URL`), so end users omit
`--download-base`; against a local gateway pass it explicitly as above.

Per-platform installer detail — including Windows, macOS, and WSL — is in
[Install the Desktop Bridge](/documentation/bridge-install).

## Verifying from a clean state

Run after any change to the connect path. The failure mode is silent — a machine
holding a valid credential skips sign-in and still exits 0.

Clone to a new directory with no profile and no sibling checkout. The commands
below use `8081`/`5436` rather than the `8080`/`5432` defaults so the test
instance coexists with a running gateway; substitute the defaults if nothing
else is up.

```bash
git clone https://github.com/systempromptio/systemprompt-astound.git fresh && cd fresh
just setup-local <provider-key> "" "" 8081 5436
just build
just bridge-build
just start
```

Re-runs keep the first run's choices: a bare `just setup-local` reads the HTTP
and Postgres ports back from the existing profile and compose file rather than
reverting to `8080`/`5432`.

The database has no users. Either register at `http://localhost:8081/admin/login`
and take the code from the profile page, or stay headless:

```bash
systemprompt admin users create --name you --email you@example.com --if-not-exists
systemprompt admin users role promote you@example.com
systemprompt admin bridge issue-code --user-id you@example.com
```

Scripting the last step? Add the global `--json` flag and take the `code`
field from the artifact instead of scraping the rendered table:

```bash
systemprompt --json admin bridge issue-code --user-id you@example.com \
  | jq -r '.sections[] | select(.heading == "code") | .content'
```

Both write the same `bridge_exchange_codes` row. Registering exercises the
passkey path as well; the CLI path skips it.

`claude-reset` is load-bearing — without it a surviving credential carries the
test:

```bash
just claude-reset
just claude <code> http://localhost:8081
```

Assert on the output, not the exit code.

| Output | Meaning |
|--------|---------|
| `signing in with the supplied code` | Pass — the code was redeemed |
| `already signed in — reusing the stored PAT` | Fail — sign-in never ran |

`astound-bridge doctor` runs last, one line per check. The hook-token warning is
expected: OAuth client provisioning is lazy, on the first plugin hook request,
not during sync.

## Troubleshooting

| Symptom | Cause |
|---------|-------|
| `Client not built yet` | The client is a separate workspace; `just build` does not produce it. `just bridge-build` does, and clones `systemprompt-core` beside this repo because the client depends on it by path. |
| Code rejected | 10-minute TTL, single use. Reload the profile page. |
| Prompted for a code on a repeat run | The stored credential did not validate against this gateway — usually a different gateway from the one that issued it. `just claude-reset`, then connect with a fresh code. |
| Session works, audit trail empty | Not routed through the gateway. Check `ANTHROPIC_BASE_URL` points at the loopback proxy and that `astound-bridge doctor` reports it running. |
| Container cannot reach the gateway | Inside a container `localhost` is the container. `just claude` rewrites it; by hand, use `http://host.docker.internal:8080`. |
| 401 from a container that signed in fine | The gateway the container passes to `login --gateway` is not the gateway the PAT was issued by. The bridge follows only its config file (`login --gateway` writes it); re-run the container with the right `ASTOUND_BRIDGE_GATEWAY_URL` and reconnect. |
