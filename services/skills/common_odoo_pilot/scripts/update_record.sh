#!/bin/bash
# update_record.sh - Update record(s) in an Odoo model
# Usage: ./update_record.sh <model> <record_ids_json> <values_json>
# Example: ./update_record.sh res.partner '[123,124]' '{"phone":"+34612345678"}'
# Requires: ODOO_URL, ODOO_DB, ODOO_USER, ODOO_KEY, ODOO_UID, ODOO_SESSION_ID (for JSON-RPC)

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Check arguments
if [[ $# -lt 3 ]]; then
    echo -e "${RED}Error: Model, record IDs, and values required${NC}"
    echo "Usage: $0 <model> <record_ids_json> <values_json>"
    echo ""
    echo "Examples:"
    echo "  $0 res.partner '[123]' '{\"phone\":\"+34612345678\"}'"
    echo "  $0 product.product '[1,2,3]' '{\"list_price\":150.00}'"
    exit 1
fi

MODEL="$1"
RECORD_IDS="$2"
VALUES="$3"

# Check required environment variables
if [[ -z "${ODOO_URL:-}" ]] || [[ -z "${ODOO_DB:-}" ]] || [[ -z "${ODOO_USER:-}" ]] || [[ -z "${ODOO_KEY:-}" ]]; then
    echo -e "${RED}Error: Missing required environment variables${NC}"
    echo "Required: ODOO_URL, ODOO_DB, ODOO_USER, ODOO_KEY"
    exit 1
fi

echo -e "${YELLOW}Updating record(s) in model: ${MODEL}${NC}" >&2
echo "Record IDs: ${RECORD_IDS}" >&2
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

    # JSON2's write expects 'vals' not 'values'
    RESPONSE=$(curl -s -X POST "${ODOO_URL}/json/2/${MODEL}/write" \
      -H 'Content-Type: application/json' \
      -H "Authorization: bearer ${ODOO_KEY}" \
      -H "X-Odoo-Database: ${ODOO_DB}" \
      -d "{
        \"ids\": ${RECORD_IDS},
        \"vals\": ${VALUES}
      }")
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
                \"write\",
                [${RECORD_IDS}, ${VALUES}]
            ]
        },
        \"id\": null
      }")
fi

# Check for errors
if echo "$RESPONSE" | grep -q '"error"'; then
    ERROR_MSG=$(echo "$RESPONSE" | grep -o '"message":"[^"]*"' | head -1 | cut -d'"' -f4)
    echo -e "${RED}Update failed: ${ERROR_MSG}${NC}" >&2
    exit 1
fi

# Extract result based on protocol (should be true)
if [[ "$PROTOCOL" == "json2" ]]; then
    # JSON2 returns data directly
    RESULT="$RESPONSE"
else
    # JSON-RPC wraps data in "result" field
    RESULT=$(echo "$RESPONSE" | grep -o '"result"[[:space:]]*:[[:space:]]*[a-z]*' | sed 's/.*:[[:space:]]*//')
fi

if [[ "$RESULT" == "true" ]]; then
    COUNT=$(echo "$RECORD_IDS" | grep -o '[0-9]*' | wc -l | tr -d ' ')
    echo -e "${GREEN}✓ Updated ${COUNT} record(s) successfully${NC}" >&2
    echo "true"
    exit 0
else
    echo -e "${RED}Update failed: unexpected response${NC}" >&2
    exit 1
fi
