#!/bin/bash
# auth.sh - Unified authentication script for Odoo (supports both JSON2 and JSON-RPC)
# Usage: ./auth.sh [protocol]
# Parameters:
#   protocol: Optional. "json2" or "jsonrpc". If not provided, auto-detects based on version.
# Requires: ODOO_URL, ODOO_DB, ODOO_KEY (and ODOO_USER for JSON-RPC)
# Output: Prints environment variables to stdout (use with eval)
#
# Auto-detection logic:
#   1. If protocol parameter provided, use it
#   2. Try JSON2 first (Odoo >= 19.0)
#   3. If JSON2 fails with 404, fallback to JSON-RPC
#   4. If ODOO_USER is missing and JSON2 fails, require ODOO_USER for JSON-RPC

set -euo pipefail

# Colors for output (to stderr)
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Parse optional protocol argument
FORCE_PROTOCOL="${1:-}"

# Check required base environment variables
if [[ -z "${ODOO_URL:-}" ]] || [[ -z "${ODOO_DB:-}" ]] || [[ -z "${ODOO_KEY:-}" ]]; then
    echo -e "${RED}Error: Missing required environment variables${NC}" >&2
    echo "Required: ODOO_URL, ODOO_DB, ODOO_KEY" >&2
    echo "Optional: ODOO_USER (required for JSON-RPC)" >&2
    exit 1
fi

# Function to try JSON2 authentication
try_json2_auth() {
    echo -e "${BLUE}Trying JSON2 authentication (Odoo >= 19.0)...${NC}" >&2

    RESPONSE=$(curl -s -X POST "${ODOO_URL}/json/2/res.users/search_read" \
      -H "Content-Type: application/json" \
      -H "Authorization: bearer ${ODOO_KEY}" \
      -H "X-Odoo-Database: ${ODOO_DB}" \
      -d '{
        "domain": [],
        "fields": ["id", "name", "login"],
        "limit": 1
      }' 2>&1)

    # Check for 404 (endpoint not available)
    if echo "$RESPONSE" | grep -qi "not found\|404"; then
        echo -e "${YELLOW}JSON2 endpoint not available (404)${NC}" >&2
        return 1
    fi

    # Check for errors
    if echo "$RESPONSE" | grep -q '"error"'; then
        ERROR_MSG=$(echo "$RESPONSE" | grep -o '"message":"[^"]*"' | head -1 | cut -d'"' -f4)
        echo -e "${YELLOW}JSON2 authentication failed: ${ERROR_MSG}${NC}" >&2
        return 1
    fi

    # Parse response - JSON2 returns array directly
    ODOO_UID=$(echo "$RESPONSE" | grep -o '"id":[[:space:]]*[0-9]*' | head -1 | grep -o '[0-9]*')
    USERNAME=$(echo "$RESPONSE" | grep -o '"name":"[^"]*"' | head -1 | cut -d'"' -f4 || echo "unknown")
    LOGIN=$(echo "$RESPONSE" | grep -o '"login":"[^"]*"' | head -1 | cut -d'"' -f4 || echo "unknown")

    # Check if authentication was successful
    if [[ -z "$ODOO_UID" ]]; then
        echo -e "${YELLOW}Could not retrieve user ID from JSON2${NC}" >&2
        return 1
    fi

    # Try to get Odoo version (may not work via JSON2, fallback to default)
    VERSION="19.0+"  # Default for JSON2

    # Output results to stderr for user feedback
    echo -e "${GREEN}✓ Authenticated successfully via JSON2 (Bearer token)${NC}" >&2
    echo "Protocol: JSON2" >&2
    echo "User ID: ${ODOO_UID}" >&2
    echo "Username: ${USERNAME}" >&2
    echo "Login: ${LOGIN}" >&2
    echo "Odoo Version: ${VERSION}" >&2
    echo "Database: ${ODOO_DB}" >&2
    echo "Note: Using API key authentication (stateless)" >&2

    # Output environment variables to stdout (for eval)
    echo "export ODOO_UID='${ODOO_UID}'"
    echo "export ODOO_VERSION='${VERSION}'"
    echo "export ODOO_PROTOCOL='json2'"
    echo "export ODOO_AUTH_HEADER='Authorization: bearer ${ODOO_KEY}'"

    return 0
}

