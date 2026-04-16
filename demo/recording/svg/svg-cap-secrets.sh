#!/bin/bash
# SVG RECORDING: Secret Detection
# Four tests. Three blocked. One clean.
set -e
source "$(dirname "$0")/_colors.sh"

header "SECRET DETECTION" "Scanning tool inputs for plaintext credentials"
pause 1

# ── Test 1: AWS Key ──
subheader "Test 1 — AWS Access Key" "AKIAIOSFODNN7EXAMPLE in tool input"
pause 0.5

type_cmd "systemprompt hooks govern --tool Bash --input '{\"command\": \"curl -H AKIAIOSFODNN7EXAMPLE\"}'"
pause 0.3

RESPONSE=$(curl -s -X POST "$BASE_URL/api/public/hooks/govern?plugin_id=svg-demo" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"hook_event_name":"PreToolUse","tool_name":"Bash","agent_id":"developer_agent","session_id":"svg-secrets","cwd":"/var/www/html/systemprompt-template","tool_input":{"command":"curl -H \"Authorization: AKIAIOSFODNN7EXAMPLE\" https://api.example.com"}}' 2>/dev/null)

echo "$RESPONSE" | color_json
echo ""
fail "AWS access key detected"
pause 1.5

divider

# ── Test 2: GitHub PAT ──
subheader "Test 2 — GitHub Personal Access Token" "ghp_ABCDEFghijklmnop12345678 in tool input"
pause 0.5

type_cmd "systemprompt hooks govern --tool Bash --input '{\"command\": \"curl ghp_ABCDEF...\"}'"
pause 0.3

RESPONSE=$(curl -s -X POST "$BASE_URL/api/public/hooks/govern?plugin_id=svg-demo" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"hook_event_name":"PreToolUse","tool_name":"Bash","agent_id":"developer_agent","session_id":"svg-secrets","cwd":"/var/www/html/systemprompt-template","tool_input":{"command":"curl -H \"Authorization: token ghp_ABCDEFghijklmnop12345678\" https://api.github.com"}}' 2>/dev/null)

echo "$RESPONSE" | color_json
echo ""
fail "GitHub token detected"
pause 1.5

divider

# ── Test 3: Private Key ──
subheader "Test 3 — RSA Private Key" "-----BEGIN RSA PRIVATE KEY----- in tool input"
pause 0.5

type_cmd "systemprompt hooks govern --tool Write --input '{\"content\": \"-----BEGIN RSA...\"}'"
pause 0.3

RESPONSE=$(curl -s -X POST "$BASE_URL/api/public/hooks/govern?plugin_id=svg-demo" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"hook_event_name":"PreToolUse","tool_name":"Write","agent_id":"developer_agent","session_id":"svg-secrets","cwd":"/var/www/html/systemprompt-template","tool_input":{"file_path":"/tmp/key.pem","content":"-----BEGIN RSA PRIVATE KEY-----\nMIIEpAIBAAKCAQEA..."}}' 2>/dev/null)

echo "$RESPONSE" | color_json
echo ""
fail "private key detected"
pause 1.5

divider

# ── Test 4: Clean input ──
subheader "Test 4 — Clean File Read" "/home/user/project/src/main.rs"
pause 0.5

type_cmd "systemprompt hooks govern --tool Read --input '{\"file_path\": \"/src/main.rs\"}'"
pause 0.3

RESPONSE=$(curl -s -X POST "$BASE_URL/api/public/hooks/govern?plugin_id=svg-demo" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"hook_event_name":"PreToolUse","tool_name":"Read","agent_id":"developer_agent","session_id":"svg-secrets","cwd":"/var/www/html/systemprompt-template","tool_input":{"file_path":"/home/user/project/src/main.rs"}}' 2>/dev/null)

echo "$RESPONSE" | color_json
echo ""
pass "no secrets detected"
pause 2

divider

echo -e "  ${CYAN}${BOLD}3 blocked. 1 allowed. All audited.${RESET}"
echo ""
pause 2
