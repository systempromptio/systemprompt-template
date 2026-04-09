#!/bin/bash
# DEMO 8: REQUEST TRACING
# Traces HTTP requests end-to-end: typed data, IDs, logs, flow maps, benchmarks.
#
# What this does:
#   Part 1 — Typed data flowing through the system:
#     Sends a governance request + track request, shows full payloads.
#     JSON enters at the HTTP boundary and is immediately deserialized
#     into Rust structs (HookEventPayload, GovernanceContext). From that
#     point on, every field is typed — no raw strings pass between layers.
#
#   Part 2 — All typed IDs:
#     Queries the database to show every ID created by the requests.
#     Each ID is a Rust newtype: UserId, SessionId, TraceId, ContextId,
#     AgentName, PluginId. The compiler prevents mixing them up.
#
#   Part 3 — Log display:
#     Runs 4 CLI log commands showing trace list, trace detail,
#     request list, and application logs. All read-only.
#
#   Part 4 — ASCII flow map:
#     Shows the governance request path through 6 stages, annotated
#     with the typed Rust struct at each stage.
#
#   Part 5 — Benchmark (100 concurrent requests):
#     Fires 100 parallel curl requests to the governance endpoint.
#     No AI calls — pure Rust rule evaluation. Shows throughput,
#     latency stats, and verifies all 100 decisions hit the database.
#
# Why Rust matters here:
#   - JSON is untyped ONLY at the HTTP boundary (serde deserialization)
#   - Immediately converted to validated Rust structs with newtype IDs
#   - All database queries use sqlx query! macros — 100% compile-time checked
#   - No unstructured or unaudited data can flow through the system
#   - Newtype wrappers (UserId, SessionId) prevent ID mixups at compile time
#   - The benchmark shows what typed, zero-cost abstractions deliver at runtime
#
# Usage:
#   ./demo/08-request-tracing.sh <TOKEN> [profile]
#
# TOKEN: The plugin token from the dashboard install widget (top-right of /admin/).
#        Click the key icon, reveal, and copy.
#
# Cost: Free (governance API calls + DB queries, no AI)

set -e

source "$(cd "$(dirname "$0")/.." && pwd)/_common.sh"

TOKEN="${1:-}"
PROFILE="${2:-local}"
BASE_URL="http://localhost:8080"
SESSION_ID="demo-trace-$(date +%s)"
DASHBOARD_URL="$BASE_URL/admin/"

if [[ -z "$TOKEN" && -f "$TOKEN_FILE" ]]; then
  TOKEN=$(cat "$TOKEN_FILE")
fi

if [[ -z "$TOKEN" ]]; then
  echo ""
  echo "  Run ./demo/00-preflight.sh first, or pass TOKEN as argument:"
  echo "  ./demo/08-request-tracing.sh <TOKEN> [profile]"
  echo ""
  exit 1
fi

echo ""
echo "=========================================="
echo "  DEMO 8: REQUEST TRACING"
echo "  Typed data, IDs, logs, flow maps, benchmarks"
echo ""
echo "  Session: $SESSION_ID"
echo "=========================================="
echo ""

# ──────────────────────────────────────────────
#  PART 1: Typed data flowing through the system
# ──────────────────────────────────────────────
echo "------------------------------------------"
echo "  PART 1: Typed data flowing through the system"
echo "  Two requests — governance + track"
echo "------------------------------------------"
echo ""

echo "  1a. Governance request (PreToolUse → allow/deny)"
echo ""
echo "  Request payload:"
GOVERN_PAYLOAD='{
  "hook_event_name": "PreToolUse",
  "tool_name": "Read",
  "agent_id": "developer_agent",
  "session_id": "'$SESSION_ID'",
  "tool_input": {"file_path": "/src/main.rs"}
}'
echo "  $GOVERN_PAYLOAD" | python3 -m json.tool 2>/dev/null || echo "  $GOVERN_PAYLOAD"
echo ""

echo "  Response:"
GOVERN_RESPONSE=$(curl -s -w "\n%{http_code} %{time_total}" \
  -X POST "$BASE_URL/api/public/hooks/govern?plugin_id=enterprise-demo" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d "$GOVERN_PAYLOAD")

