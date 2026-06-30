# Claude Code → Gateway on Linux (and any headless host)

Route the **Claude Code CLI** through this gateway instead of logging in to
claude.ai — the same governed `/v1/messages` path Cowork uses. Every request
lands one row in the audit spine (`user_id`, `trace_id`, model, tokens, cost).

This is the Linux / server story. macOS and Windows users get the desktop GUI
(see [install-macos.md](install-macos.md) / [install-windows.md](install-windows.md));
on Linux the GUI is not built, so the bridge runs its local inference proxy
**headlessly** via `systemprompt-bridge proxy`.

---

## Why a proxy, not just env vars

Claude Code can point at any Anthropic-shaped endpoint with two env vars:

| Variable | Value |
|---|---|
| `ANTHROPIC_BASE_URL` | the endpoint (no `/v1` suffix — Claude Code appends it) |
| `ANTHROPIC_AUTH_TOKEN` | sent as `Authorization: Bearer …` |

But you **cannot** point Claude Code straight at the gateway with a bridge JWT.
The gateway's `/v1/messages` requires an `x-session-id` header and validates it
against the JWT's `session_id` claim (`"X-Session-ID does not match
authenticated session"`). The bridge mints a fresh session per token, and
Claude Code's `apiKeyHelper` can only supply a token — not the matching header —
so a static header goes stale on every refresh.

The **bridge loopback proxy** solves this exactly as it does for Cowork: it owns
both the JWT and the matching identity headers, injects them on every forwarded
request, and refreshes the JWT in the background. Claude Code points at the
proxy; the proxy talks to the gateway.

```
claude  ──Bearer <loopback-secret>──▶  systemprompt-bridge proxy  ──JWT + x-session-id──▶  gateway /v1/messages
        (ANTHROPIC_BASE_URL=127.0.0.1:48217)        (loopback only)                        (governed, audited)
```

---

## 1. Build / install the bridge on Linux

No Linux release artifact is published yet — build from the core checkout:

```bash
cd ../systemprompt-core
just build-bridge                  # → bin/bridge/target/release/systemprompt-bridge (glibc x86_64)
install -m 0755 bin/bridge/target/release/systemprompt-bridge ~/.local/bin/   # ensure ~/.local/bin is on PATH
systemprompt-bridge --version
```

Config lives at `~/.config/systemprompt/systemprompt-bridge.toml`; the loopback
secret at `~/.config/systemprompt/bridge-loopback.key`. The GUI subcommand is a
no-op on Linux — use `proxy` instead.

## 2. Get a PAT

A PAT is the credential the proxy exchanges for JWTs. Two ways:

- **UI:** `http://<gateway>/admin/login` → `/admin/devices` → **Issue PAT** → copy `sp-live-…`.
- **API (scriptable):** with an admin session token,

  ```bash
  TOKEN=$(systemprompt admin session login --token-only)
  curl -sS -X POST http://localhost:8080/api/v1/admin/api-keys \
    -H "Authorization: Bearer $TOKEN" -H "content-type: application/json" \
    -d '{"name":"claude-code"}' | jq -r .secret
  ```

## 3. Configure and start the proxy

```bash
systemprompt-bridge login sp-live-… --gateway http://localhost:8080   # or export SP_BRIDGE_PAT / SP_BRIDGE_GATEWAY_URL
systemprompt-bridge proxy
```

`proxy` prints the base URL and loopback token to use:

```
systemprompt-bridge proxy listening on http://127.0.0.1:48217

  export ANTHROPIC_BASE_URL=http://127.0.0.1:48217
  export ANTHROPIC_AUTH_TOKEN=<loopback-secret>
```

The proxy binds loopback only and rejects any other host or a wrong secret (403).

## 4. Point Claude Code at it

Either export the two vars in your shell, or persist them in
`~/.claude/settings.json`:

```json
{
  "env": {
    "ANTHROPIC_BASE_URL": "http://127.0.0.1:48217",
    "ANTHROPIC_AUTH_TOKEN": "<loopback-secret>"
  }
}
```

