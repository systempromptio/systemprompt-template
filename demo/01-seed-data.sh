#!/bin/bash
# SEED DATA — Populate the Enterprise Demo with baseline state for every demo.
# Run after 00-preflight.sh. Idempotent enough to re-run; reseeding will
# re-post governance events and create additional contexts.
#
# What this populates:
#   1. Skills synced to database (agent_skills)
#   2. Contexts (demo-review, incident-response, onboarding)
#   3. Files uploaded from demo/fixtures/ (uploads, user_activity)
#   4. Governance decisions — allow, scope_restriction, secret_injection,
#      tool_blocklist — across 5 sessions, both agents, 6 tool names
#      (governance_decisions)
#   5. PostToolUse tracking events across sessions (plugin_usage_events)
#   6. Synthetic page view traffic (user_activity: 100 rows across
#      varied paths / referers / countries / user agents)
#   7. Content ingestion (markdown_content)
#
# Optional:
#   SEED_AGENT_RUN=1 ./demo/01-seed-data.sh
#     Additionally sends a real A2A message to developer_agent which produces
#     a trace + artifact + cost row. Costs a few cents of Gemini tokens.
#
# Cost: Free by default. ~$0.01 with SEED_AGENT_RUN=1.

set -e

source "$(cd "$(dirname "$0")" && pwd)/_common.sh"
load_token

header "SEED DATA" "Populating Enterprise Demo with baseline state"

# ── STEP 1: Skills sync ────────────────────────
subheader "STEP 1: Sync skills to database"
cmd "systemprompt core skills sync"
"$CLI" core skills sync --profile "$PROFILE" 2>&1 | tail -5 | sed 's/^/  /' || true
echo ""

# ── STEP 2: Contexts ───────────────────────────
subheader "STEP 2: Create contexts"
for ctx in demo-review incident-response onboarding; do
  info "Creating context: $ctx"
  "$CLI" core contexts create --name "$ctx" --profile "$PROFILE" > /dev/null 2>&1 || true
done
pass "3 contexts ensured"
echo ""

# ── STEP 3: Files ──────────────────────────────
subheader "STEP 3: Upload sample files"
CONTEXT_ID=$("$CLI" --json core contexts list --profile "$PROFILE" 2>/dev/null \
  | grep -oE '"id":\s*"[^"]+"' | head -1 | sed -E 's/.*"([^"]+)"$/\1/')
if [[ -z "$CONTEXT_ID" ]]; then
  warn "No context id found; using 'demo-review' literal"
  CONTEXT_ID="demo-review"
fi
for f in "$DEMO_ROOT/fixtures/"*.{md,txt,png,wav}; do
  [[ -f "$f" ]] || continue
  info "Uploading: $(basename "$f")"
  "$CLI" core files upload "$f" --context "$CONTEXT_ID" --profile "$PROFILE" > /dev/null 2>&1 || true
done
pass "Fixture files uploaded (documents, images, audio)"
echo ""

# ── STEP 4: Governance decisions ───────────────
subheader "STEP 4: Generate governance decisions"

# Truncate first so the LIMIT 5 window in governance-03 only sees fresh
# rows written against the current agent-scope resolver, not stale
# "unknown"-scope rows from earlier seed runs.
"$CLI" infra db query "TRUNCATE governance_decisions" --profile "$PROFILE" > /dev/null 2>&1 || true

gov() {
  local session="$1" agent="$2" tool="$3" input="$4"
  curl -s -X POST "$BASE_URL/api/public/hooks/govern?plugin_id=enterprise-demo" \
    -H "Authorization: Bearer $TOKEN" \
    -H "Content-Type: application/json" \
    -d "{\"hook_event_name\":\"PreToolUse\",\"tool_name\":\"$tool\",\"agent_id\":\"$agent\",\"session_id\":\"$session\",\"tool_input\":$input}" \
    > /dev/null 2>&1 || true
}

track() {
  local session="$1" agent="$2" tool="$3" input="$4" latency="$5"
  curl -s -X POST "$BASE_URL/api/public/hooks/track?plugin_id=enterprise-demo" \
    -H "Authorization: Bearer $TOKEN" \
    -H "Content-Type: application/json" \
    -d "{\"hook_event_name\":\"PostToolUse\",\"tool_name\":\"$tool\",\"agent_id\":\"$agent\",\"session_id\":\"$session\",\"tool_input\":$input,\"tool_result\":\"ok\",\"duration_ms\":$latency}" \
    > /dev/null 2>&1 || true
}

# developer_agent sessions — admin scope, should mostly allow
for i in 1 2 3 4 5; do
  s="session-dev-0$i"
  gov "$s" developer_agent Read '{"file":"src/main.rs"}'
  gov "$s" developer_agent Bash '{"command":"cargo check"}'
  gov "$s" developer_agent mcp__systemprompt__list_agents '{"filter":"all"}'
  gov "$s" developer_agent mcp__skill-manager__list_skills '{}'
  track "$s" developer_agent Read '{"file":"src/main.rs"}' $((50 + i * 30))
  track "$s" developer_agent Bash '{"command":"cargo check"}' $((1200 + i * 250))
done
pass "20 developer_agent governance events (allows + tracking)"

