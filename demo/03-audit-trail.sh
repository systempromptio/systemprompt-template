#!/bin/bash
# DEMO 3: AUDIT TRAIL — Governance Decisions
# Queries governance_decisions recorded by demos 01 + 02
#
# What this does:
#   1. Queries the governance_decisions table for the 5 most recent decisions
#      - Each decision has: decision (ALLOW/DENY), tool_name, agent_id,
#        agent_scope, policy matched, and reason
#   2. Shows a contrast table explaining the expected results:
#      - Demo 01 (developer_agent): admin scope, ALLOW via default_allow
#      - Demo 02 (associate_agent): user scope, DENY via scope_restriction
#   3. Shows cost breakdown by agent via `analytics costs breakdown --by agent`
#
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
echo "  DEMO 3: AUDIT TRAIL — Governance Decisions"
echo "=========================================="
echo ""

echo "Querying governance_decisions for recent decisions..."
echo ""
"$CLI" infra db query "SELECT decision, tool_name, agent_id, agent_scope, policy, reason FROM governance_decisions ORDER BY created_at DESC LIMIT 5"

echo ""
echo "=========================================="
echo "  EXPECTED RESULTS"
echo "=========================================="
echo ""
echo "You should see the ALLOW from Demo 01 (developer_agent)"
echo "and the DENY from Demo 02 (associate_agent)."
echo ""
echo "  Demo | Agent             | Scope | Decision | Policy"
echo "  -----|-------------------|-------|----------|------------------"
echo "  01   | developer_agent   | admin | ALLOW    | default_allow"
echo "  02   | associate_agent   | user  | DENY     | scope_restriction"
echo ""

# Cost breakdown
echo "=========================================="
echo "  COST BREAKDOWN BY AGENT"
echo "=========================================="
echo ""
"$CLI" analytics costs breakdown --by agent

echo ""
echo "=========================================="
echo "  Demo complete."
echo "=========================================="
echo ""
echo "Now run: ./demo/04-governance-happy.sh"
