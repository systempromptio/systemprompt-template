#!/bin/bash
# DEMO 6: GOVERNANCE — SECRET INJECTION BREACH
# Demonstrates detection and blocking of plaintext secrets in tool inputs.
#
# When an LLM is tricked (via prompt injection) into passing a secret as a
# tool argument, the governance endpoint detects the secret pattern and
# immediately blocks the tool call — regardless of agent scope.

set -e

# Resolve the CLI binary
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
CLI="$PROJECT_DIR/target/debug/systemprompt"
if [[ -x "$PROJECT_DIR/target/release/systemprompt" ]]; then
  CLI="$PROJECT_DIR/target/release/systemprompt"
fi
if [[ ! -x "$CLI" ]]; then
  echo "ERROR: CLI binary not found. Run: cargo build" >&2
  exit 1
fi

echo ""
echo "=========================================="
echo "  DEMO 6: SECRET INJECTION BREACH"
echo ""
echo "  Even an admin-scope agent is BLOCKED"
echo "  when tool input contains plaintext secrets."
echo "=========================================="
echo ""

# Get a token for the API call
TOKEN=$("$CLI" cloud auth token 2>/dev/null || echo "")

if [[ -z "$TOKEN" ]]; then
  echo "ERROR: Could not obtain auth token. Is the platform running?" >&2
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
    "tool_input": {
      "file_path": "/home/user/project/src/main.rs"
    }
  }' | python3 -m json.tool 2>/dev/null || echo "(Could not pretty-print response)"

echo ""
echo "=========================================="
echo "  Tests 1-3: DENIED (secret_injection)"
echo "  Test 4:    ALLOWED (clean input)"
echo ""
echo "  The governance layer blocks plaintext"
echo "  secrets BEFORE they reach the tool —"
echo "  even for admin-scope agents."
echo "=========================================="
