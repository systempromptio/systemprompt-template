#!/bin/bash
# create_record.sh - Create a record in an Odoo model
# Usage: ./create_record.sh <model> <values_json>
# Example: ./create_record.sh res.partner '{"name":"Test Company","email":"test@example.com","is_company":true}'
# Requires: ODOO_URL, ODOO_DB, ODOO_USER, ODOO_KEY, ODOO_UID, ODOO_SESSION_ID (for JSON-RPC)

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Check arguments
if [[ $# -lt 2 ]]; then
    echo -e "${RED}Error: Model and values required${NC}"
    echo "Usage: $0 <model> <values_json>"
    echo ""
    echo "Examples:"
    echo "  $0 res.partner '{\"name\":\"John Doe\",\"email\":\"john@example.com\"}'"
    echo "  $0 product.product '{\"name\":\"New Product\",\"list_price\":99.99}'"
    exit 1
fi

MODEL="$1"
VALUES="$2"

# Check required environment variables
if [[ -z "${ODOO_URL:-}" ]] || [[ -z "${ODOO_DB:-}" ]] || [[ -z "${ODOO_USER:-}" ]] || [[ -z "${ODOO_KEY:-}" ]]; then
    echo -e "${RED}Error: Missing required environment variables${NC}"
    echo "Required: ODOO_URL, ODOO_DB, ODOO_USER, ODOO_KEY"
    exit 1
fi

echo -e "${YELLOW}Creating record in model: ${MODEL}${NC}" >&2
echo "Values: ${VALUES}" >&2
echo "" >&2

# Detect protocol
PROTOCOL="${ODOO_PROTOCOL:-jsonrpc}"

if [[ "$PROTOCOL" == "json2" ]]; then
    # JSON2 Protocol (Odoo >= 19.0) - Uses Bearer token authentication
    if [[ -z "${ODOO_KEY:-}" ]]; then
        echo -e "${RED}Error: API Key required for JSON2. Run auth_json2.sh first${NC}" >&2
        exit 1
    fi

    # JSON2's create expects vals_list (array of dicts), not a single dict
    RESPONSE=$(curl -s -X POST "${ODOO_URL}/json/2/${MODEL}/create" \
      -H 'Content-Type: application/json' \
      -H "Authorization: bearer ${ODOO_KEY}" \
      -H "X-Odoo-Database: ${ODOO_DB}" \
      -d "{\"vals_list\": [${VALUES}]}")
else
    # JSON-RPC Protocol (Odoo < 19.0)
    if [[ -z "${ODOO_UID:-}" ]]; then
        echo -e "${RED}Error: Not authenticated. Run auth_jsonrpc.sh first${NC}" >&2
        exit 1
    fi

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
                \"${MODEL}\",
                \"create\",
                [${VALUES}]
            ]
        },
        \"id\": null
      }")
fi

# Check for errors
if echo "$RESPONSE" | grep -q '"error"'; then
    ERROR_MSG=$(echo "$RESPONSE" | grep -o '"message":"[^"]*"' | head -1 | cut -d'"' -f4)
    echo -e "${RED}Creation failed: ${ERROR_MSG}${NC}" >&2
    exit 1
fi

# Extract result (new record ID) based on protocol
if [[ "$PROTOCOL" == "json2" ]]; then
    # JSON2 returns array of IDs: [123] or [123, 124]
    RECORD_ID=$(echo "$RESPONSE" | grep -o '\[[0-9, ]*\]' | grep -o '[0-9]*' | head -1)
else
    # JSON-RPC wraps result: {"result": 123}
    RECORD_ID=$(echo "$RESPONSE" | grep -o '"result"[[:space:]]*:[[:space:]]*[0-9]*' | grep -o '[0-9]*')
fi

if [[ -z "$RECORD_ID" ]]; then
    echo -e "${RED}Failed to extract record ID from response${NC}" >&2
    echo "Response was: $RESPONSE" >&2
    exit 1
fi

echo -e "${GREEN}✓ Record created successfully${NC}" >&2
echo "Record ID: ${RECORD_ID}" >&2

# Output record ID to stdout
echo "$RECORD_ID"
