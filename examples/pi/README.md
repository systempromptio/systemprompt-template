# Pi + Enterprise Demo gateway

Drive [Pi](https://pi.dev) — a minimal, extensible coding-agent CLI — through
the Enterprise Demo governance gateway. Every prompt Pi sends is authenticated,
policy-checked, audited, and cost-tracked, and a single Pi provider entry gives
you models from Anthropic, OpenAI, and Google behind one governed endpoint.

## How it works

Pi supports custom providers via `~/.pi/agent/models.json`. The gateway exposes
an Anthropic-compatible `POST /v1/messages`, so one provider entry with
`"api": "anthropic-messages"` covers everything:

- **Auth** — `Authorization: Bearer <jwt>` minted by
  `systemprompt admin session login --token-only`.
- **Session binding** — the gateway requires an `x-session-id` header naming a
  session it issued: for a JWT caller, the token's own `session_id` claim; for a
  personal access token, a session minted at `POST /api/public/gateway/sessions`.
  A session id the server never issued is rejected with a 401, so every request
  is tied to an auditable session rather than a label the client picked.
- **Model routing** — the gateway routes by model id glob (`claude-*` →
  Anthropic, `gpt-*` → OpenAI, `gemini-*` → Google). Pi's `/model` picker
  (Ctrl+L) switches between them mid-session; the gateway does the rest.

Secrets never live in Pi config: `models.json` reads the token from
`~/.config/systemprompt-pi/` at request time via Pi's `!command` value
resolution. The session header is set by the governance extension, which
resolves one session per Pi conversation — the JWT's claim, or a freshly minted
one for a PAT.

## Setup

With the server running (`just start`):

```bash
examples/pi/setup.sh
```

The script installs Pi (npm), mints a gateway token, installs the provider
config and the branded `systemprompt` theme, and smoke-tests `/v1/messages`.
It derives the gateway URL from your active profile
(`.systemprompt/profiles/<flavour>/profile.yaml`), so non-default ports just work.

Then:

```bash
pi
```

Pick a model with `/model` — Claude Sonnet 4.6, Claude Opus 4.8, GPT-5 mini,
and Gemini 2.5 Flash are pre-configured.

Tokens expire eventually; re-auth without reinstalling anything:

```bash
examples/pi/setup.sh --refresh
```

## Watch the governance spine

Every Pi turn lands in the same audit tables as every other client:

```bash
systemprompt infra logs request list --limit 10     # one row per /v1/messages hit
systemprompt infra logs audit <request-id> --full   # identity → policy → cost chain
systemprompt analytics costs                        # rollups
```

Try asking the same question on two different models and compare the provider,
token, and cost columns.

The gateway only serves models in its registry (`allow_unlisted_models:
false`). Add a fifth model id that is not registered to `models.json` and the
gateway denies it with a 403 before any upstream call — governance working as
intended.

## Tool-level governance

The gateway governs what Pi *sends*. The governance extension governs what Pi
is *about to do*, using the same `POST /api/public/hooks/govern` endpoint and
the same four policies (scope check, secret scan, blocklist, rate limit) that
Claude Code's `PreToolUse` hook hits.

Pi has no hook by that name. It has the equivalents:

| Pi event | Claude Code analogue | What the extension does |
|---|---|---|
| `input` | `UserPromptSubmit` | governs the raw prompt; a denial stops the turn, so a pasted credential never reaches a provider |
| `tool_call` | `PreToolUse` | governs the tool call; a denial blocks execution and the reason goes back to the model |
| `tool_result` | `PostToolUse` | records the fire to `plugin_usage_events` |

`setup.sh` installs it to `~/.pi/agent/extensions/systemprompt-governance.ts`
(reload inside a running Pi with `/reload`). A tool call whose verdict cannot be
obtained is blocked, not allowed — the `FAIL_OPEN` constant at the top of the
file is where that choice lives.

The extension registers two tools that exist only to be blocked —
`mcp__systemprompt__list_agents` (denied by `scope_check`) and `delete_records`
(denied by `tool_blocklist`) — because no stock Pi tool name matches either
policy's patterns.

**Two credentials, two endpoints.** `/v1/messages` accepts the `sp-live-…`
personal access token, alongside an `x-session-id` minted for it at
`POST /api/public/gateway/sessions` (`new-user.sh` does this for its smoke test;
the extension does it once per Pi conversation). `/hooks/govern` validates a JWT
and rejects a PAT, so `new-user.sh` also mints a plugin token to
`~/.config/systemprompt-pi/hook-token`. Which user you select changes what the
demo proves: admins are exempt from `scope_check` and `tool_blocklist`, so
those two cases come back allowed, and `09-pi-agent.sh` asserts that exemption
rather than narrating a denial that did not happen. Select a non-admin to see
the denial path. `secret_scan` has no exemption and denies either way.

See it end to end:

```bash
./demo/governance/09-pi-agent.sh          # free, deterministic, self-asserting
./demo/governance/09-pi-agent.sh --live   # drives the real pi binary
```

Then open **`/admin/demo/trace`** (sidebar: Governance → Demo Trace) and pick
your session — one Pi conversation is one session row — to see it as one
timeline: prompt gate, tool gate, model calls, and tool fires in
the order they happened. A blocked prompt has no model call after it.

## Branding

`themes/systemprompt.json` is a Pi theme built from the systemprompt.io
palette (deep navy background, cyan/blue accents). `setup.sh` installs and
activates it; switch themes any time with `/settings`. Pi themes control
colors only — the Pi name and logo are not themeable.

## The full demo loop

[`WALKTHROUGH.md`](WALKTHROUGH.md) walks the complete story: pick the user Pi
acts as (`new-user.sh`), drive Pi as them, then govern them live from the
dashboard's **Model Selection** page (`/admin/models`) — per-user model
enable/disable with immediate 403s, plus full usage and audit visibility.
Run `routes.sh` once (plus a server restart) to split the demo models into
individually governable gateway routes.

## Files

| File | Purpose |
|------|---------|
| `models.json` | Provider template installed into `~/.pi/agent/models.json` |
| `extensions/governance.ts` | Prompt gate + tool gate; installed into `~/.pi/agent/extensions/` |
| `themes/systemprompt.json` | Branded Pi theme |
| `setup.sh` | Idempotent installer + token mint + smoke test (admin) |
| `new-user.sh` | Pick the user Pi acts as from the database (or create one) + issue their gateway API key, governance token, and `user.json` identity file |
| `trace.sh` | Send your own prompt through the gateway, then link its verified dashboard trace |
| `routes.sh` | Add per-model gateway routes for per-user governance |
| `RUNNING-AS-A-USER.md` | Setup manual: acting as a named database user, in the TUI as well as the scripts |
| `WALKTHROUGH.md` | The end-to-end demo script |
