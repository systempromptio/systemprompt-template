#!/bin/bash
# install_module.sh - Install an Odoo module
# Usage: ./install_module.sh <module_name>
# Requires: ODOO_URL, ODOO_DB, ODOO_USER, ODOO_KEY, ODOO_UID, ODOO_SESSION_ID (for JSON-RPC)

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Check arguments
if [[ $# -lt 1 ]]; then
    echo -e "${RED}Error: Module name required${NC}"
    echo "Usage: $0 <module_name>"
    echo "Example: $0 sale_management"
    exit 1
fi

MODULE_NAME="$1"

# Check required environment variables
if [[ -z "${ODOO_URL:-}" ]] || [[ -z "${ODOO_DB:-}" ]] || [[ -z "${ODOO_USER:-}" ]] || [[ -z "${ODOO_KEY:-}" ]]; then
    echo -e "${RED}Error: Missing required environment variables${NC}"
    echo "Required: ODOO_URL, ODOO_DB, ODOO_USER, ODOO_KEY"
    exit 1
fi

echo -e "${YELLOW}Installing module: ${MODULE_NAME}${NC}"

# Detect protocol
PROTOCOL="${ODOO_PROTOCOL:-jsonrpc}"

if [[ "$PROTOCOL" == "json2" ]]; then
    # JSON2 Protocol (Odoo >= 19.0) - Uses Bearer token authentication
    if [[ -z "${ODOO_KEY:-}" ]]; then
        echo -e "${RED}Error: API Key required for JSON2. Run auth_json2.sh first${NC}" >&2
        exit 1
    fi

    # First, search for the module
    echo -e "${YELLOW}Searching for module...${NC}" >&2
    SEARCH_RESPONSE=$(curl -s -X POST "${ODOO_URL}/json/2/ir.module.module/search_read" \
      -H 'Content-Type: application/json' \
      -H "Authorization: bearer ${ODOO_KEY}" \
      -H "X-Odoo-Database: ${ODOO_DB}" \
      -d "{
        \"domain\": [[\"name\", \"=\", \"${MODULE_NAME}\"]],
        \"fields\": [\"id\", \"name\", \"state\"],
        \"limit\": 1
      }")

    # JSON2 returns array directly
    MODULE_ID=$(echo "$SEARCH_RESPONSE" | grep -o '"id":[[:space:]]*[0-9]*' | head -1 | grep -o '[0-9]*')

    if [[ -z "$MODULE_ID" ]]; then
        echo -e "${RED}Module not found: ${MODULE_NAME}${NC}" >&2
        exit 1
    fi

    # Call button_immediate_install on the module
    # JSON2 format: parameters go directly in the body, not wrapped in args/kwargs
    RESPONSE=$(curl -s -X POST "${ODOO_URL}/json/2/ir.module.module/button_immediate_install" \
      -H 'Content-Type: application/json' \
      -H "Authorization: bearer ${ODOO_KEY}" \
      -H "X-Odoo-Database: ${ODOO_DB}" \
      -d "{
        \"ids\": [${MODULE_ID}]
      }")
else
    # JSON-RPC Protocol (Odoo < 19.0)
    if [[ -z "${ODOO_UID:-}" ]]; then
        echo -e "${RED}Error: Not authenticated. Run auth_jsonrpc.sh first${NC}"
        exit 1
    fi

    # First, find the module ID
    SEARCH_RESPONSE=$(curl -s -X POST "${ODOO_URL}/jsonrpc" \
      -H 'Content-Type: application/json' \
      -H "Cookie: session_id=${ODOO_SESSION_ID:-}" \
      -d "{
        \"jsonrpc\": \"2.0\",
        \"method\": \"call\",
        \"params\": {
            \"service\": \"object\",
            \"method\": \"execute_kw\",
            \"args\": [
                \"${ODOO_DB}\",
                ${ODOO_UID},
                \"${ODOO_KEY}\",
                \"ir.module.module\",
                \"search\",
                [[[\"name\", \"=\", \"${MODULE_NAME}\"]]],
                {\"limit\": 1}
            ]
        },
        \"id\": null
      }")

    MODULE_ID=$(echo "$SEARCH_RESPONSE" | grep -o '"result"[[:space:]]*:[[:space:]]*\[[0-9]*\]' | grep -o '[0-9]*' | head -1 || echo "")

    if [[ -z "$MODULE_ID" ]]; then
        echo -e "${RED}Error: Module '${MODULE_NAME}' not found${NC}"
        exit 1
    fi

    echo "Found module ID: ${MODULE_ID}"

    # Install the module
    RESPONSE=$(curl -s -X POST "${ODOO_URL}/jsonrpc" \
      -H 'Content-Type: application/json' \
      -H "Cookie: session_id=${ODOO_SESSION_ID:-}" \
      -d "{
        \"jsonrpc\": \"2.0\",
        \"method\": \"call\",
        \"params\": {
            \"service\": \"object\",
            \"method\": \"execute_kw\",
            \"args\": [
                \"${ODOO_DB}\",
                ${ODOO_UID},
                \"${ODOO_KEY}\",
                \"ir.module.module\",
                \"button_immediate_install\",
                [[${MODULE_ID}]]
            ]
        },
        \"id\": null
      }")
fi

# Check for errors
if echo "$RESPONSE" | grep -q '"error"'; then
    ERROR_MSG=$(echo "$RESPONSE" | grep -o '"message":"[^"]*"' | head -1 | cut -d'"' -f4)
    echo -e "${RED}Installation failed: ${ERROR_MSG}${NC}"
    exit 1
fi

echo -e "${GREEN}✓ Module '${MODULE_NAME}' installed successfully${NC}"
