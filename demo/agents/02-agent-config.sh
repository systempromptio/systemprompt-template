#!/bin/bash
# AGENTS: CONFIGURATION — Validation, tools, status
#
# Cost: Free (read-only CLI commands)
#
# Usage:
#   ./demo/agents/02-agent-config.sh

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

header "AGENTS: CONFIGURATION" "Validation and MCP tool inventory for both demo agents"

subheader "STEP 1: Agent Process Status"
run_cli_indented admin agents status

subheader "STEP 2: Validate developer_agent"
run_cli_reshape_json 'if .valid and (.errors|length)==0 and (.warnings|length)==0 then "✓ Valid — \(.items_checked) item checked, no errors or warnings" else . end' \
  admin agents validate developer_agent

subheader "STEP 3: MCP Tools Available to developer_agent"
run_cli_head 30 admin agents tools developer_agent

subheader "STEP 4: Validate associate_agent"
run_cli_reshape_json 'if .valid and (.errors|length)==0 and (.warnings|length)==0 then "✓ Valid — \(.items_checked) item checked, no errors or warnings" else . end' \
  admin agents validate associate_agent

subheader "STEP 5: MCP Tools Available to associate_agent"
run_cli_head 30 admin agents tools associate_agent

header "AGENT CONFIG DEMO COMPLETE"
