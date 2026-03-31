#!/bin/bash
# DEMO 3: AUDIT TRAIL
# Shows the two most recent traces with full detail
#
# What this does:
#   1. Queries trace list via `infra logs trace list --limit 2`
#      - Each trace has: trace_id (UUID), agent name, status, event count
#   2. Extracts the two most recent trace IDs using grep
#   3. Shows full detail for each trace via `infra logs trace show <id> --all`
#      - Trace detail includes: AI requests, MCP tool calls, execution steps,
#        skills loaded, timing per step, and cost tracking
#   4. Shows cost breakdown by agent via `analytics costs breakdown --by agent`
#
# Expected output (after running demos 01 + 02):
#   Trace 1 (associate_agent): ~4 events, 1 AI request, 0 MCP calls
#   Trace 2 (developer_agent): ~11 events, 3 AI requests, 1 MCP call
#
# IDs shown: trace_id, agent name, request IDs within each trace
# Cost: Free (read-only queries)

# Resolve the CLI binary
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
CLI="$PROJECT_DIR/target/debug/systemprompt"
if [[ -x "$PROJECT_DIR/target/release/systemprompt" && "$PROJECT_DIR/target/release/systemprompt" -nt "$CLI" ]]; then
  CLI="$PROJECT_DIR/target/release/systemprompt"
fi
if [[ ! -x "$CLI" ]]; then
  echo "ERROR: CLI binary not found. Run: cargo build" >&2
  exit 1
fi

echo ""
echo "=========================================="
echo "  DEMO 3: AUDIT TRAIL"
echo "=========================================="
echo ""

# Get the two most recent trace IDs
TRACES_JSON=$("$CLI" infra logs trace list --limit 2 2>&1)

# Extract trace IDs (skip "system" traces)
TRACE_1=$(echo "$TRACES_JSON" | grep -oP '"trace_id":\s*"\K[0-9a-f-]+' | head -1)
TRACE_2=$(echo "$TRACES_JSON" | grep -oP '"trace_id":\s*"\K[0-9a-f-]+' | head -2 | tail -1)
AGENT_1=$(echo "$TRACES_JSON" | grep -oP '"agent":\s*"\K[^"]+' | head -1)
AGENT_2=$(echo "$TRACES_JSON" | grep -oP '"agent":\s*"\K[^"]+' | head -2 | tail -1)

echo "Found 2 recent traces:"
echo "  1. $AGENT_1 ($TRACE_1)"
echo "  2. $AGENT_2 ($TRACE_2)"

# Show full trace for the most recent (should be associate_agent / refused path)
if [[ -n "$TRACE_1" ]]; then
  echo ""
  echo "=========================================="
  echo "  TRACE: $AGENT_1 (most recent)"
  echo "=========================================="
  echo ""
  "$CLI" infra logs trace show "$TRACE_1" --all
fi

# Show full trace for the second (should be developer_agent / happy path)
if [[ -n "$TRACE_2" ]]; then
  echo ""
  echo "=========================================="
  echo "  TRACE: $AGENT_2"
  echo "=========================================="
  echo ""
  "$CLI" infra logs trace show "$TRACE_2" --all
fi

# Cost breakdown
echo ""
echo "=========================================="
echo "  COST BREAKDOWN BY AGENT"
echo "=========================================="
echo ""
"$CLI" analytics costs breakdown --by agent

echo ""
echo "=========================================="
echo "  Demo complete."
echo "=========================================="