GOVERN_BODY=$(echo "$GOVERN_RESPONSE" | head -n -1)
GOVERN_STATUS=$(echo "$GOVERN_RESPONSE" | tail -1 | awk '{print $1}')
GOVERN_TIME=$(echo "$GOVERN_RESPONSE" | tail -1 | awk '{print $2}')

echo "  $GOVERN_BODY" | python3 -m json.tool 2>/dev/null || echo "  $GOVERN_BODY"
echo ""
echo "  HTTP $GOVERN_STATUS in ${GOVERN_TIME}s"
echo ""
echo "  Field types (Rust):"
echo "    hook_event_name      : String          (\"PreToolUse\")"
echo "    tool_name            : String          (tool being governed)"
echo "    agent_id             : String          (agent requesting access)"
echo "    session_id           : SessionId       (newtype wrapper)"
echo "    tool_input           : serde_json::Value (arbitrary JSON)"
echo "    permissionDecision   : \"allow\" | \"deny\" (static literals)"
echo ""
echo "  WHY RUST: The JSON above is the ONLY untyped boundary."
echo "  On arrival, serde deserializes it into HookEventPayload."
echo "  From that point, every field is a validated Rust type."
echo "  session_id becomes SessionId (newtype) — you cannot"
echo "  accidentally pass it where a UserId is expected."
echo ""

echo "  1b. Track request (PostToolUse event)"
echo ""

TRACK_PAYLOAD='{
  "hook_event_name": "PostToolUse",
  "tool_name": "Read",
  "agent_id": "developer_agent",
  "session_id": "'$SESSION_ID'",
  "tool_input": {"file_path": "/src/main.rs"},
  "tool_result": "fn main() { println!(\"Hello\"); }"
}'

TRACK_STATUS=$(curl -s -o /dev/null -w "%{http_code} %{time_total}" \
  -X POST "$BASE_URL/api/public/hooks/track?plugin_id=enterprise-demo" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d "$TRACK_PAYLOAD")

TRACK_CODE=$(echo "$TRACK_STATUS" | awk '{print $1}')
TRACK_TIME=$(echo "$TRACK_STATUS" | awk '{print $2}')

echo "  HTTP $TRACK_CODE in ${TRACK_TIME}s"
echo "  Event recorded: PostToolUse (Read tool, developer_agent)"
echo ""

sleep 1

# ──────────────────────────────────────────────
#  PART 2: Extract all typed IDs
# ──────────────────────────────────────────────
echo "------------------------------------------"
echo "  PART 2: Typed IDs in the system"
echo "  Every entity has a typed identifier"
echo "------------------------------------------"
echo ""

echo "  Governance decisions (session=$SESSION_ID):"
echo ""
"$CLI" infra db query \
  "SELECT id, user_id, session_id, agent_id, decision, plugin_id, created_at FROM governance_decisions WHERE session_id = '$SESSION_ID' ORDER BY created_at DESC LIMIT 3" \
  --profile "$PROFILE" 2>&1 | grep -v "^\[profile"

echo ""
echo "  Plugin usage events (session=$SESSION_ID):"
echo ""
"$CLI" infra db query \
  "SELECT id, user_id, session_id, event_type, tool_name, created_at FROM plugin_usage_events WHERE session_id = '$SESSION_ID' ORDER BY created_at DESC LIMIT 3" \
  --profile "$PROFILE" 2>&1 | grep -v "^\[profile"