# Function to try JSON-RPC authentication
try_jsonrpc_auth() {
    echo -e "${BLUE}Trying JSON-RPC authentication (Odoo < 19.0)...${NC}" >&2

    # Check if ODOO_USER is set for JSON-RPC
    if [[ -z "${ODOO_USER:-}" ]]; then
        echo -e "${RED}Error: ODOO_USER required for JSON-RPC authentication${NC}" >&2
        echo "Please set: export ODOO_USER=\"your-username\"" >&2
        return 1
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
            echo -e "${GREEN}✓ Authenticated successfully via JSON-RPC (session)${NC}" >&2
            echo "Protocol: JSON-RPC" >&2
            echo "User ID: ${USER_ID}" >&2
            echo "Session ID: ${SESSION_ID:0:20}..." >&2
            echo "Odoo Version: ${VERSION}" >&2
            echo "Database: ${ODOO_DB}" >&2

            echo "export ODOO_SESSION_ID='${SESSION_ID}'"
            echo "export ODOO_UID='${USER_ID}'"
            echo "export ODOO_VERSION='${VERSION}'"
            echo "export ODOO_PROTOCOL='jsonrpc'"
            return 0
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
        echo -e "${RED}JSON-RPC authentication failed: ${ERROR_MSG}${NC}" >&2
        return 1
    fi

    # Extract USER_ID
    USER_ID=$(echo "$AUTH_RESPONSE" | grep -o '"result":[[:space:]]*[0-9]*' | grep -o '[0-9]*')

    if [[ -z "$USER_ID" ]] || [[ "$USER_ID" == "false" ]]; then
        echo -e "${RED}JSON-RPC authentication failed: Invalid credentials${NC}" >&2
        return 1
    fi

    echo -e "${GREEN}✓ Authenticated successfully via JSON-RPC (API key)${NC}" >&2
    echo "Protocol: JSON-RPC" >&2
    echo "User ID: ${USER_ID}" >&2
    echo "Odoo Version: ${VERSION}" >&2
    echo "Database: ${ODOO_DB}" >&2
    echo "Note: Using API key - no session cookie available" >&2

    # Output environment variables to stdout (for eval)
    echo "export ODOO_SESSION_ID=''"
    echo "export ODOO_UID='${USER_ID}'"
    echo "export ODOO_VERSION='${VERSION}'"
    echo "export ODOO_PROTOCOL='jsonrpc'"

    return 0
}

# Main authentication logic
if [[ -n "$FORCE_PROTOCOL" ]]; then
    # User specified protocol
    if [[ "$FORCE_PROTOCOL" == "json2" ]]; then
        if try_json2_auth; then
            exit 0
        else
            echo -e "${RED}Failed to authenticate with JSON2${NC}" >&2
            exit 1
        fi
    elif [[ "$FORCE_PROTOCOL" == "jsonrpc" ]]; then
        if try_jsonrpc_auth; then
            exit 0
        else
            echo -e "${RED}Failed to authenticate with JSON-RPC${NC}" >&2
            exit 1
        fi
    else
        echo -e "${RED}Error: Invalid protocol '${FORCE_PROTOCOL}'${NC}" >&2
        echo "Valid protocols: json2, jsonrpc" >&2
        exit 1
    fi
else
    # Auto-detect: Try JSON2 first, fallback to JSON-RPC
    echo -e "${BLUE}Auto-detecting authentication protocol...${NC}" >&2

    if try_json2_auth; then
        exit 0
    fi

    echo -e "${YELLOW}Falling back to JSON-RPC...${NC}" >&2

    if try_jsonrpc_auth; then
        exit 0
    fi

    # Both methods failed
    echo -e "${RED}Authentication failed with both JSON2 and JSON-RPC${NC}" >&2
    echo "" >&2
    echo "Troubleshooting:" >&2
    echo "1. Verify ODOO_URL is correct" >&2
    echo "2. Verify ODOO_DB exists" >&2
    echo "3. Verify ODOO_KEY is valid (generate from: Preferences > Account Security > API Keys)" >&2
    echo "4. For JSON-RPC, ensure ODOO_USER is set" >&2
    echo "5. Check if your Odoo instance is accessible" >&2
    exit 1
fi