# associate_agent sessions — user scope hitting admin tools (should deny)
for i in 1 2 3 4 5; do
  s="session-assoc-0$i"
  gov "$s" associate_agent Read '{"file":"README.md"}'
  gov "$s" associate_agent WebSearch '{"query":"ai governance"}'
  gov "$s" associate_agent mcp__systemprompt__list_agents '{"filter":"enabled"}'
  gov "$s" associate_agent mcp__skill-manager__delete_agent '{"agent":"x"}'
  track "$s" associate_agent Read '{"file":"README.md"}' $((40 + i * 20))
  track "$s" associate_agent WebSearch '{"query":"ai governance"}' $((800 + i * 150))
done
pass "20 associate_agent governance events (mix of allow + scope_restriction)"

# Secret-injection denials
for i in 1 2 3; do
  s="session-secret-0$i"
  gov "$s" developer_agent Write '{"content":"AKIA1234567890ABCDEF"}'
  gov "$s" associate_agent Write '{"content":"sk-ant-demo-FAKE1234567890"}'
done
pass "6 secret_injection denial events"

# Destructive blocklist hits from associate_agent
for i in 1 2; do
  s="session-destroy-0$i"
  gov "$s" associate_agent drop_table '{"table":"users"}'
  gov "$s" associate_agent delete_user '{"id":"42"}'
done
pass "4 tool_blocklist denial events"
echo ""

# ── STEP 5: Synthetic page view traffic ────────
subheader "STEP 5: Generate synthetic page view traffic"
paths=(/dashboard /admin/plugins /admin/agents /admin/skills /admin/governance /content/guides/ai-governance /content/guides/claude-code /analytics/costs /analytics/traffic /infra/logs)
referers=(https://news.ycombinator.com/ https://www.google.com/ https://twitter.com/ https://reddit.com/r/LocalLLaMA direct)
countries=(US GB DE FR JP AU CA SE IN BR)
agents_ua=("Mozilla/5.0 (Macintosh; Intel Mac OS X 14_4) Chrome/126" "Mozilla/5.0 (Windows NT 10.0) Firefox/125" "Mozilla/5.0 (iPhone; CPU iPhone OS 17_4) Safari/604.1" "Mozilla/5.0 (Linux; Android 14) Chrome/125")

posted=0
for i in $(seq 1 100); do
  p=${paths[$((RANDOM % ${#paths[@]}))]}
  r=${referers[$((RANDOM % ${#referers[@]}))]}
  c=${countries[$((RANDOM % ${#countries[@]}))]}
  ua=${agents_ua[$((RANDOM % ${#agents_ua[@]}))]}
  curl -s -X POST "$BASE_URL/api/public/hooks/track?plugin_id=enterprise-demo" \
    -H "Authorization: Bearer $TOKEN" \
    -H "Content-Type: application/json" \
    -d "{\"hook_event_name\":\"PageView\",\"tool_name\":\"page_view\",\"agent_id\":\"browser\",\"session_id\":\"traffic-$i\",\"tool_input\":{\"path\":\"$p\",\"referer\":\"$r\",\"country\":\"$c\",\"user_agent\":\"$ua\"},\"tool_result\":\"ok\",\"duration_ms\":$((120 + RANDOM % 800))}" \
    > /dev/null 2>&1 || true
  posted=$((posted + 1))
done
pass "$posted synthetic page_view events recorded"
echo ""

# ── STEP 6: Content ingestion ──────────────────
subheader "STEP 6: Ingest content"
cmd "systemprompt infra jobs run blog_content_ingestion"
"$CLI" infra jobs run blog_content_ingestion --profile "$PROFILE" 2>&1 | tail -3 | sed 's/^/  /' || true
echo ""

# ── STEP 7: Optional real agent run ────────────
if [[ "${SEED_AGENT_RUN:-0}" = "1" ]]; then
  subheader "STEP 7: Real agent invocation (costs tokens)"
  cmd "systemprompt admin agents message developer_agent --blocking"
  "$CLI" admin agents message developer_agent \
    --message "Summarise the governance pipeline in three bullet points." \
    --blocking --profile "$PROFILE" 2>&1 | tail -15 | sed 's/^/  /' || true
  echo ""
else
  info "Skipping real agent run (set SEED_AGENT_RUN=1 to enable)"
  echo ""
fi

# ── Verify ─────────────────────────────────────
subheader "Verify seed data"

count() {
  "$CLI" infra db query "SELECT COUNT(*) as count FROM $1" --profile "$PROFILE" 2>&1 \
    | grep -oP '"count":\s*\K[0-9]+' | head -1 || echo "0"
}

DECISIONS=$(count governance_decisions)
EVENTS=$(count plugin_usage_events)
SKILLS=$(count agent_skills)
CONTENT=$(count markdown_content)
ACTIVITY=$(count user_activity)
CONTEXTS=$(count contexts)

echo "  governance_decisions: ${DECISIONS:-0} rows"
echo "  plugin_usage_events:  ${EVENTS:-0} rows"
echo "  agent_skills:         ${SKILLS:-0} rows"
echo "  markdown_content:     ${CONTENT:-0} rows"
echo "  user_activity:        ${ACTIVITY:-0} rows"
echo "  contexts:             ${CONTEXTS:-0} rows"
echo ""

header "SEED DATA COMPLETE" "All demos now have rich baseline data to display"
