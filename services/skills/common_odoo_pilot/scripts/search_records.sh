#!/bin/bash
# search_records.sh - Search records in an Odoo model
# Usage: ./search_records.sh <model> <domain_json> [fields_json] [limit] [offset]
# Example: ./search_records.sh res.partner '[["is_company","=",true]]' '["name","email"]' 10 0
# Requires: ODOO_URL, ODOO_DB, ODOO_USER, ODOO_KEY, ODOO_UID, ODOO_SESSION_ID (for JSON-RPC)

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Check arguments
if [[ $# -lt 2 ]]; then
    echo -e "${RED}Error: Model and domain required${NC}"
    echo "Usage: $0 <model> <domain_json> [fields_json] [limit] [offset]"
    echo ""
    echo "Examples:"
    echo "  $0 res.partner '[[\"is_company\",\"=\",true]]' '[\"name\",\"email\"]' 5"
    echo "  $0 sale.order '[[\"state\",\"=\",\"sale\"]]'"
    echo "  $0 product.product '[]' '[\"name\",\"list_price\"]' 10 0"
    exit 1
fi

MODEL="$1"
DOMAIN="$2"
FIELDS="${3:-[]}"
LIMIT="${4:-100}"
OFFSET="${5:-0}"

# Check required environment variables
if [[ -z "${ODOO_URL:-}" ]] || [[ -z "${ODOO_DB:-}" ]] || [[ -z "${ODOO_USER:-}" ]] || [[ -z "${ODOO_KEY:-}" ]]; then
    echo -e "${RED}Error: Missing required environment variables${NC}"
    echo "Required: ODOO_URL, ODOO_DB, ODOO_USER, ODOO_KEY"
    exit 1
fi

echo -e "${YELLOW}Searching records in model: ${MODEL}${NC}" >&2
echo "Domain: ${DOMAIN}" >&2
echo "Fields: ${FIELDS}" >&2
echo "Limit: ${LIMIT}, Offset: ${OFFSET}" >&2
echo "" >&2

# Detect protocol
PROTOCOL="${ODOO_PROTOCOL:-jsonrpc}"

if [[ "$PROTOCOL" == "json2" ]]; then
    # JSON2 Protocol (Odoo >= 19.0) - Uses Bearer token authentication
    if [[ -z "${ODOO_KEY:-}" ]]; then
        echo -e "${RED}Error: API Key required for JSON2. Run auth_json2.sh first${NC}" >&2
        exit 1
    fi

    RESPONSE=$(curl -s -X POST "${ODOO_URL}/json/2/${MODEL}/search_read" \
      -H 'Content-Type: application/json' \
      -H "Authorization: bearer ${ODOO_KEY}" \
      -H "X-Odoo-Database: ${ODOO_DB}" \
      -d "{
        \"domain\": ${DOMAIN},
        \"fields\": ${FIELDS},
        \"limit\": ${LIMIT},
        \"offset\": ${OFFSET}
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
                \"search_read\",
                [${DOMAIN}],
                {
                    \"fields\": ${FIELDS},
                    \"limit\": ${LIMIT},
                    \"offset\": ${OFFSET}
                }
            ]
        },
        \"id\": null
      }")
fi

# Check for errors
if echo "$RESPONSE" | grep -q '"error"'; then
    ERROR_MSG=$(echo "$RESPONSE" | grep -o '"message":"[^"]*"' | head -1 | cut -d'"' -f4)
    echo -e "${RED}Search failed: ${ERROR_MSG}${NC}" >&2
    exit 1
fi

# Extract result based on protocol
if [[ "$PROTOCOL" == "json2" ]]; then
    # JSON2 returns data directly (no wrapper)
    RESULT="$RESPONSE"
else
    # JSON-RPC wraps data in "result" field
    RESULT=$(echo "$RESPONSE" | grep -o '"result"[[:space:]]*:[[:space:]]*\[.*\]' | sed 's/"result"[[:space:]]*:[[:space:]]*//')
fi

if [[ -z "$RESULT" ]] || [[ "$RESULT" == "[]" ]]; then
    echo -e "${YELLOW}No records found${NC}" >&2
    echo "[]"
    exit 0
fi

# Count results
COUNT=$(echo "$RESULT" | grep -o '"id":[0-9]*' | wc -l | tr -d ' ')
echo -e "${GREEN}✓ Found ${COUNT} record(s)${NC}" >&2

# Output result as JSON to stdout
echo "$RESULT"
