#!/bin/bash
# DEMO: FILE MANAGEMENT — LISTING, CONFIG, STATISTICS
# Read-only file management operations.
#
# What this does:
#   1. Lists managed files
#   2. Shows upload configuration
#   3. Displays file statistics
#
# Cost: Free (no AI call)

set -e

source "$(cd "$(dirname "$0")/.." && pwd)/_common.sh"

# Runs a CLI command, splits the non-JSON preamble from the JSON body,
# pipes the body through a jq filter, and indents everything for display.
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

header "SKILLS: FILE MANAGEMENT" "File listing, config, statistics"

subheader "STEP 1: List Files"
run_cli_head 20 core files list

subheader "STEP 2: Upload Config"
run_cli_indented core files config

subheader "STEP 3: File Statistics"
run_cli_reshape_json '.by_category |= with_entries(select(.value.count > 0))' core files stats

header "FILE MANAGEMENT DEMO COMPLETE"