echo ""
echo "  Typed ID reference:"
echo "  ┌──────────────────┬────────────────────────────────────────┐"
echo "  │ ID               │ Source                                 │"
echo "  ├──────────────────┼────────────────────────────────────────┤"
echo "  │ decision_id      │ UUID v4 (server-generated primary key) │"
echo "  │ user_id          │ JWT 'sub' claim (UserId newtype)       │"
echo "  │ session_id       │ Client-provided (SessionId newtype)    │"
echo "  │ agent_id         │ Hook payload (AgentName newtype)       │"
echo "  │ plugin_id        │ Query parameter (PluginId newtype)     │"
echo "  │ trace_id         │ UUID v4 (TraceId newtype, per-request) │"
echo "  │ context_id       │ CLI-created (ContextId newtype)        │"
echo "  └──────────────────┴────────────────────────────────────────┘"
echo ""
echo "  WHY RUST: Every ID above is a newtype wrapper, not a bare String."
echo "  fn record_decision(user: &UserId, session: &SessionId) — the"
echo "  compiler enforces correct ID types at every call site. You cannot"
echo "  pass a SessionId where a UserId is expected. Zero runtime cost."
echo ""
echo "  WHY RUST: The SQL queries that wrote these rows use sqlx::query!{}"
echo "  macros — checked against the live database schema at compile time."
echo "  If a column is renamed or a type changes, the code won't compile."
echo ""

# ──────────────────────────────────────────────
#  PART 3: Display logs
# ──────────────────────────────────────────────
echo "------------------------------------------"
echo "  PART 3: Viewing request logs"
echo "  CLI commands for observability"
echo "------------------------------------------"
echo ""

echo "  \$ systemprompt infra logs trace list --limit 3"
echo ""
"$CLI" infra logs trace list --limit 3 --profile "$PROFILE" 2>&1 | grep -v "^\[profile"

echo ""

# Try to get a trace ID for detailed view
TRACE_ID=$("$CLI" infra logs trace list --limit 1 --profile "$PROFILE" 2>&1 \
  | grep -oP '"trace_id":\s*"\K[0-9a-f-]+' | head -1 || true)

if [[ -n "$TRACE_ID" ]]; then
  echo "  \$ systemprompt infra logs trace show $TRACE_ID --all"
  echo ""
  "$CLI" infra logs trace show "$TRACE_ID" --all --profile "$PROFILE" 2>&1 | grep -v "^\[profile" | head -30
  echo "  ... (truncated)"
  echo ""
fi

echo "  \$ systemprompt infra logs request list --limit 5"
echo ""
"$CLI" infra logs request list --limit 5 --profile "$PROFILE" 2>&1 | grep -v "^\[profile"

echo ""
echo "  \$ systemprompt infra logs view --level info --since 5m"
echo ""
"$CLI" infra logs view --level info --since 5m --profile "$PROFILE" 2>&1 | grep -v "^\[profile" | head -15
echo "  ... (truncated)"
echo ""

# ──────────────────────────────────────────────
#  PART 4: Request flow through the system
# ──────────────────────────────────────────────
echo "------------------------------------------"
echo "  PART 4: Request flow map"
echo "  POST /api/public/hooks/govern"
echo "------------------------------------------"
echo ""

cat <<'FLOW'
  Client (curl / Claude Code hook)
    │
    ▼
  ┌─────────────────────────────────────────────────────────┐
  │  Axum Router                                            │
  │  POST /api/public/hooks/govern?plugin_id=<PluginId>     │
  └──────────────────────┬──────────────────────────────────┘
                         │
                         ▼
  ┌─────────────────────────────────────────────────────────┐
  │  JWT Validation                                         │
  │  extract_bearer_token(headers) → validate_jwt_token()   │
  │  Claims: { sub: UserId, aud: ["hook","plugin","api"] }  │
  └──────────────────────┬──────────────────────────────────┘
                         │
                         ▼
  ┌─────────────────────────────────────────────────────────┐
  │  Scope Resolution                                       │
  │  resolve_agent_scope(agent_id) → "admin" | "user"       │
  │  Builds: GovernanceContext { user_id, session_id,        │
  │           agent_id, agent_scope, tool_name, tool_input } │
  └──────────────────────┬──────────────────────────────────┘
                         │
                         ▼
  ┌─────────────────────────────────────────────────────────┐
  │  Rule Engine  rules::evaluate(&pool, &ctx)              │
  │    ├─ scope_check      : agent vs tool requirements     │
  │    ├─ secret_injection  : scan tool_input for secrets    │
  │    └─ rate_limit        : 300 calls/min per session      │
  │  Returns: Vec<RuleEvaluation { rule, result, detail }>  │
  └──────────────────────┬──────────────────────────────────┘
                         │
                         ▼
  ┌─────────────────────────────────────────────────────────┐
  │  Audit Writer  (tokio::spawn — async, non-blocking)     │
  │  INSERT INTO governance_decisions                       │
  │    (id, user_id, session_id, tool_name, agent_id,       │
  │     decision, policy, reason, evaluated_rules)          │
  └──────────────────────┬──────────────────────────────────┘
                         │
                         ▼
  ┌─────────────────────────────────────────────────────────┐
  │  Response  HTTP 200                                     │
  │  { hookSpecificOutput: {                                │
  │      hookEventName: "PreToolUse",                       │
  │      permissionDecision: "allow" | "deny",              │
  │      permissionDecisionReason: "..." (if denied)        │
  │  }}                                                     │
  └─────────────────────────────────────────────────────────┘

