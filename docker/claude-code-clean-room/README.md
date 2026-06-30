# Claude Code clean-room

A throwaway Docker container that proves Claude Code talks to the systemprompt
gateway from a **fresh install** — no prior `~/.claude`, no login — while the
gateway runs separately on the host. See the full writeup in
[`docs/cowork/claude-code-linux.md`](../../docs/cowork/claude-code-linux.md).

## Run

```bash
# 1. Build the Linux bridge binary (once):
(cd ../../../systemprompt-core && just build-bridge)

# 2. Issue a PAT (UI /admin/devices, or POST /api/v1/admin/api-keys).

# 3. Launch the clean room:
SP_BRIDGE_PAT=sp-live-… ./run.sh
```

The entrypoint starts `systemprompt-bridge proxy` headless, wires Claude Code at
it via `~/.claude/settings.json`, runs a raw `/v1/messages` smoke test and a
`claude -p` turn, then installs the **org marketplace** (`install` + `sync`) and
asserts the managed config landed — `marketplace.json`, the plugin bundle with
`.mcp.json` (the `systemprompt` MCP server) and `skills/<id>/SKILL.md`,
`installed_plugins.json`, and `settings.json` `enabledPlugins` — cross-checked
against Claude Code's own `claude plugin list` / `claude mcp list`. Then it drops
to a shell (when run with a TTY).

## What's where

| File | Role |
|---|---|
| `Dockerfile` | `node:22-bookworm-slim` (glibc) + `npm i -g @anthropic-ai/claude-code` + curl/jq + the `channels/` push-test harness, unprivileged `dev` user. |
| `entrypoint.sh` | Starts the proxy, configures Claude Code, runs the smoke test + `claude -p`. |
| `run.sh` | Builds the image and `docker run`s it: mounts the bridge binary read-only, injects the PAT, reaches the host gateway via `host.docker.internal`. |
| `channels/` | Throwaway spike: can a custom **channel** push a server-initiated message that wakes an idle Claude Code session behind our gateway? **Answer: no — gated by a server-delivered feature flag.** See [`channels/README.md`](channels/README.md). |

## Networking

The container reaches the host gateway through `host.docker.internal:8080`. On
Docker Desktop (incl. WSL2) this works even though the gateway binds
`127.0.0.1`. On native Linux Docker, the `--add-host` maps the name to the host,
but the gateway must then bind `0.0.0.0`. (`--network host` does **not** work on
Docker Desktop — it attaches to the Docker VM, not the host loopback.)

The bridge binary and PAT live only inside the disposable container (binary
mounted read-only; PAT passed as an env var, never baked into the image).
