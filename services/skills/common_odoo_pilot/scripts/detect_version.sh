#!/bin/bash
# detect_version.sh - Auto-detect Odoo version and protocol
# Usage: ./detect_version.sh
# Requires: ODOO_URL, ODOO_DB, ODOO_USER, ODOO_KEY environment variables

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check required environment variables
if [[ -z "${ODOO_URL:-}" ]] || [[ -z "${ODOO_DB:-}" ]] || [[ -z "${ODOO_USER:-}" ]] || [[ -z "${ODOO_KEY:-}" ]]; then
    echo -e "${RED}Error: Missing required environment variables${NC}"
    echo "Required: ODOO_URL, ODOO_DB, ODOO_USER, ODOO_KEY"
    echo ""
    echo "Example:"
    echo "  export ODOO_URL='https://demo.odoo.com'"
    echo "  export ODOO_DB='demo'"
    echo "  export ODOO_USER='admin'"
    echo "  export ODOO_KEY='your-api-key'"
    exit 1
fi

echo -e "${YELLOW}Detecting Odoo version...${NC}"
echo "URL: ${ODOO_URL}"
echo "Database: ${ODOO_DB}"
echo ""

# Try JSON2 protocol first (Odoo >= 19.0)
echo -e "${YELLOW}Attempting JSON2 protocol (Odoo >= 19.0)...${NC}"
JSON2_RESPONSE=$(curl -s -w "\nHTTP_CODE:%{http_code}" -X POST "${ODOO_URL}/json/2/call" \
  -H 'Content-Type: application/json' \
  -d "{
    \"jsonrpc\": \"2.0\",
    \"method\": \"call\",
    \"params\": {
        \"context\": {
            \"db\": \"${ODOO_DB}\",
            \"login\": \"${ODOO_USER}\",
            \"password\": \"${ODOO_KEY}\"
        }
    },
    \"id\": 1
  }" 2>&1 || true)

HTTP_CODE=$(echo "$JSON2_RESPONSE" | grep "HTTP_CODE:" | cut -d':' -f2)
RESPONSE_BODY=$(echo "$JSON2_RESPONSE" | grep -v "HTTP_CODE:")

if [[ "$HTTP_CODE" == "200" ]]; then
    # Check if response contains error
    if echo "$RESPONSE_BODY" | grep -q '"error"'; then
        echo -e "${YELLOW}JSON2 endpoint exists but authentication failed${NC}"
        ERROR_MSG=$(echo "$RESPONSE_BODY" | grep -o '"message":"[^"]*"' | head -1 | cut -d'"' -f4)
        echo -e "${RED}Error: ${ERROR_MSG}${NC}"
        echo ""
        echo "Trying JSON-RPC protocol..."
    else
        # Extract version from response
        VERSION=$(echo "$RESPONSE_BODY" | grep -o '"server_version":"[^"]*"' | cut -d'"' -f4 || echo "unknown")
        echo -e "${GREEN}✓ Detected Odoo ${VERSION} (JSON2 protocol)${NC}"
        echo ""
        echo "Protocol: JSON2"
        echo "Endpoint: ${ODOO_URL}/json/2/call"
        echo "Version: ${VERSION}"
        exit 0
    fi
fi

# Try JSON-RPC protocol (Odoo < 19.0)
echo -e "${YELLOW}Attempting JSON-RPC protocol (Odoo < 19.0)...${NC}"
JSONRPC_RESPONSE=$(curl -s -w "\nHTTP_CODE:%{http_code}" -X POST "${ODOO_URL}/web/session/authenticate" \
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
  }" 2>&1 || true)

HTTP_CODE=$(echo "$JSONRPC_RESPONSE" | grep "HTTP_CODE:" | cut -d':' -f2)
RESPONSE_BODY=$(echo "$JSONRPC_RESPONSE" | grep -v "HTTP_CODE:")

if [[ "$HTTP_CODE" == "200" ]]; then
    # Check for error in response
    if echo "$RESPONSE_BODY" | grep -q '"error"'; then
        ERROR_MSG=$(echo "$RESPONSE_BODY" | grep -o '"message":"[^"]*"' | head -1 | cut -d'"' -f4)
        echo -e "${RED}✗ Authentication failed: ${ERROR_MSG}${NC}"
        exit 1
    fi

    # Extract version and uid from response
    VERSION=$(echo "$RESPONSE_BODY" | grep -o '"server_version":"[^"]*"' | cut -d'"' -f4 || echo "unknown")
    UID=$(echo "$RESPONSE_BODY" | grep -o '"uid":[0-9]*' | grep -o '[0-9]*' || echo "unknown")

    if [[ "$UID" != "false" ]] && [[ "$UID" != "unknown" ]]; then
        echo -e "${GREEN}✓ Detected Odoo ${VERSION} (JSON-RPC protocol)${NC}"
        echo ""
        echo "Protocol: JSON-RPC"
        echo "Endpoint: ${ODOO_URL}/jsonrpc"
        echo "Auth Endpoint: ${ODOO_URL}/web/session/authenticate"
        echo "Version: ${VERSION}"
        echo "User ID: ${UID}"
        exit 0
    fi
fi

# If both failed
echo -e "${RED}✗ Could not detect Odoo version${NC}"
echo ""
echo "Possible causes:"
echo "  - Invalid credentials"
echo "  - Wrong URL or database name"
echo "  - Network connectivity issues"
echo "  - Odoo instance is down"
exit 1
