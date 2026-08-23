#!/bin/bash
# Send a prompt of your own through the governed gateway, then show the trace.
#
#   examples/pi/trace.sh "explain what this repo does"
#   examples/pi/trace.sh --model gpt-oss-120b "say pong"
#
# The point of the demo is that the prompt is yours: whatever you type here is
# what the dashboard timeline will show, next to the policy decisions it passed
# through and the provider call it produced.
#
# What this does:
#   1. Mints (or reuses) one gateway session, so every event below shares a
#      timeline. The gateway attests x-session-id against a row it issued, so
#      this is a mint, not a label.
#   2. Runs the prompt through Pi with that session pinned
#      (SYSTEMPROMPT_PI_SESSION), so Pi's governance extension reports the same
#      session on the hook spine that the gateway records on the request spine.
#      With Pi absent it falls back to a direct POST /v1/messages — the
#      request spine still lands, the hook spine does not.
#   3. Verifies the trace actually renders at /admin/demo/trace before printing
#      the link, so a URL here means a page with rows behind it.
#
# Portable across macOS and Linux: no grep -P, no head -n -1, no sed -i.
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$HERE/../.." && pwd)"
CRED_DIR="$HOME/.config/systemprompt-pi"

say()  { printf '\033[36m==>\033[0m %s\n' "$*"; }
pass() { printf '\033[32m ok\033[0m %s\n' "$*"; }
warn() { printf '\033[33m  !\033[0m %s\n' "$*"; }
die()  { printf '\033[31mERROR:\033[0m %s\n' "$*" >&2; exit 1; }

MODEL="claude-sonnet-4-6"
PROMPT=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    --model) MODEL="${2:?--model needs a value}"; shift 2 ;;
    --model=*) MODEL="${1#--model=}"; shift ;;
    -h|--help) sed -n '2,23p' "$0"; exit 0 ;;
    *) PROMPT="$1"; shift ;;
  esac
done

if [[ -z "$PROMPT" ]]; then
  if [[ -t 0 ]]; then
    printf '\033[36m??\033[0m Prompt to send [Reply with exactly one word: pong]: ' >&2
    read -r PROMPT || true
  fi
  PROMPT="${PROMPT:-Reply with exactly one word: pong}"
fi

CLI="$PROJECT_DIR/target/debug/systemprompt"
if [[ -x "$PROJECT_DIR/target/release/systemprompt" && "$PROJECT_DIR/target/release/systemprompt" -nt "$CLI" ]]; then
  CLI="$PROJECT_DIR/target/release/systemprompt"
fi
[[ -x "$CLI" ]] || CLI="$(command -v systemprompt || true)"
[[ -n "$CLI" && -x "$CLI" ]] || die "systemprompt CLI not found. Run: just build"

PROFILE="${PROFILE:-local}"
PROFILE_YAML="$PROJECT_DIR/.systemprompt/profiles/$PROFILE/profile.yaml"
BASE_URL="${BASE_URL:-}"
if [[ -z "$BASE_URL" && -f "$PROFILE_YAML" ]]; then
  BASE_URL=$(grep -E '^[[:space:]]*api_server_url:' "$PROFILE_YAML" | head -1 \
    | sed -E 's/.*api_server_url:[[:space:]]*//; s/[[:space:]]*$//; s/^"//; s/"$//')
fi
BASE_URL="${BASE_URL:-http://127.0.0.1:8080}"
# curl gets 127.0.0.1 (localhost can resolve to a [::1] the server is not bound
# to); the browser gets the configured host, because the dashboard session
# cookie is host-only and a 127.0.0.1 link is a different cookie jar from a
# login at localhost.
DASHBOARD_URL="$BASE_URL"
BASE_URL="${BASE_URL/localhost/127.0.0.1}"
BASE_URL="${BASE_URL%/}"

TOKEN_FILE="$CRED_DIR/token"
[[ -s "$TOKEN_FILE" ]] || die "no credential at $TOKEN_FILE — run examples/pi/setup.sh (and new-user.sh) first"
CREDENTIAL=$(cat "$TOKEN_FILE")

