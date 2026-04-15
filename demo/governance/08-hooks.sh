#!/bin/bash
# DEMO: HOOKS — Hook listing and validation
# Shows all configured hooks across plugins.
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

header "GOVERNANCE: HOOKS" "Hook configuration and validation"

subheader "STEP 1: List All Hooks"
run_cli_head 40 core hooks list

subheader "STEP 2: Validate Hooks"
run_cli_reshape_json 'if ([.results[] | select((.errors|length)>0)] | length)==0
                      then "✓ All \(.results|length) plugin hook set(s) valid — no errors"
                      else .
                      end' \
  core hooks validate

header "HOOKS DEMO COMPLETE"
