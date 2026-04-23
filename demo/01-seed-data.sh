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
#   6. Synthetic page view traffic — 100 rows into engagement_events +
#      user_sessions (real analytics tables that power the traffic dashboard)
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
cmd "systemprompt core skills sync --direction to-db --yes"
"$CLI" core skills sync --direction to-db --yes --profile "$PROFILE" 2>&1 | tail -5 | sed 's/^/  /' || true
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

gov() {
  local session="$1" agent="$2" tool="$3" input="$4"
  curl -s -X POST "$BASE_URL/api/public/hooks/govern?plugin_id=enterprise-demo" \
    -H "Authorization: Bearer $TOKEN" \
    -H "Content-Type: application/json" \
    -d "{\"hook_event_name\":\"PreToolUse\",\"tool_name\":\"$tool\",\"agent_id\":\"$agent\",\"session_id\":\"$session\",\"cwd\":\"$PROJECT_DIR\",\"tool_input\":$input}" \
    > /dev/null 2>&1 || true
}

track() {
  local session="$1" agent="$2" tool="$3" input="$4" latency="$5"
  curl -s -X POST "$BASE_URL/api/public/hooks/track?plugin_id=enterprise-demo" \
    -H "Authorization: Bearer $TOKEN" \
    -H "Content-Type: application/json" \
    -d "{\"hook_event_name\":\"PostToolUse\",\"tool_name\":\"$tool\",\"agent_id\":\"$agent\",\"session_id\":\"$session\",\"cwd\":\"$PROJECT_DIR\",\"tool_input\":$input,\"tool_response\":\"ok\"}" \
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
# Inserts directly into the real analytics tables that power the traffic
# dashboard (engagement_events + user_sessions) so every KPI — top pages,
# referrer split, geo split, device breakdown — has live data. Uses psql
# because `infra db query` is read-only.
DB_URL="$(grep database_url "$PROJECT_DIR/.systemprompt/profiles/$PROFILE/secrets.json" 2>/dev/null \
  | head -1 \
  | sed 's/.*"database_url".*"\(postgres[^"]*\)".*/\1/')"
if [[ -z "$DB_URL" ]] || ! command -v psql >/dev/null 2>&1; then
  warn "Skipping traffic seed (psql or database_url unavailable)"
else
  psql "$DB_URL" -v ON_ERROR_STOP=1 -q <<'SQL' > /dev/null
  WITH dims AS (
    SELECT
      (SELECT id FROM users WHERE email = 'demo@systemprompt.io' LIMIT 1) AS demo_user_id,
      ARRAY['/dashboard','/admin/plugins','/admin/agents','/admin/skills','/admin/governance','/content/guides/ai-governance','/content/guides/claude-code','/analytics/costs','/analytics/traffic','/infra/logs']::text[] AS paths,
      ARRAY['google','hackernews','twitter','reddit','Direct']::text[] AS sources,
      ARRAY['US','GB','DE','FR','JP','AU','CA','SE','IN','BR']::text[] AS countries,
      ARRAY['desktop','desktop','mobile','tablet']::text[] AS devices,
      ARRAY['Chrome','Firefox','Safari','Edge']::text[] AS browsers
  ),
  pick AS (
    SELECT
      i,
      d.demo_user_id,
      'traffic-' || i AS sid,
      d.paths[1 + (random()*(array_length(d.paths,1)-1))::int] AS page_url,
      d.sources[1 + (random()*(array_length(d.sources,1)-1))::int] AS ref_source,
      d.countries[1 + (random()*(array_length(d.countries,1)-1))::int] AS country,
      d.devices[1 + (random()*(array_length(d.devices,1)-1))::int] AS device_type,
      d.browsers[1 + (random()*(array_length(d.browsers,1)-1))::int] AS browser,
      NOW() - (random() * interval '24 hours') AS ts,
      (30000 + (random()*270000)::int) AS time_on_page_ms,
      (20 + (random()*80)::int) AS scroll_depth
    FROM generate_series(1,100) AS i
    CROSS JOIN dims d
  ),
  ins_sessions AS (
    INSERT INTO user_sessions
      (session_id, user_id, started_at, last_activity_at, client_id, client_type,
       request_count, is_bot, is_scanner, country, device_type, browser,
       referrer_source, landing_page, session_source)
    SELECT
      sid, demo_user_id, ts, ts + interval '2 minutes', 'sp_web', 'firstparty',
      3, false, false, country, device_type, browser,
      ref_source, page_url, 'web'
    FROM pick
    ON CONFLICT (session_id) DO NOTHING
    RETURNING 1
  )
  INSERT INTO engagement_events
    (session_id, user_id, page_url, event_type, time_on_page_ms,
     max_scroll_depth, created_at, updated_at)
  SELECT
    sid, demo_user_id, page_url, 'page_exit', time_on_page_ms,
    scroll_depth, ts, ts
  FROM pick;
SQL
  pass "100 engagement_events + user_sessions rows seeded"
fi
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

# ── Install demo plugin for the authenticated user ─────
# Cowork sync is scoped per-user. Without this step, a freshly set-up template
# shows zero plugins/skills/agents/MCP in the manifest, and the Cowork desktop
# app receives an empty bundle. Forking enterprise-demo materialises
# user_plugins + user_plugin_skills + user_plugin_agents + user_plugin_mcp_servers
# rows for the current session's user.
subheader "Install enterprise-demo plugin for current user"
EXISTING=$(curl -s "$BASE_URL/api/public/admin/user/plugins" \
  -H "Authorization: Bearer $TOKEN" \
  | grep -o '"base_plugin_id":"enterprise-demo"' | head -1)
if [[ -n "$EXISTING" ]]; then
  info "enterprise-demo already installed for this user (skipping fork)"
else
  FORK_RESP=$(curl -s -o /tmp/fork_resp.json -w '%{http_code}' \
    -X POST "$BASE_URL/api/public/admin/user/fork/plugin" \
    -H "Authorization: Bearer $TOKEN" \
    -H "Content-Type: application/json" \
    -d '{"org_plugin_id":"enterprise-demo"}')
  if [[ "$FORK_RESP" == "201" ]]; then
    FORK_JSON=$(cat /tmp/fork_resp.json)
    FSKILLS=$(printf '%s' "$FORK_JSON" | sed -n 's/.*"forked_skills":[[:space:]]*\([0-9]*\).*/\1/p' | head -1)
    FAGENTS=$(printf '%s' "$FORK_JSON" | sed -n 's/.*"forked_agents":[[:space:]]*\([0-9]*\).*/\1/p' | head -1)
    pass "Forked enterprise-demo (skills=${FSKILLS:-?}, agents=${FAGENTS:-?})"
  else
    warn "Fork returned HTTP $FORK_RESP"
    cat /tmp/fork_resp.json 2>/dev/null | head -c 500
    echo ""
  fi
  rm -f /tmp/fork_resp.json
fi
echo ""

# ── Verify ─────────────────────────────────────
subheader "Verify seed data"

count() {
  "$CLI" infra db query "SELECT COUNT(*) as count FROM $1" --profile "$PROFILE" 2>&1 \
    | sed -n 's/.*"count":[[:space:]]*\([0-9]*\).*/\1/p' | head -1 || echo "0"
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
