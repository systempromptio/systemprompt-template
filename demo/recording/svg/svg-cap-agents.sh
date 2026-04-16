#!/bin/bash
# SVG RECORDING: Closed-Loop Agents
# Agents that reason, execute, and observe their own metrics.
set -e
source "$(dirname "$0")/_colors.sh"

header "CLOSED-LOOP AGENTS" "Reasoning + MCP tools + self-observation"
pause 1

# ── Registry ──
subheader "Agent Registry" "A2A discovery — every agent addressable"
pause 0.3

type_cmd "systemprompt admin agents registry"
pause 0.3
"$CLI" admin agents registry --profile "$PROFILE" 2>&1 \
  | grep -v "^\[profile\|^\[2m" \
  | head -15 \
  | while IFS= read -r l; do echo "    $l"; done
echo ""; pause 1

divider

# ── Message ──
info "Creating context..."
CONTEXT_ID=$("$CLI" core contexts create --name "SVG Agents $(date +%H:%M:%S)" --profile "$PROFILE" 2>&1 \
  | grep -oP '"id":\s*"\K[^"]+' | head -1)
echo -e "  ${CYAN}Context: $CONTEXT_ID${R}"

subheader "Agent Messaging" "developer_agent with MCP tool access"
pause 0.3

type_cmd "systemprompt admin agents message developer_agent -m \"List all agents\" --blocking"
pause 0.3

info "Agent is reasoning..."
RESPONSE=$("$CLI" admin agents message developer_agent \
  -m "List all agents running on this platform. Be concise - one sentence per agent." \
  --blocking --timeout 60 --profile "$PROFILE" 2>&1 | grep -v "^\[profile\|^\[2m")

echo "$RESPONSE" | head -20 | while IFS= read -r l; do
  [[ -n "$l" ]] && echo "    $l"
done
echo ""
pass "AI reasoning + MCP tool execution"
pause 1.5

divider

# ── Self-observation ──
subheader "Self-Observation" "agents query their own performance metrics"
pause 0.3

type_cmd "systemprompt analytics agents show developer_agent"
pause 0.3
"$CLI" analytics agents show developer_agent --profile "$PROFILE" 2>&1 \
  | grep -v "^\[profile\|^\[2m" \
  | head -15 \
  | while IFS= read -r l; do printf "    ${CYAN}%s${R}\n" "$l"; done
echo ""; pause 1.2

divider

# ── Cost ──
subheader "Cost Attribution" "per-agent AI spend"
pause 0.3

type_cmd "systemprompt analytics costs breakdown --by agent"
pause 0.3
"$CLI" analytics costs breakdown --by agent --profile "$PROFILE" 2>&1 \
  | grep -v "^\[profile\|^\[2m" \
  | head -10 \
  | while IFS= read -r l; do echo "    $l"; done
echo ""

echo -e "  ${CYAN}${BOLD}AI reasoning. MCP tools. Self-observation. Full trace.${R}"
echo ""
pause 2