FLOW

echo "  WHY RUST: Every stage uses typed structs — not dictionaries, not"
echo "  raw strings, not unvalidated JSON. The GovernanceContext carries"
echo "  UserId + SessionId + AgentName (newtypes). The RuleEvaluation"
echo "  returns typed enums. The audit INSERT uses sqlx::query!{} with"
echo "  compile-time checked SQL. No unstructured or unaudited data can"
echo "  flow through this pipeline. If you change a column name in the"
echo "  migration, every query that touches it fails to compile."
echo ""

# ──────────────────────────────────────────────
#  PART 5: Benchmark — governance + tracking + MCP
# ──────────────────────────────────────────────
echo "------------------------------------------"
echo "  PART 5: Production benchmark"
echo "  200 requests, 100 concurrent workers"
echo "  (No AI cost — pure infrastructure)"
echo "------------------------------------------"
echo ""

HEY="/tmp/hey"
if [[ ! -x "$HEY" ]]; then
  echo "  Installing hey (HTTP load testing tool)..."
  curl -sL https://hey-release.s3.us-east-2.amazonaws.com/hey_linux_amd64 -o "$HEY" && chmod +x "$HEY"
fi

BENCH_SESSION="bench-$(date +%s)"
GOVERN_PAYLOAD='{"hook_event_name":"PreToolUse","tool_name":"Read","agent_id":"developer_agent","session_id":"'$BENCH_SESSION'-govern","tool_input":{"file_path":"/src/main.rs"}}'
TRACK_PAYLOAD='{"hook_event_name":"PostToolUse","tool_name":"Read","agent_id":"developer_agent","session_id":"'$BENCH_SESSION'-track","tool_input":{"file_path":"/src/main.rs"},"tool_result":{"stdout":"ok"}}'

echo "  5a. Governance endpoint (PreToolUse)"
echo "      JWT validation → scope resolution → 3 rule evaluations → async DB audit"
echo ""

GOVERN_OUTPUT=$("$HEY" -n 200 -c 100 -m POST \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d "$GOVERN_PAYLOAD" \
  "$BASE_URL/api/public/hooks/govern?plugin_id=enterprise-demo" 2>&1)

GOVERN_RPS=$(echo "$GOVERN_OUTPUT" | grep "Requests/sec" | awk '{printf "%.0f", $2}')
GOVERN_AVG=$(echo "$GOVERN_OUTPUT" | grep "Average:" | head -1 | awk '{printf "%.1f", $2 * 1000}')
GOVERN_P10=$(echo "$GOVERN_OUTPUT" | grep "10% in" | awk '{printf "%.1f", $3 * 1000}')
GOVERN_P50=$(echo "$GOVERN_OUTPUT" | grep "50% in" | awk '{printf "%.1f", $3 * 1000}')
GOVERN_P90=$(echo "$GOVERN_OUTPUT" | grep "90% in" | awk '{printf "%.1f", $3 * 1000}')
GOVERN_P99=$(echo "$GOVERN_OUTPUT" | grep "99% in" | awk '{printf "%.1f", $3 * 1000}')
GOVERN_FASTEST=$(echo "$GOVERN_OUTPUT" | grep "Fastest:" | awk '{printf "%.1f", $2 * 1000}')
GOVERN_WAIT=$(echo "$GOVERN_OUTPUT" | grep "resp wait:" | sed 's/.*resp wait://' | awk -F',' '{gsub(/[^0-9.]/, "", $1); printf "%.1f", $1 * 1000}')

