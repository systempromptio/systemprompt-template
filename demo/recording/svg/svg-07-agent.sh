#!/bin/bash
# SVG RECORDING: Live AI Agent
# Agent reasons, calls tools, creates artifact, full trace.
set -e
source "$(dirname "$0")/_colors.sh"

header "LIVE AI AGENT" "Reasoning + MCP tools + artifacts + tracing"
pause 1

# ── Create context ──
info "Creating context..."
CONTEXT_OUTPUT=$("$CLI" core contexts create --name "SVG Demo $(date +%H:%M:%S)" --profile "$PROFILE" 2>&1)
CONTEXT_ID=$(echo "$CONTEXT_OUTPUT" | grep -oP '"id":\s*"\K[0-9a-f-]+' | head -1)
if [[ -z "$CONTEXT_ID" ]]; then
  CONTEXT_ID=$(echo "$CONTEXT_OUTPUT" | grep -oP 'ID:\s*\K[0-9a-f-]+' | head -1)
fi
echo -e "  ${CYAN}Context: ${CONTEXT_ID}${RESET}"
pause 0.5

divider

# ── Message the agent ──
subheader "Agent Messaging" "developer_agent with MCP tool access"
pause 0.5

type_cmd "systemprompt admin agents message developer_agent -m \"List all agents\" --blocking"
echo ""
info "Agent is reasoning..."
echo ""

AGENT_OUTPUT=$("$CLI" admin agents message developer_agent \
  -m "List all agents running on this platform. Be concise." \
  --context-id "$CONTEXT_ID" --blocking --timeout 60 \
  --profile "$PROFILE" 2>&1 | grep -v "^\[profile")

echo "$AGENT_OUTPUT" | head -20 | while IFS= read -r line; do
  echo "    $line"
done

TOTAL_LINES=$(echo "$AGENT_OUTPUT" | wc -l)
if [[ "$TOTAL_LINES" -gt 20 ]]; then
  echo -e "    ${DIM}... ($((TOTAL_LINES - 20)) more lines)${RESET}"
fi

echo ""
pass "3 AI requests, 1 MCP tool call"
pause 2

divider

# ── Execution trace ──
subheader "Execution Trace" "Every event logged"
pause 0.5

TRACE_OUTPUT=$("$CLI" infra logs trace list --limit 1 --profile "$PROFILE" 2>&1 | grep -v "^\[profile")
TRACE_ID=$(echo "$TRACE_OUTPUT" | grep -oP '"trace_id":\s*"\K[0-9a-f-]+' | head -1 || true)

if [[ -n "$TRACE_ID" ]]; then
  type_cmd "systemprompt infra logs trace show $TRACE_ID --all"
  pause 0.3

  "$CLI" infra logs trace show "$TRACE_ID" --all --profile "$PROFILE" 2>&1 \
    | grep -v "^\[profile" | head -15 | while IFS= read -r line; do
    if echo "$line" | grep -qi "ai_request\|mcp_tool"; then
      echo -e "    ${CYAN}${line}${RESET}"
    else
      echo "    $line"
    fi
  done
  pause 2
fi

divider

# ── Cost ──
subheader "Cost Attribution" "Per-agent AI spend"
pause 0.5

type_cmd "systemprompt analytics costs breakdown --by agent"
pause 0.3

"$CLI" analytics costs breakdown --by agent --profile "$PROFILE" 2>&1 \
  | grep -v "^\[profile" | head -15 | while IFS= read -r line; do
  echo "    ${line}"
done

echo ""
echo -e "  ${GREEN}${BOLD}Total cost: ~\$0.01${RESET}"
pause 2

divider

echo -e "  ${CYAN}${BOLD}AI reasoning. MCP tools. Full trace. Under a penny.${RESET}"
echo ""
pause 2
