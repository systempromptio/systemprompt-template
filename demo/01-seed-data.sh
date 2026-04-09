#!/bin/bash
# SEED DATA — Populate the system with demo data
# Run after 00-preflight.sh to generate data for analytics, logs, and trace demos.
#
# What this does:
#   1. Runs governance allow + deny requests (populates governance_decisions)
#   2. Runs MCP tool calls (populates mcp_tool_executions, user_activity)
#   3. Syncs skills to database (populates agent_skills)
#   4. Ingests content (populates markdown_content)
#
# Cost: Free (no AI calls)

set -e

source "$(cd "$(dirname "$0")" && pwd)/_common.sh"
load_token

header "SEED DATA" "Populating system with demo data for analytics and traces"

# ── Governance decisions ───────────────────────
subheader "STEP 1: Generate governance decisions"

info "Sending ALLOW request (admin agent, clean input)..."
curl -s -X POST "$BASE_URL/api/public/hooks/govern?plugin_id=enterprise-demo" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"hook_event_name":"PreToolUse","tool_name":"mcp__systemprompt__list_agents","agent_id":"developer_agent","session_id":"seed-data","tool_input":{"command":"list"}}' \
  > /dev/null 2>&1
pass "Governance ALLOW recorded"

info "Sending DENY request (user agent, admin tool)..."
curl -s -X POST "$BASE_URL/api/public/hooks/govern?plugin_id=enterprise-demo" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"hook_event_name":"PreToolUse","tool_name":"mcp__systemprompt__list_agents","agent_id":"associate_agent","session_id":"seed-data","tool_input":{"command":"list"}}' \
  > /dev/null 2>&1
pass "Governance DENY recorded"

info "Sending secret detection request..."
curl -s -X POST "$BASE_URL/api/public/hooks/govern?plugin_id=enterprise-demo" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"hook_event_name":"PreToolUse","tool_name":"Read","agent_id":"developer_agent","session_id":"seed-data","tool_input":{"content":"AKIA1234567890ABCDEF"}}' \
  > /dev/null 2>&1
pass "Secret detection DENY recorded"

# Send tracking events
info "Sending PostToolUse tracking events..."
for i in 1 2 3 4 5; do
  curl -s -X POST "$BASE_URL/api/public/hooks/track?plugin_id=enterprise-demo" \
    -H "Authorization: Bearer $TOKEN" \
    -H "Content-Type: application/json" \
    -d "{\"hook_event_name\":\"PostToolUse\",\"tool_name\":\"Read\",\"agent_id\":\"developer_agent\",\"session_id\":\"seed-data-$i\",\"tool_input\":{\"file\":\"main.rs\"},\"tool_result\":\"ok\"}" \
    > /dev/null 2>&1
done
pass "5 PostToolUse events recorded"
echo ""

# ── Skills sync ────────────────────────────────
subheader "STEP 2: Sync skills to database"
cmd "systemprompt core skills sync"
"$CLI" core skills sync --profile "$PROFILE" 2>&1 | tail -5 | sed 's/^/  /'
echo ""

# ── Content ingestion ──────────────────────────
subheader "STEP 3: Ingest content"
cmd "systemprompt infra jobs run blog_content_ingestion"
"$CLI" infra jobs run blog_content_ingestion --profile "$PROFILE" 2>&1 | tail -3 | sed 's/^/  /'
echo ""

# ── Verify ─────────────────────────────────────
subheader "STEP 4: Verify seed data"

DECISIONS=$("$CLI" infra db query "SELECT COUNT(*) as count FROM governance_decisions" --profile "$PROFILE" 2>&1 | grep -oP '"count":\s*\K[0-9]+' || echo "0")
EVENTS=$("$CLI" infra db query "SELECT COUNT(*) as count FROM plugin_usage_events" --profile "$PROFILE" 2>&1 | grep -oP '"count":\s*\K[0-9]+' || echo "0")
SKILLS=$("$CLI" infra db query "SELECT COUNT(*) as count FROM agent_skills" --profile "$PROFILE" 2>&1 | grep -oP '"count":\s*\K[0-9]+' || echo "0")
CONTENT=$("$CLI" infra db query "SELECT COUNT(*) as count FROM markdown_content" --profile "$PROFILE" 2>&1 | grep -oP '"count":\s*\K[0-9]+' || echo "0")

echo "  governance_decisions: $DECISIONS rows"
echo "  plugin_usage_events:  $EVENTS rows"
echo "  agent_skills:         $SKILLS rows"
echo "  markdown_content:     $CONTENT rows"
echo ""

header "SEED DATA COMPLETE" "Analytics, logs, and trace demos now have data to display"