# ── 1. One session for the whole run ──
# A JWT already carries an attested session_id claim; a PAT mints a row.
jwt_session_claim() {
  local payload pad
  [[ "$(printf '%s' "$1" | awk -F. '{print NF}')" == "3" ]] || return 0
  payload=$(printf '%s' "$1" | cut -d. -f2)
  pad=$(( (4 - ${#payload} % 4) % 4 ))
  # Why: a bare `&&` test as the group's last command returns 1 when the payload
  # needs no padding, and `pipefail` then fails the whole pipeline for nothing.
  { printf '%s' "$payload"; if [[ $pad -gt 0 ]]; then printf '%.0s=' $(seq 1 $pad); fi; } \
    | tr '_-' '/+' | base64 -d 2>/dev/null \
    | sed -n 's/.*"session_id"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' | head -1
}

say "Resolving a gateway session"
SESSION_ID=$(jwt_session_claim "$CREDENTIAL")
if [[ -z "$SESSION_ID" ]]; then
  SESSION_ID=$(curl -fsS -X POST "$BASE_URL/api/public/gateway/sessions" \
    -H "x-api-key: $CREDENTIAL" | sed -n 's/.*"session_id":"\([^"]*\)".*/\1/p') \
    || die "session mint failed at POST $BASE_URL/api/public/gateway/sessions"
fi
[[ -n "$SESSION_ID" ]] || die "could not resolve a session id"
printf '%s' "$SESSION_ID" > "$CRED_DIR/session"
pass "Session $SESSION_ID"

# ── 2. Send the prompt ──
say "Sending your prompt through $MODEL"
printf '    \033[2m%s\033[0m\n' "$PROMPT"
if command -v pi >/dev/null 2>&1; then
  # Pinning the session makes Pi's hook events and the gateway's request row
  # land on the same timeline instead of two adjacent ones.
  SYSTEMPROMPT_PI_SESSION="$SESSION_ID" \
    pi -p --provider systemprompt --model "$MODEL" "$PROMPT" || \
    warn "Pi exited non-zero — a policy denial does that too; the trace below tells you which"
else
  warn "Pi is not installed — falling back to a direct gateway call (no hook spine)"
  AUTH_HEADER="x-api-key: $CREDENTIAL"
  [[ "$(printf '%s' "$CREDENTIAL" | awk -F. '{print NF}')" == "3" ]] && AUTH_HEADER="Authorization: Bearer $CREDENTIAL"
  BODY=$(PROMPT="$PROMPT" MODEL="$MODEL" python3 -c 'import json,os; print(json.dumps({"model":os.environ["MODEL"],"max_tokens":256,"messages":[{"role":"user","content":os.environ["PROMPT"]}]}))')
  curl -fsS -m 60 -X POST "$BASE_URL/v1/messages" \
    -H "$AUTH_HEADER" -H "x-session-id: $SESSION_ID" \
    -H "anthropic-version: 2023-06-01" -H "content-type: application/json" \
    -d "$BODY" | sed -n 's/.*"text":"\([^"]*\)".*/    gateway replied: \1/p'
fi

# ── 3. Verify the trace renders, then link it ──
TRACE_URL="$DASHBOARD_URL/admin/demo/trace?session=$SESSION_ID"
# The render check is curl, so it uses the 127.0.0.1 form; TRACE_URL above is
# the one printed for a browser.
TRACE_CHECK_URL="$BASE_URL/admin/demo/trace?session=$SESSION_ID"
say "Checking the trace rendered at /admin/demo/trace"
ADMIN_TOKEN=$("$CLI" admin session login --token-only 2>/dev/null \
  | grep -oE '[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+' | head -1 || true)
if [[ -z "$ADMIN_TOKEN" ]]; then
  warn "could not mint an admin token — skipping the render check"
else
  PAGE=$(curl -fsS -H "Authorization: Bearer $ADMIN_TOKEN" "$TRACE_CHECK_URL" || true)
  if printf '%s' "$PAGE" | grep -q "$SESSION_ID"; then
    # One timeline row per `<td class="col-date">` in demo-trace.hbs.
    ROWS=$(printf '%s' "$PAGE" | grep -c '<td class="col-date">' || true)
    pass "Trace page rendered${ROWS:+ ($ROWS timeline rows)}"
  else
    warn "the trace page did not mention $SESSION_ID yet — events can lag a second; reload it"
  fi
fi

echo
say "Evidence — open these as an admin:"
echo "    governed timeline $TRACE_URL"
echo "    this session      $DASHBOARD_URL/admin/entities/sessions/$SESSION_ID"
echo "    all requests      $DASHBOARD_URL/admin/entities/requests"
echo "    policy decisions  $DASHBOARD_URL/admin/governance/decisions"
echo
echo "    Same view from the CLI:"
echo "      $(basename "$CLI") infra logs request list --limit 5"
