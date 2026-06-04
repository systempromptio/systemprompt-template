#!/bin/bash
# DEMO 6: GOVERNANCE — SECRET INJECTION BREACH
# Demonstrates detection and blocking of plaintext secrets in tool inputs.
#
# What this does:
#   Gets auth token, then sends 4 direct API calls to /api/public/hooks/govern:
#
#   Test 1 — AWS Access Key:
#     tool_input contains "AKIAIOSFODNN7EXAMPLE" in a curl command
#     → secret_scan rule detects AWS key pattern → DENY
#
#   Test 2 — GitHub PAT:
#     tool_input writes "ghp_ABCDEFghijklmnop..." to a .env file
#     → secret_scan rule detects GitHub PAT pattern → DENY
#
#   Test 3 — Private Key:
#     tool_input writes "-----BEGIN RSA PRIVATE KEY-----" to .ssh/id_rsa
#     → secret_scan rule detects PEM key header → DENY
#
#   Test 4 — Clean input (control):
#     tool_input reads a normal .rs source file
#     → No secrets detected → all rules pass → ALLOW
#
# Key point: Even admin-scope agents (developer_agent) are BLOCKED.
#   Secret detection overrides scope — it's a safety net against prompt
#   injection attacks that trick the LLM into leaking credentials.
#
# Flow per test:
#   curl → POST /hooks/govern → JWT auth → secret_scan scan
#   → pattern match on tool_input values → DENY (or ALLOW for clean input)
#
# Cost: Free (direct API calls, no AI)

set -e

source "$(cd "$(dirname "$0")/.." && pwd)/_common.sh"

echo ""
echo "=========================================="
echo "  DEMO 6: SECRET INJECTION BREACH"
echo ""
echo "  Even an admin-scope agent is BLOCKED"
echo "  when tool input contains plaintext secrets."
echo "=========================================="
echo ""

# secret_scan fires regardless of scope, so this demo deliberately uses the
# ADMIN token (demo/.token) to prove an admin-scope caller is blocked too. This
# is the opposite of the scope_check / tool_blocklist deny demos, which must use
# the user-scope token because admins are exempt from those two policies.
load_token "${1:-}"

# A per-run session id (PID-based — portable across BSD/GNU, no date dependency)
# so the audit summary below reflects EXACTLY this run's four calls, not the
# accumulation of every historical run against a fixed session id.
SESSION="demo-secret-breach-$$"

echo "  Session: $SESSION (token: admin scope — secret_scan denies it anyway)"
echo ""

echo "------------------------------------------"
echo "  TEST 1: AWS Access Key in tool input"
echo "------------------------------------------"
echo ""

RESPONSE=$(curl -s -X POST "${BASE_URL}/api/public/hooks/govern?plugin_id=enterprise-demo" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "hook_event_name": "PreToolUse",
    "tool_name": "Bash",
    "agent_id": "developer_agent",
    "session_id": "'"$SESSION"'",
    "cwd": "/var/www/html/systemprompt-template",
    "tool_input": {
      "command": "curl -H \"Authorization: AKIAIOSFODNN7EXAMPLE\" https://s3.amazonaws.com/bucket",
      "description": "Fetch S3 object"
    }
  }')
echo "$RESPONSE" | python3 -m json.tool 2>/dev/null || echo "(Could not pretty-print response)"
echo ""
assert_decision "$RESPONSE" "deny" "secret_scan denies AWS Access Key (admin scope)"

echo ""
echo "------------------------------------------"
echo "  TEST 2: GitHub PAT in tool input"
echo "------------------------------------------"
echo ""

RESPONSE=$(curl -s -X POST "${BASE_URL}/api/public/hooks/govern?plugin_id=enterprise-demo" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "hook_event_name": "PreToolUse",
    "tool_name": "Write",
    "agent_id": "developer_agent",
    "session_id": "'"$SESSION"'",
    "cwd": "/var/www/html/systemprompt-template",
    "tool_input": {
      "file_path": "/home/user/.env",
      "content": "GITHUB_TOKEN=ghp_ABCDEFghijklmnop1234567890abcdef\nDATABASE_URL=postgres://localhost/db"
    }
  }')
