#!/usr/bin/env bash
# Channels-behind-the-gateway viability spike. RUNS INSIDE the clean-room
# container (node:22-bookworm-slim, the bridge proxy reachable on 127.0.0.1:48217).
#
#   ./run-channels-test.sh spike   Stage 1 — single instance: load a custom
#                                   channel, POST a ping to its HTTP listener,
#                                   and prove the model wakes and writes the body
#                                   to /tmp/from-channel.txt.  DECISION GATE.
#   ./run-channels-test.sh demo    Stage 2 — two instances A->B through a relay:
#                                   instance A (claude -p) POSTs the relay, the
#                                   relay routes to instance B's channel, B wakes
#                                   and writes /tmp/B-inbox.txt.
#
# Both stages reuse ONE bridge proxy and the gateway routing claude-* ->
# gemini-2.5-flash. The model B/A run on is a knob (CC_MODEL); if gemini
# tool-calling via the gateway is flaky, set CC_MODEL=claude-opus-4-8 (-> cerebras).
#
# Silent-drop detection: webhook-channel.mjs logs every notification it emits to
# /tmp/webhook-channel.log. If that log shows "notification emitted" but the
# OUTFILE never appears, that is the GATED outcome — Channels do not fire behind
# our gateway (events dropped silently, exactly as the docs warn).
set -uo pipefail

STAGE="${1:-spike}"
CC_MODEL="${CC_MODEL:-claude-sonnet-4-6}"
PROXY_BASE_URL="http://127.0.0.1:48217"
SECRET_FILE="${HOME}/.config/systemprompt/bridge-loopback.key"
CHANNELS_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
READY_TIMEOUT="${READY_TIMEOUT:-180}"   # claude startup + MCP handshake
REACT_TIMEOUT="${REACT_TIMEOUT:-150}"   # model wake + react turn through gateway

say()  { echo "==> $*"; }
info() { echo "    $*"; }

cleanup_pids=()
cleanup() {
  for p in "${cleanup_pids[@]:-}"; do kill "$p" 2>/dev/null || true; done
}
trap cleanup EXIT

tcp_up() { (exec 3<>"/dev/tcp/127.0.0.1/$1") 2>/dev/null && { exec 3>&- 3<&-; return 0; }; return 1; }

# ---------------------------------------------------------------------------
# 0. One shared bridge proxy. Reuse the entrypoint's if it is already up.
# ---------------------------------------------------------------------------
ensure_proxy() {
  if [ -s "${SECRET_FILE}" ] && tcp_up 48217; then
    info "reusing bridge proxy already up on ${PROXY_BASE_URL}"
  else
    say "Starting bridge proxy (headless)"
    systemprompt-bridge proxy >/tmp/bridge-proxy.log 2>&1 &
    cleanup_pids+=("$!")
    for _ in $(seq 1 30); do [ -s "${SECRET_FILE}" ] && tcp_up 48217 && break; sleep 0.5; done
    if [ ! -s "${SECRET_FILE}" ]; then
      echo "!! proxy did not come up:" >&2; cat /tmp/bridge-proxy.log >&2; exit 1
    fi
    info "proxy up on ${PROXY_BASE_URL}"
  fi
  LOOPBACK_SECRET="$(cat "${SECRET_FILE}")"
  export ANTHROPIC_BASE_URL="${PROXY_BASE_URL}"
  export ANTHROPIC_AUTH_TOKEN="${LOOPBACK_SECRET}"
}

