#!/bin/bash
# auth_jsonrpc.sh - Authenticate with Odoo using JSON-RPC protocol (< v19)
# Usage: ./auth_jsonrpc.sh
# Requires: ODOO_URL, ODOO_DB, ODOO_USER, ODOO_KEY environment variables
# Output: Prints session_id and uid to stdout (use with eval)
# Note: Tries session auth first, falls back to common.authenticate for API keys

set -euo pipefail

# Colors for output (to stderr)
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Check required environment variables
if [[ -z "${ODOO_URL:-}" ]] || [[ -z "${ODOO_DB:-}" ]] || [[ -z "${ODOO_USER:-}" ]] || [[ -z "${ODOO_KEY:-}" ]]; then
    echo -e "${RED}Error: Missing required environment variables${NC}" >&2
    echo "Required: ODOO_URL, ODOO_DB, ODOO_USER, ODOO_KEY" >&2
    exit 1
fi

# First, get server version
VERSION_RESPONSE=$(curl -s -X POST "${ODOO_URL}/jsonrpc" \
  -H 'Content-Type: application/json' \
  -d '{"jsonrpc":"2.0","method":"call","params":{"service":"common","method":"version","args":[]},"id":1}' 2>&1)

VERSION=$(echo "$VERSION_RESPONSE" | grep -o '"server_version"[[:space:]]*:[[:space:]]*"[^"]*"' | cut -d'"' -f4 || echo "unknown")

# Try method 1: Session authentication (works with passwords)
echo -e "${YELLOW}Attempting session authentication...${NC}" >&2
RESPONSE=$(curl -s -X POST "${ODOO_URL}/web/session/authenticate" \
  -H 'Content-Type: application/json' \
  -d "{
    \"jsonrpc\": \"2.0\",
    \"method\": \"call\",
    \"params\": {
        \"db\": \"${ODOO_DB}\",
        \"login\": \"${ODOO_USER}\",
        \"password\": \"${ODOO_KEY}\"
    },
    \"id\": null
  }" 2>&1)

# Check if session auth succeeded
if ! echo "$RESPONSE" | grep -q '"error"'; then
    SESSION_ID=$(echo "$RESPONSE" | grep -o '"session_id":"[^"]*"' | cut -d'"' -f4)
    USER_ID=$(echo "$RESPONSE" | grep -o '"uid":[0-9]*' | grep -o '[0-9]*')

    if [[ -n "$SESSION_ID" ]] && [[ -n "$USER_ID" ]] && [[ "$USER_ID" != "false" ]]; then
        echo -e "${GREEN}✓ Authenticated successfully (session)${NC}" >&2
        echo "User ID: ${USER_ID}" >&2
        echo "Session ID: ${SESSION_ID:0:20}..." >&2
        echo "Odoo Version: ${VERSION}" >&2

        echo "export ODOO_SESSION_ID='${SESSION_ID}'"
        echo "export ODOO_UID='${USER_ID}'"
        echo "export ODOO_VERSION='${VERSION}'"
        exit 0
    fi
fi

# Method 2: common.authenticate (works with API keys)
echo -e "${YELLOW}Session auth failed, trying API key authentication...${NC}" >&2
AUTH_RESPONSE=$(curl -s -X POST "${ODOO_URL}/jsonrpc" \
  -H 'Content-Type: application/json' \
  -d "{
    \"jsonrpc\": \"2.0\",
    \"method\": \"call\",
    \"params\": {
        \"service\": \"common\",
        \"method\": \"authenticate\",
        \"args\": [\"${ODOO_DB}\", \"${ODOO_USER}\", \"${ODOO_KEY}\", {}]
    },
    \"id\": 1
  }" 2>&1)

# Check for errors
if echo "$AUTH_RESPONSE" | grep -q '"error"'; then
    ERROR_MSG=$(echo "$AUTH_RESPONSE" | grep -o '"message":"[^"]*"' | head -1 | cut -d'"' -f4)
    echo -e "${RED}Authentication failed: ${ERROR_MSG}${NC}" >&2
    exit 1
fi

# Extract USER_ID
USER_ID=$(echo "$AUTH_RESPONSE" | grep -o '"result":[[:space:]]*[0-9]*' | grep -o '[0-9]*')

if [[ -z "$USER_ID" ]] || [[ "$USER_ID" == "false" ]]; then
    echo -e "${RED}Authentication failed: Invalid credentials${NC}" >&2
    exit 1
fi

echo -e "${GREEN}✓ Authenticated successfully (API key)${NC}" >&2
echo "User ID: ${USER_ID}" >&2
echo "Odoo Version: ${VERSION}" >&2
echo "Note: Using API key - no session cookie available" >&2

# Output environment variables to stdout (for eval)
echo "export ODOO_SESSION_ID=''"
echo "export ODOO_UID='${USER_ID}'"
echo "export ODOO_VERSION='${VERSION}'"