echo "$RESPONSE" | python3 -m json.tool 2>/dev/null || echo "(Could not pretty-print response)"
echo ""
assert_decision "$RESPONSE" "deny" "secret_scan denies GitHub PAT (admin scope)"

echo ""
echo "------------------------------------------"
echo "  TEST 3: Private key in tool input"
echo "------------------------------------------"
echo ""

RESPONSE=$(curl -s -X POST "${BASE_URL}/api/public/hooks/govern?plugin_id=enterprise-demo" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "hook_event_name": "PreToolUse",
    "tool_name": "Write",
    "agent_id": "developer_agent",
    "session_id": "'"$SESSION"'",
    "cwd": "/var/www/html/systemprompt-template",
    "tool_input": {
      "file_path": "/home/user/.ssh/id_rsa",
      "content": "-----BEGIN RSA PRIVATE KEY-----\nMIIEpAIBAAKCAQEA..."
    }
  }')
echo "$RESPONSE" | python3 -m json.tool 2>/dev/null || echo "(Could not pretty-print response)"
echo ""
assert_decision "$RESPONSE" "deny" "secret_scan denies RSA Private Key (admin scope)"

echo ""
echo "------------------------------------------"
echo "  TEST 4: Clean input (no secrets) — PASSES"
echo "------------------------------------------"
echo ""

RESPONSE=$(curl -s -X POST "${BASE_URL}/api/public/hooks/govern?plugin_id=enterprise-demo" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "hook_event_name": "PreToolUse",
    "tool_name": "Read",
    "agent_id": "developer_agent",
    "session_id": "'"$SESSION"'",
    "cwd": "/var/www/html/systemprompt-template",
    "tool_input": {
      "file_path": "/home/user/project/src/main.rs"
    }
  }')
echo "$RESPONSE" | python3 -m json.tool 2>/dev/null || echo "(Could not pretty-print response)"
echo ""
assert_decision "$RESPONSE" "allow" "clean input passes all stages"

# ──────────────────────────────────────────────
#  AUDIT: Verify secret breach decisions
# ──────────────────────────────────────────────
echo ""
echo "=========================================="
echo "  AUDIT: Governance decisions for this session"
echo "=========================================="
echo ""

echo "  Decision counts (session=$SESSION):"
"$CLI" infra db query \
  "SELECT decision, COUNT(*) as count FROM governance_decisions WHERE session_id = '$SESSION' GROUP BY decision ORDER BY decision" \
  2>&1 | grep -v "^\[profile"

echo ""
echo "  Expected: 3 deny (secret_scan) + 1 allow (clean) = 4 total"
echo ""
assert_eq "$(db_count "SELECT COUNT(*) FROM governance_decisions WHERE session_id = '$SESSION' AND decision = 'deny' AND policy = 'secret_scan'")" \
  "3" "3 secret_scan denials landed for this session"
assert_min "$(db_count "SELECT COUNT(*) FROM governance_decisions WHERE session_id = '$SESSION' AND decision = 'allow'")" \
  1 "clean input allowed for this session"
echo ""

echo "  Detailed decisions (most recent first):"
"$CLI" infra db query \
  "SELECT decision, tool_name, policy, reason FROM governance_decisions WHERE session_id = '$SESSION' ORDER BY created_at DESC" \
  2>&1 | grep -v "^\[profile"

echo ""
echo "=========================================="
echo "  AUDIT COMMANDS (run manually):"
echo "  $CLI infra db query \"SELECT * FROM governance_decisions WHERE session_id = '$SESSION' ORDER BY created_at\""
echo ""
echo "  Tests 1-3: DENIED (secret_scan)"
echo "  Test 4:    ALLOWED (clean input)"
echo ""
echo "  The governance layer blocks plaintext"
echo "  secrets BEFORE they reach the tool —"
echo "  even for admin-scope agents."
echo ""
echo "  Now run: ./demo/mcp/02-mcp-access-tracking.sh"
echo "=========================================="