# Make sure first-run UI is skipped and channels are not org-blocked.
#
# Crucial for the persistent listener: an INTERACTIVE `claude` shows a one-time
# folder-trust dialog ("Is this a project you trust?") and blocks there forever
# under a pty, never loading MCP servers. `claude -p` skips it. We pre-trust both
# the channels dir (instance B) and HOME (instance A) via projects[*]
# .hasTrustDialogAccepted so the TUI proceeds straight to loading the channel.
prep_claude_settings() {
  mkdir -p "${HOME}/.claude"
  local j="${HOME}/.claude.json"
  local jbase; jbase="$( [ -f "$j" ] && cat "$j" || echo '{}' )"
  echo "$jbase" | jq \
    --arg ch "${CHANNELS_DIR}" --arg home "${HOME}" '
      .hasCompletedOnboarding = true
      | .bypassPermissionsModeAccepted = true
      | .projects = ((.projects // {})
          + { ($ch):   ((.projects[$ch]   // {}) + {hasTrustDialogAccepted:true, hasCompletedProjectOnboarding:true}),
              ($home): ((.projects[$home] // {}) + {hasTrustDialogAccepted:true, hasCompletedProjectOnboarding:true}) })
    ' > "${j}.tmp" && mv "${j}.tmp" "$j"

  local s="${HOME}/.claude/settings.json"
  local base; base="$( [ -f "$s" ] && cat "$s" || echo '{}' )"
  echo "$base" | jq \
    --arg url "${PROXY_BASE_URL}" --arg tok "${LOOPBACK_SECRET}" '
      .env = ((.env // {}) + {ANTHROPIC_BASE_URL:$url, ANTHROPIC_AUTH_TOKEN:$tok})
      | .channelsEnabled = true' > "${s}.tmp" && mv "${s}.tmp" "$s"
  info "settings.json: env wired at proxy, channelsEnabled=true; folders pre-trusted"
}

# Wait for the channel server (spawned BY claude) to log readiness: it has both
# connected its MCP stdio (claude initialized it) and bound its HTTP listener.
wait_channel_ready() {
  local log="$1" port="$2"
  say "Waiting for the channel to load (MCP init + HTTP listener on :$port; up to ${READY_TIMEOUT}s)"
  for _ in $(seq 1 "${READY_TIMEOUT}"); do
    if [ -f "$log" ] && grep -q 'MCP session initialized' "$log" && tcp_up "$port"; then
      info "channel ready (claude connected the MCP channel; HTTP :$port up)"
      return 0
    fi
    sleep 1
  done
  echo "!! channel did not become ready in ${READY_TIMEOUT}s" >&2
  echo "---- channel log ----" >&2; cat "$log" 2>/dev/null >&2 || true
  echo "---- session log ----" >&2; tail -40 /tmp/b-typescript.log 2>/dev/null >&2 || true
  return 1
}

# Persistent, idle, interactive `claude` under a pty with a held-open stdin so it
# stays alive waiting for a channel event. `claude -p` is one-shot and exits — it
# cannot listen. This is the heart of the spike.
start_listener_B() {
  local outfile="$1"
  export CHANNEL_HTTP_PORT=8788
  export CHANNEL_LOG=/tmp/webhook-channel.log
  export CHANNEL_OUTFILE="${outfile}"
  export CHANNEL_SOURCE=webhook
  rm -f "${outfile}" /tmp/webhook-channel.log /tmp/b-typescript.log

  # Hold the fifo's write end open so claude's stdin never EOFs (else it exits).
  rm -f /tmp/b-stdin.fifo; mkfifo /tmp/b-stdin.fifo
  sleep 100000 > /tmp/b-stdin.fifo &
  cleanup_pids+=("$!")

  say "Starting persistent listener (instance B): claude + webhook channel, model ${CC_MODEL}"
  # cd into the channels dir so claude finds .mcp.json (server:webhook).
  ( cd "${CHANNELS_DIR}" && exec script -qfc \
      "claude --dangerously-skip-permissions --dangerously-load-development-channels server:webhook --model ${CC_MODEL}" \
      /tmp/b-typescript.log ) < /tmp/b-stdin.fifo > /dev/null 2>&1 &
  cleanup_pids+=("$!")
  info "listener B launched (pid $!)"
}

push_and_assert() {
  local target_port="$1" outfile="$2" payload="$3" expect="$4"
  say "Pushing event -> 127.0.0.1:${target_port}  payload=${payload}"
  curl -sS -m 10 -X POST "127.0.0.1:${target_port}/" \
    -H 'content-type: application/json' -d "${payload}" | sed 's/^/    relay\/channel resp: /' || true

  say "Waiting up to ${REACT_TIMEOUT}s for the model to wake and write ${outfile}"
  for _ in $(seq 1 "${REACT_TIMEOUT}"); do
    if [ -f "${outfile}" ] && grep -qF "${expect}" "${outfile}"; then
      echo
      info "PASS: ${outfile} contains '${expect}'"
      info "----- ${outfile} -----"; sed 's/^/      /' "${outfile}"
      return 0
    fi
    sleep 1
  done

  echo
  echo "!! FAIL: ${outfile} never contained '${expect}' within ${REACT_TIMEOUT}s" >&2
  # Strip the pty control codes so we can grep the TUI for the gate verdict.
  local ts; ts="$(grep -aoE '[ -~]{3,}' /tmp/b-typescript.log 2>/dev/null || true)"
  local emitted=0; grep -q 'notification emitted' /tmp/webhook-channel.log 2>/dev/null && emitted=1
  if echo "$ts" | grep -qiE 'Channels are not currently available|not available on (Bedrock|third-party)|development-channels ignored'; then
    echo "!! GATED OUTCOME (explicit): Claude Code reports channels unavailable behind our gateway and" >&2
    echo "!! ignored the dev-channel. Root cause is the server-delivered feature gate (tengu_harbor /" >&2
    echo "!! czH()=false) — our gateway does not serve Anthropic's first-party channel gate, so the" >&2
    echo "!! 'claude/channel' capability is stripped at connect time. The channel emitted=${emitted}; the" >&2
    echo "!! model was never given the event. => Channels do NOT work behind our gateway. CLEAN NEGATIVE." >&2
    echo "!! TUI verdict line(s):" >&2
    echo "$ts" | grep -iE 'Channels are not currently available|not available on|development-channels ignored' | sed 's/^/!!   /' | sort -u >&2
  elif [ "$emitted" = 1 ]; then
    echo "!! SILENT-DROP / GATED OUTCOME: the channel emitted the notification but the model never woke and" >&2
    echo "!! the TUI showed no explicit verdict. => Channels do NOT fire behind our gateway (silent drop)." >&2
  else
    echo "!! channel never emitted — check the HTTP push / channel log (NOT a channel-gate result):" >&2
  fi
  echo "---- /tmp/webhook-channel.log ----" >&2; cat /tmp/webhook-channel.log 2>/dev/null >&2 || true
  echo "---- B session (printable tail) ----" >&2; echo "$ts" | tail -25 >&2
  return 1
}

# ---------------------------------------------------------------------------
# Stage 1 — viability spike (single instance). DECISION GATE.
# ---------------------------------------------------------------------------
run_spike() {
  ensure_proxy
  prep_claude_settings
  local nonce; nonce="ping-$(date +%s)-$$"
  start_listener_B /tmp/from-channel.txt
  wait_channel_ready /tmp/webhook-channel.log 8788 || exit 1
  if push_and_assert 8788 /tmp/from-channel.txt "{\"content\":\"${nonce}\"}" "${nonce}"; then
    echo
    say "STAGE 1 PASS — Channels fire behind our gateway."
    info "Now confirm the audited react-turn ON THE HOST:"
    info "  systemprompt infra logs request list --limit 5   # expect a fresh /v1/messages row"
    info "Then re-run with: ./run-channels-test.sh demo"
    exit 0
  fi
  echo
  say "STAGE 1 FAIL — see silent-drop diagnosis above. Stopping (do not proceed to Stage 2)."
  exit 1
}

# ---------------------------------------------------------------------------
# Stage 2 — two-instance A->B demo through the relay (only after Stage 1 passes).
# ---------------------------------------------------------------------------
run_demo() {
  ensure_proxy
  prep_claude_settings

  say "Starting relay (instance-id -> channel-port) on :8790"
  RELAY_PORT=8790 node "${CHANNELS_DIR}/relay.mjs" >/tmp/relay.stdout.log 2>&1 &
  cleanup_pids+=("$!")
  for _ in $(seq 1 20); do tcp_up 8790 && break; sleep 0.5; done
  tcp_up 8790 || { echo "!! relay did not come up" >&2; cat /tmp/relay.stdout.log >&2; exit 1; }
  info "relay up on :8790"

  # Instance B registers B->8788 with the relay on channel startup.
  export RELAY_URL="http://127.0.0.1:8790"
  export INSTANCE_ID="B"
  start_listener_B /tmp/B-inbox.txt
  wait_channel_ready /tmp/webhook-channel.log 8788 || exit 1
  grep -q 'registered with relay' /tmp/webhook-channel.log \
    && info "B registered with the relay" \
    || info "(warning: B registration not yet logged; relay /send may 404)"

  # Instance A originates the send: a separate, channel-less `claude -p` that uses
  # its Bash tool to POST the relay. Run from $HOME (no .mcp.json) so A is plain.
  say "Instance A (claude -p) sends 'hello from A' to the relay -> routed to B"
  ( cd "${HOME}" && timeout 120 claude -p --dangerously-skip-permissions --model "${CC_MODEL}" \
      "Use the Bash tool to run exactly: curl -sS -X POST http://127.0.0.1:8790/send -H 'content-type: application/json' -d '{\"to\":\"B\",\"text\":\"hello from A\"}' — then stop." \
      </dev/null 2>/tmp/a-run.err ) | sed 's/^/    A: /' || { echo "!! instance A failed:" >&2; cat /tmp/a-run.err >&2; }

  say "Waiting up to ${REACT_TIMEOUT}s for B to wake and write /tmp/B-inbox.txt"
  for _ in $(seq 1 "${REACT_TIMEOUT}"); do
    if [ -f /tmp/B-inbox.txt ] && grep -qF 'hello from A' /tmp/B-inbox.txt; then
      echo
      say "STAGE 2 PASS — instance A -> instance B push-listen proven."
      info "----- /tmp/B-inbox.txt -----"; sed 's/^/      /' /tmp/B-inbox.txt
      info "Now confirm BOTH turns are audited ON THE HOST:"
      info "  systemprompt infra logs request list --limit 10   # A's send-turn AND B's react-turn"
      exit 0
    fi
    sleep 1
  done
  echo
  echo "!! STAGE 2 FAIL: /tmp/B-inbox.txt never contained 'hello from A'." >&2
  echo "---- relay log ----" >&2; cat /tmp/relay.log 2>/dev/null >&2 || true
  echo "---- channel log ----" >&2; cat /tmp/webhook-channel.log 2>/dev/null >&2 || true
  echo "---- B session (tail) ----" >&2; tail -60 /tmp/b-typescript.log 2>/dev/null >&2 || true
  exit 1
}

command -v jq >/dev/null || { echo "jq required" >&2; exit 1; }
case "${STAGE}" in
  spike) run_spike ;;
  demo)  run_demo ;;
  *) echo "usage: $0 [spike|demo]" >&2; exit 2 ;;
esac