Then run Claude Code normally — no login:

```bash
claude -p "say hello"
```

## 5. Verify on the gateway

```bash
systemprompt infra logs request list --limit 5
systemprompt infra logs audit <request-id> --full
```

Each Claude Code turn is one governed `/v1/messages` row — identity, model,
tokens, cost, trace_id.

---

## Org marketplace (skills + MCP servers)

The same `systemprompt-bridge` that proxies inference also installs the org
marketplace — the GUI does this on Win/Mac; on Linux it's `sync`:

```bash
PUBKEY=$(curl -fsS http://localhost:8080/v1/bridge/pubkey | jq -r .pubkey)
systemprompt-bridge install --gateway http://localhost:8080 --pubkey "$PUBKEY"
systemprompt-bridge sync          # fetches the signed manifest, verifies ed25519, writes config
```

`sync` pulls the signed manifest (`GET /v1/bridge/manifest`), verifies it against
the pinned pubkey, and materializes it into Claude Code's plugin tree —
identical to the managed config the desktop app writes:

```
~/.claude/plugins/
  marketplaces/org-provisioned/.claude-plugin/marketplace.json
  marketplaces/org-provisioned/plugins/systemprompt-managed/
    .claude-plugin/plugin.json
    .claude-plugin/.mcp.json          # managed MCP servers (e.g. "systemprompt")
    skills/<skill-id>/SKILL.md         # org skills
  known_marketplaces.json
  installed_plugins.json               # systemprompt-managed@org-provisioned
~/.claude/settings.json                # enabledPlugins + extraKnownMarketplaces (merged, env block preserved)
~/.local/share/Claude/org-plugins/systemprompt-managed/   # synthetic-plugin mirror
```

Verify with Claude Code's own view: `claude plugin list` shows
`systemprompt-managed@org-provisioned` **enabled**, and `claude mcp list` lists
the `systemprompt` server. Managed MCP URLs are rewritten to route through the
loopback proxy (`http://127.0.0.1:48217/mcp/<slug>`). Note: an OAuth-protected
MCP server won't *connect* in a headless run (its first use needs an interactive
browser OAuth, same constraint as any headless host) — but the config is
installed and recognized exactly as on Win/Mac.

## Clean-room test (Docker)

`docker/claude-code-clean-room/` is a throwaway container that proves the
from-scratch install path while the gateway runs separately on the host:
`node:22-bookworm-slim` with a fresh `npm install -g @anthropic-ai/claude-code`,
no prior `~/.claude`, the bridge binary mounted read-only, the PAT injected at
runtime. It starts the proxy, wires Claude Code at it, runs a smoke test + a
`claude -p` turn, then `sync`s the marketplace and asserts the MCP server + skill
config landed.

```bash
# build the bridge first: (cd ../systemprompt-core && just build-bridge)
SP_BRIDGE_PAT=sp-live-… ./docker/claude-code-clean-room/run.sh
```

The container reaches the host gateway via `host.docker.internal:8080` (works on
Docker Desktop / WSL2 even though the gateway binds `127.0.0.1`). `--network host`
does **not** work on Docker Desktop — it attaches to the Docker VM, not the host
loopback. On native Linux Docker, the `--add-host` maps the name to the host but
the gateway must then bind `0.0.0.0`.

---

## Reference

- Bridge auth endpoints: `POST /v1/auth/bridge/{pat,session,mtls}`, `GET /v1/auth/bridge/capabilities`
- Marketplace: `GET /v1/bridge/manifest` (signed), `GET /v1/bridge/pubkey` (ed25519 verify key)
- Identity probe: `GET /v1/bridge/whoami`
- Inference: `POST /v1/messages` (requires `x-session-id` matching the JWT)
- Auth modes / config: [device-auth.md](device-auth.md)
- Desktop equivalent: [desktop-app.md](desktop-app.md)
