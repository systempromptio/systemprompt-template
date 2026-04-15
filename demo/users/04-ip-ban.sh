#!/bin/bash
# DEMO: IP BAN MANAGEMENT — VIEW AND MANAGE IP BANS
# Demonstrates ban add/remove with cleanup.
#
# What this does:
#   1. Shows current ban list
#   2. Adds a temporary test ban
#   3. Verifies the ban was added
#   4. Removes the test ban
#   5. Confirms removal
#
# Cost: Free (no AI call)

set -e

source "$(cd "$(dirname "$0")/.." && pwd)/_common.sh"

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

header "USERS: IP BAN MANAGEMENT" "View and manage IP bans"

subheader "STEP 1: Current Ban List"
run_cli_reshape_json 'if (.bans|length)==0 then "✓ No active IP bans (total: \(.total))" else . end' \
  admin users ban list

subheader "STEP 2: Add Test Ban"
info "Adding temporary ban for test IP 192.168.99.99..."
cmd "systemprompt admin users ban add 192.168.99.99 --reason \"demo test\""
"$CLI" admin users ban add 192.168.99.99 --reason "demo test" --profile "$PROFILE" 2>&1 | sed 's/^/  /' || true

subheader "STEP 3: Verify Ban"
run_cli_indented admin users ban list

subheader "STEP 4: Remove Test Ban"
info "Cleaning up test ban..."
cmd "systemprompt admin users ban remove 192.168.99.99"
"$CLI" admin users ban remove 192.168.99.99 --yes --profile "$PROFILE" 2>&1 | sed 's/^/  /' || true

subheader "STEP 5: Confirm Removal"
run_cli_reshape_json 'if (.bans|length)==0 then "✓ Ban removed — no active IP bans (total: \(.total))" else . end' \
  admin users ban list

header "IP BAN DEMO COMPLETE"