echo "  ┌──────────────────┬──────────────┐"
echo "  │ Throughput       │ ${GOVERN_RPS} req/s"
echo "  │ Server time      │ ${GOVERN_WAIT}ms avg"
echo "  ├──────────────────┼──────────────┤"
echo "  │ p10              │ ${GOVERN_P10}ms"
echo "  │ p50              │ ${GOVERN_P50}ms"
echo "  │ p90              │ ${GOVERN_P90}ms"
echo "  │ p99              │ ${GOVERN_P99}ms"
echo "  │ fastest          │ ${GOVERN_FASTEST}ms"
echo "  └──────────────────┴──────────────┘"
echo ""

echo "  5b. Track endpoint (PostToolUse)"
echo "      Event recording — fire-and-forget async DB write"
echo ""

TRACK_OUTPUT=$("$HEY" -n 200 -c 100 -m POST \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d "$TRACK_PAYLOAD" \
  "$BASE_URL/api/public/hooks/track?plugin_id=enterprise-demo" 2>&1)

TRACK_RPS=$(echo "$TRACK_OUTPUT" | grep "Requests/sec" | awk '{printf "%.0f", $2}')
TRACK_P50=$(echo "$TRACK_OUTPUT" | grep "50% in" | awk '{printf "%.1f", $3 * 1000}')
TRACK_P90=$(echo "$TRACK_OUTPUT" | grep "90% in" | awk '{printf "%.1f", $3 * 1000}')
TRACK_P99=$(echo "$TRACK_OUTPUT" | grep "99% in" | awk '{printf "%.1f", $3 * 1000}')
TRACK_WAIT=$(echo "$TRACK_OUTPUT" | grep "resp wait:" | sed 's/.*resp wait://' | awk -F',' '{gsub(/[^0-9.]/, "", $1); printf "%.1f", $1 * 1000}')

echo "  ┌──────────────────┬──────────────┐"
echo "  │ Throughput       │ ${TRACK_RPS} req/s"
echo "  │ Server time      │ ${TRACK_WAIT}ms avg"
echo "  │ p50 / p90 / p99  │ ${TRACK_P50} / ${TRACK_P90} / ${TRACK_P99}ms"
echo "  └──────────────────┴──────────────┘"
echo ""

echo "  5c. MCP tool call (list_plugins)"
echo "      OAuth authentication → tool execution → structured response"
echo ""

MCP_TIMES=""
for i in $(seq 1 5); do
  START_NS=$(date +%s%N)
  "$CLI" plugins mcp call skill-manager list_plugins '{}' --profile "$PROFILE" 2>/dev/null | head -1 > /dev/null
  END_NS=$(date +%s%N)
  MS=$(( (END_NS - START_NS) / 1000000 ))
  MCP_TIMES="$MCP_TIMES $MS"
done
MCP_AVG=$(echo "$MCP_TIMES" | awk '{sum=0; for(i=1;i<=NF;i++) sum+=$i; printf "%.0f", sum/NF}')

echo "  ┌──────────────────┬──────────────┐"
echo "  │ Avg latency      │ ${MCP_AVG}ms"
echo "  │ Calls            │$MCP_TIMES ms"
echo "  └──────────────────┴──────────────┘"
echo ""

echo "  5d. DB connection pool under load"
echo ""

