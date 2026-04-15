#!/bin/bash
# AGENTS: DISCOVERY — List and inspect configured agents
#
# Cost: Free (read-only CLI commands)
#
# Usage:
#   ./demo/agents/01-list-agents.sh

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

header "AGENTS: DISCOVERY" "List and inspect developer_agent and associate_agent side by side"

subheader "STEP 1: List All Agents"
run_cli_reshape_json '.agents |= map(del(.tags))' admin agents list

subheader "STEP 2: Agent Process Status"
run_cli_indented admin agents status

subheader "STEP 3: Show developer_agent (admin scope)"
run_cli_head 30 admin agents show developer_agent

subheader "STEP 4: Show associate_agent (user scope)"
run_cli_head 30 admin agents show associate_agent

header "AGENT DISCOVERY DEMO COMPLETE"
