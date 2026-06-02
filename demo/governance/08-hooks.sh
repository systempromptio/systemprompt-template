#!/bin/bash
# DEMO: HOOKS — Listing, validation, and live firing
# Shows configured Claude Code hooks, validates them, then proves the
# hook-tracking webhook actually records a fire by POSTing a synthetic
# PostToolUse event and reading the row back from plugin_usage_events.
#
# Cost: Free

set -e

source "$(cd "$(dirname "$0")/.." && pwd)/_common.sh"
load_token

# Runs a CLI command, splits the non-JSON preamble from the JSON body,
# pipes the body through a jq filter to reshape intrinsically-empty
# fields into prose, and indents everything for display.
run_cli_reshape_json() {
  local filter="$1"; shift
  local display="systemprompt $*"
  cmd "$display"
  local output preamble body reshaped
  output=$("$CLI" "$@" --profile "$PROFILE" 2>&1)
  preamble=$(echo "$output" | awk '/^\{/{exit} {print}')
  body=$(echo "$output" | awk '/^\{/{flag=1} flag{print}')
  [[ -n "$preamble" ]] && echo "$preamble" | sed 's/^/  /'
  if [[ -n "$body" ]]; then
    if reshaped=$(echo "$body" | jq -r "$filter" 2>/dev/null); then
      echo "$reshaped" | sed 's/^/  /'
    else
      echo "$body" | sed 's/^/  /'
    fi
  fi
  echo ""
}

header "GOVERNANCE: HOOKS" "Configuration, validation, and live firing"

subheader "STEP 1: List All Hooks"
run_cli_head 40 core hooks list

subheader "STEP 2: Validate Hooks"
run_cli_reshape_json 'if ([.results[] | select((.errors|length)>0)] | length)==0
                      then "✓ All \(.results|length) plugin hook set(s) valid — no errors"
                      else .
                      end' \
  core hooks validate

# ──────────────────────────────────────────────
#  STEP 3: Fire a hook end-to-end
#
#  In production, Claude Code (or any Anthropic-SDK client) sends every
#  hook event it fires locally to /api/public/hooks/track so the gateway
#  can audit, summarise, and bill on it. Here we replay that exact wire
#  format with a synthetic PostToolUse so we can prove the path runs:
#  HTTP 200, then a fresh row in plugin_usage_events keyed on session_id.
# ──────────────────────────────────────────────
subheader "STEP 3: Fire a PostToolUse hook end-to-end"
SID="demo-hooks-fire-$(date +%s)-$$"
info "session_id = $SID"
echo "  $ curl -X POST ${BASE_URL}/api/public/hooks/track  (PostToolUse)"
echo ""

HTTP_CODE=$(curl -s -o /tmp/demo-hook-track.json -w "%{http_code}" \
  -X POST "${BASE_URL}/api/public/hooks/track?plugin_id=enterprise-demo" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d "{
    \"hook_event_name\": \"PostToolUse\",
    \"session_id\": \"$SID\",
    \"cwd\": \"/var/www/html/systemprompt-template\",
    \"transcript_path\": \"/tmp/demo-transcript\",
    \"permission_mode\": \"default\",
    \"tool_name\": \"Read\",
    \"tool_input\": {\"file_path\": \"/etc/hostname\"},
    \"tool_response\": {\"content\": \"localhost\"},
    \"tool_use_id\": \"toolu_demo_$SID\"
  }")

if [[ "$HTTP_CODE" == "200" ]]; then
  pass "Gateway accepted the hook event — HTTP 200"
else
  fail "Expected HTTP 200, got $HTTP_CODE"
  cat /tmp/demo-hook-track.json | sed 's/^/  /'
  exit 1
fi
echo ""

subheader "STEP 4: Audit — confirm the fire was persisted"
echo "  $ infra db query  -- plugin_usage_events WHERE session_id = '$SID'"
echo ""
ROWS=$("$CLI" infra db query \
  "SELECT event_type, tool_name, description, created_at FROM plugin_usage_events WHERE session_id = '$SID'" \
  --profile "$PROFILE" 2>&1 | grep -v "^\[profile")
echo "$ROWS" | sed 's/^/  /'

# Fail loud if the row never landed — this is the actual proof, not the 200.
COUNT=$("$CLI" --json --profile "$PROFILE" infra db query \
  "SELECT COUNT(*)::int AS c FROM plugin_usage_events WHERE session_id = '$SID'" \
  2>/dev/null | sed -n 's/.*"c"[[:space:]]*:[[:space:]]*\([0-9]*\).*/\1/p' | head -1)
if [[ "${COUNT:-0}" -eq 1 ]]; then
  echo ""
  pass "Hook fire recorded — 1 row in plugin_usage_events for session $SID"
else
  echo ""
  fail "Hook fire NOT recorded — expected 1 row, got ${COUNT:-0}"
  exit 1
fi

header "HOOKS DEMO COMPLETE"