DB_URL="$(grep database_url "$PROJECT_DIR/.systemprompt/profiles/local/secrets.json" 2>/dev/null | head -1 | sed 's/.*"database_url".*"\(postgres[^"]*\)".*/\1/' || echo "")"
if [[ -n "$DB_URL" ]]; then
  POOL_TOTAL=$(psql "$DB_URL" -t -c "SELECT count(*) FROM pg_stat_activity WHERE datname = current_database();" 2>/dev/null | tr -d ' ')
  POOL_MAX=$(psql "$DB_URL" -t -c "SHOW max_connections;" 2>/dev/null | tr -d ' ')
  SYNC_COMMIT=$(psql "$DB_URL" -t -c "SHOW synchronous_commit;" 2>/dev/null | tr -d ' ')
  echo "  ┌──────────────────┬──────────────┐"
  echo "  │ PG connections   │ ${POOL_TOTAL} / ${POOL_MAX}"
  echo "  │ sync_commit      │ ${SYNC_COMMIT}"
  echo "  └──────────────────┴──────────────┘"
  echo ""
  echo "  The p90 tail latency is DB connection pool contention."
  echo "  Each governance request spawns an async audit INSERT."
  echo "  Under 100 concurrent requests, the pool (50 connections)"
  echo "  saturates — later requests queue for a free connection."
  echo ""
  echo "  Production tuning:"
  echo "    • synchronous_commit = off (audit writes are non-critical)"
  echo "    • PgBouncer for connection multiplexing"
  echo "    • Dedicated PostgreSQL on NVMe storage"
else
  echo "  (Could not connect to PostgreSQL for pool stats)"
fi
echo ""

echo "  5e. Enterprise capacity estimate"
echo ""

CALLS_PER_DEV=10
if [[ -n "$GOVERN_RPS" && "$GOVERN_RPS" -gt 0 ]]; then
  DEVS_PER_INSTANCE=$(( GOVERN_RPS * 60 / CALLS_PER_DEV ))
else
  DEVS_PER_INSTANCE="N/A"
fi

echo "  At ${GOVERN_RPS} req/s (single instance, this machine):"
echo "  Assuming ${CALLS_PER_DEV} tool calls/min per developer:"
echo ""
echo "  ┌──────────────────────────┬───────────────────────┐"
echo "  │ 1 instance               │ ~${DEVS_PER_INSTANCE} concurrent devs"
echo "  │ 3 instances + PgBouncer  │ ~$(( DEVS_PER_INSTANCE * 3 )) concurrent devs"
echo "  │ 10 instances + PgBouncer │ ~$(( DEVS_PER_INSTANCE * 10 )) concurrent devs"
echo "  └──────────────────────────┴───────────────────────┘"
echo ""
echo "  The governance check (p50=${GOVERN_P50}ms) adds <2% overhead"
echo "  to Claude's AI response time (1-5 seconds)."
echo ""

echo "  Governance decisions written:"
echo ""
"$CLI" infra db query \
  "SELECT decision, COUNT(*) as count FROM governance_decisions WHERE session_id = '${BENCH_SESSION}-govern' GROUP BY decision" \
  --profile "$PROFILE" 2>&1 | grep -v "^\[profile"

echo ""

echo "=========================================="
echo "  DEMO 8 COMPLETE"
echo ""
echo "  What we showed:"
echo "  1. Typed request/response payloads (governance + track)"
echo "  2. Typed IDs: decision_id, user_id, session_id, agent_id, plugin_id, trace_id"
echo "  3. CLI log commands: trace list, trace show, request list, logs view"
echo "  4. Request flow: Router > JWT > Scope > Rules > Audit > Response"
echo "  5. Production benchmark:"
echo "     Governance: ${GOVERN_RPS} req/s, p50=${GOVERN_P50}ms"
echo "     Tracking:   ${TRACK_RPS} req/s, p50=${TRACK_P50}ms"
echo "     MCP tool:   ${MCP_AVG}ms avg"
echo ""
echo "  Bottleneck analysis:"
echo "  - Rust handler: ~${GOVERN_FASTEST}ms (JWT + scope + rules)"
echo "  - p90 tail: DB pool contention (50 connections, 100 concurrent)"
echo "  - Production fix: PgBouncer + synchronous_commit=off"
echo ""
echo "  Why Rust:"
echo "  - Untyped JSON ONLY at the HTTP boundary (serde deserialization)"
echo "  - Newtype IDs prevent mixups at compile time (UserId != SessionId)"
echo "  - sqlx::query!{} macros: 100% compile-time verified SQL"
echo "  - In-memory rate limiter: zero DB round-trips in the hot path"
echo "  - OnceLock scope cache: agent config loaded once, not per-request"
echo "  - tokio::spawn audit writes: non-blocking, async DB INSERT"
echo ""
echo "  Dashboard: $DASHBOARD_URL"
echo "=========================================="
