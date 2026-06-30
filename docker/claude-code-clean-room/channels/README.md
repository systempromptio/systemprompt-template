# Can one Claude Code instance "listen" for pushes via our gateway? — NO (gated)

Throwaway harness that answers one question: when Claude Code is pointed at our
gateway (`ANTHROPIC_BASE_URL` + Bearer JWT to `systemprompt-bridge proxy`), can a
custom **channel** push a server-initiated message into an idle session and wake
the model?

**Result: NO.** Channels are gated behind a server-delivered feature flag that
our gateway does not serve. Claude Code says so explicitly — it is *not* a silent
drop in this version.

## The decisive run

```
==> Pushing event -> 127.0.0.1:8788  payload={"content":"ping-…"}
    relay/channel resp: {"ok":true}          # our HTTP listener emitted the MCP notification
!! FAIL: /tmp/from-channel.txt never written within 45s
!! TUI verdict line(s):
!!    --dangerously-load-development-channels ignored (server:webhook)
   (and on screen:) "Channels are not currently available"
```

The channel server connected over MCP stdio, advertised
`capabilities.experimental['claude/channel'] = {}`, opened its HTTP listener, and
on the POST emitted `notifications/claude/channel {content}`. Claude Code accepted
none of it: it stripped the channel capability at connect time, ignored the
dev-load flag, and never delivered the event to the model.

## Why — exact mechanism (from the v2.1.169 binary)

Channels are gated by **two** checks, plus a capability strip:

```js
if (Mq() !== "firstParty") return { skip, reason: "channels are not available on third-party providers" };
if (!czH())                return { skip, reason: "channels feature is not currently available" };
// at the MCP connection layer:
if (caps["claude/channel"] && (!czH() || !approvedSource)) delete caps["claude/channel"];
```

- `Mq()` (provider mode) returns a third-party value **only** when
  `CLAUDE_CODE_USE_BEDROCK/VERTEX/FOUNDRY/ANTHROPIC_AWS/MANTLE` is set. Setting
  just `ANTHROPIC_BASE_URL` (our case) leaves it `"firstParty"` — so we **pass**
  the 3P-provider gate. The plan's hypothesis (blocked as a 3P provider) is *not*
  what happens.
- `czH()` is `j$("tengu_harbor", false)` — a **server-delivered Statsig/feature
  gate**. Channels are research-preview; the gate ships from Anthropic's
  first-party infrastructure during init. Our gateway does not serve that gate
  payload, so it defaults to `false` → `claude/channel` is deleted at connect →
  the dev-load flag is "ignored" → on-screen "Channels are not currently
  available".

So the block is the **feature gate (`czH()` / `tengu_harbor`)**, not the provider
type. There is no client setting that flips it (`channelsEnabled: true` only
governs the *org policy* layer, which is a separate, later check); the gate value
comes from the upstream, and the upstream is our gateway.

## Implication

The server-initiated **push/listen** path (Channels) is **blocked** behind our
gateway. Therefore:

- **Stage 2 (two-instance A→B via the relay) is not reachable** on the Channels
  path — `relay.mjs` is left in place but unused, since B can never be woken.
- The follow-up of swapping the relay for the real gateway bus
  (`event_outbox` + LISTEN/NOTIFY + `webhook/broadcast`) is moot for *push*.
- **Fallback (deferred by decision):** an MCP **poll** mailbox — a normal MCP
  tool the model *calls* to drain queued messages. That works behind the gateway
  because it needs no first-party gate, but it is poll, not listen.

## Files

| File | Role |
|---|---|
| `webhook-channel.mjs` | Custom channel: MCP stdio server (`@modelcontextprotocol/sdk`) declaring `experimental['claude/channel']`, HTTP listener on `:8788`, emits `notifications/claude/channel {content, meta}` on POST. Logs every emit to `/tmp/webhook-channel.log` for silent-drop detection. |
| `relay.mjs` | (Stage 2, unused) instance-id → channel-port shim: `POST /register {id,port}`, `POST /send {to,text}`. Stand-in for the real gateway bus. |
| `.mcp.json` | Registers `webhook` so `--dangerously-load-development-channels server:webhook` can load it. |
| `run-channels-test.sh` | Orchestrates the spike inside the container. |
| `package.json` | Pins `@modelcontextprotocol/sdk` (npm-installed in the image). |

## Re-running

```bash
# from the repo root, with the gateway up on :8080 and a PAT:
PAT=$(cat /tmp/channels-spike.pat)   # or mint one (see docs/cowork/claude-code-linux.md §2)
BIN=../../../systemprompt-core/bin/bridge/target/release/systemprompt-bridge
docker build -t systemprompt/claude-code-clean-room:latest docker/claude-code-clean-room
docker run --rm --add-host=host.docker.internal:host-gateway \
  -v "$BIN:/usr/local/bin/systemprompt-bridge:ro" \
  -e SP_BRIDGE_PAT="$PAT" -e SP_BRIDGE_GATEWAY_URL=http://host.docker.internal:8080 \
  -e REACT_TIMEOUT=45 \
  --entrypoint /home/dev/channels/run-channels-test.sh \
  systemprompt/claude-code-clean-room:latest spike
```

Expect `STAGE 1 FAIL` with the explicit gate verdict above. A `spike` PASS would
require Anthropic to serve the channel feature gate to our gateway's sessions.

## Notes / gotchas proven here

- **Persistent listener:** `claude -p` is one-shot and cannot listen; B must be a
  long-lived interactive `claude` under a pty (`script -qfc … < fifo`) with stdin
  held open so it idles.
- **Folder-trust dialog:** an interactive `claude` blocks on a one-time "Is this a
  project you trust?" dialog and never loads MCP servers until confirmed (`-p`
  skips it). The harness pre-trusts the folder via
  `~/.claude.json.projects[dir].hasTrustDialogAccepted = true`. Without this the
  session hangs with empty logs (looks like a channel failure but is not).
- The channel MCP/HTTP contract itself is correct and was verified in isolation
  (capability advertised, `oninitialized` fires, `server.notification()` with the
  custom method delivers `{content, meta}` intact). The only thing missing is the
  upstream feature gate.
