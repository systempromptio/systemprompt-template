#!/bin/bash
# DEMO 6: GOVERNANCE — SECRET INJECTION BREACH
# Demonstrates detection and blocking of plaintext secrets in tool inputs.
#
# What this does:
#   Gets auth token, then sends 4 direct API calls to /api/public/hooks/govern:
#
#   Test 1 — AWS Access Key:
#     tool_input contains "AKIAIOSFODNN7EXAMPLE" in a curl command
#     → secret_injection rule detects AWS key pattern → DENY
#
#   Test 2 — GitHub PAT:
#     tool_input writes "ghp_ABCDEFghijklmnop..." to a .env file
#     → secret_injection rule detects GitHub PAT pattern → DENY
#
#   Test 3 — Private Key:
#     tool_input writes "-----BEGIN RSA PRIVATE KEY-----" to .ssh/id_rsa
#     → secret_injection rule detects PEM key header → DENY
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
#   curl → POST /hooks/govern → JWT auth → secret_injection scan
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

TOKEN="${1:-}"
if [[ -z "$TOKEN" && -f "$TOKEN_FILE" ]]; then
  TOKEN=$(cat "$TOKEN_FILE")
fi

if [[ -z "$TOKEN" ]]; then
  echo ""
  echo "  Run ./demo/00-preflight.sh first, or pass TOKEN as argument:"
  echo "  ./demo/governance/06-secret-breach.sh <TOKEN>"
  echo ""
  exit 1
fi

echo "------------------------------------------"
echo "  TEST 1: AWS Access Key in tool input"
echo "------------------------------------------"
echo ""

curl -s -X POST "http://localhost:8080/api/public/hooks/govern?plugin_id=enterprise-demo" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "hook_event_name": "PreToolUse",
    "tool_name": "Bash",
    "agent_id": "developer_agent",
    "session_id": "demo-secret-breach",
    "cwd": "/var/www/html/systemprompt-template",
    "tool_input": {
      "command": "curl -H \"Authorization: AKIAIOSFODNN7EXAMPLE\" https://s3.amazonaws.com/bucket",
      "description": "Fetch S3 object"
    }
  }' | python3 -m json.tool 2>/dev/null || echo "(Could not pretty-print response)"

echo ""
echo "------------------------------------------"
echo "  TEST 2: GitHub PAT in tool input"
echo "------------------------------------------"
echo ""

curl -s -X POST "http://localhost:8080/api/public/hooks/govern?plugin_id=enterprise-demo" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "hook_event_name": "PreToolUse",
    "tool_name": "Write",
    "agent_id": "developer_agent",
    "session_id": "demo-secret-breach",
    "cwd": "/var/www/html/systemprompt-template",
    "tool_input": {
      "file_path": "/home/user/.env",
      "content": "GITHUB_TOKEN=ghp_ABCDEFghijklmnop1234567890abcdef\nDATABASE_URL=postgres://localhost/db"
    }
  }' | python3 -m json.tool 2>/dev/null || echo "(Could not pretty-print response)"

echo ""
echo "------------------------------------------"
echo "  TEST 3: Private key in tool input"
echo "------------------------------------------"
echo ""

curl -s -X POST "http://localhost:8080/api/public/hooks/govern?plugin_id=enterprise-demo" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "hook_event_name": "PreToolUse",
    "tool_name": "Write",
    "agent_id": "developer_agent",
    "session_id": "demo-secret-breach",
    "cwd": "/var/www/html/systemprompt-template",
    "tool_input": {
      "file_path": "/home/user/.ssh/id_rsa",
      "content": "-----BEGIN RSA PRIVATE KEY-----\nMIIEpAIBAAKCAQEA..."
    }
  }' | python3 -m json.tool 2>/dev/null || echo "(Could not pretty-print response)"

echo ""
echo "------------------------------------------"
echo "  TEST 4: Clean input (no secrets) — PASSES"
echo "------------------------------------------"
echo ""

curl -s -X POST "http://localhost:8080/api/public/hooks/govern?plugin_id=enterprise-demo" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "hook_event_name": "PreToolUse",
    "tool_name": "Read",
    "agent_id": "developer_agent",
    "session_id": "demo-secret-breach",
    "cwd": "/var/www/html/systemprompt-template",
    "tool_input": {
      "file_path": "/home/user/project/src/main.rs"
    }
  }' | python3 -m json.tool 2>/dev/null || echo "(Could not pretty-print response)"

# ──────────────────────────────────────────────
#  AUDIT: Verify secret breach decisions
# ──────────────────────────────────────────────
echo ""
echo "=========================================="
echo "  AUDIT: Governance decisions for this session"
echo "=========================================="
echo ""

echo "  Decision counts (session=demo-secret-breach):"
"$CLI" infra db query \
  "SELECT decision, COUNT(*) as count FROM governance_decisions WHERE session_id = 'demo-secret-breach' GROUP BY decision ORDER BY decision" \
  2>&1 | grep -v "^\[profile"

echo ""
echo "  Expected: 3 deny + 1 allow = 4 total"
echo ""

echo "  Detailed decisions:"
"$CLI" infra db query \
  "SELECT decision, tool_name, policy, reason FROM governance_decisions WHERE session_id = 'demo-secret-breach' ORDER BY created_at" \
  2>&1 | grep -v "^\[profile"

echo ""
echo "=========================================="
echo "  AUDIT COMMANDS (run manually):"
echo "  infra db query \"SELECT * FROM governance_decisions WHERE session_id = 'demo-secret-breach' ORDER BY created_at\""
echo ""
echo "  Tests 1-3: DENIED (secret_injection)"
echo "  Test 4:    ALLOWED (clean input)"
echo ""
echo "  The governance layer blocks plaintext"
echo "  secrets BEFORE they reach the tool —"
echo "  even for admin-scope agents."
echo ""
echo "  Now run: ./demo/mcp/02-mcp-access-tracking.sh"
echo "=========================================="
