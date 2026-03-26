#!/bin/bash
# execute_method.sh - Execute any method on an Odoo model
# Usage: ./execute_method.sh <model> <method> <args_json> [kwargs_json]
# Example: ./execute_method.sh res.partner name_get '[[123,124]]' '{}'
# Requires: ODOO_URL, ODOO_DB, ODOO_USER, ODOO_KEY, ODOO_UID, ODOO_SESSION_ID (for JSON-RPC)

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Check arguments
if [[ $# -lt 3 ]]; then
    echo -e "${RED}Error: Model, method, and args required${NC}"
    echo "Usage: $0 <model> <method> <args_json> [kwargs_json]"
    echo ""
    echo "Examples:"
    echo "  $0 res.partner name_get '[[123,124]]' '{}'"
    echo "  $0 sale.order action_confirm '[[5,6]]'"
    echo "  $0 ir.config_parameter get_param '[\"web.base.url\"]'"
    exit 1
fi

MODEL="$1"
METHOD="$2"
ARGS="$3"
KWARGS="$4"
if [[ -z "$KWARGS" ]]; then
    KWARGS="{}"
fi

# Check required environment variables
if [[ -z "${ODOO_URL:-}" ]] || [[ -z "${ODOO_DB:-}" ]] || [[ -z "${ODOO_USER:-}" ]] || [[ -z "${ODOO_KEY:-}" ]]; then
    echo -e "${RED}Error: Missing required environment variables${NC}"
    echo "Required: ODOO_URL, ODOO_DB, ODOO_USER, ODOO_KEY"
    exit 1
fi

echo -e "${YELLOW}Executing method '${METHOD}' on model: ${MODEL}${NC}" >&2
echo "Args: ${ARGS}" >&2
echo "Kwargs: ${KWARGS}" >&2
echo "" >&2

# Detect protocol
PROTOCOL="${ODOO_PROTOCOL:-jsonrpc}"

if [[ "$PROTOCOL" == "json2" ]]; then
    # JSON2 Protocol (Odoo >= 19.0) - Uses Bearer token authentication
    if [[ -z "${ODOO_KEY:-}" ]]; then
        echo -e "${RED}Error: API Key required for JSON2. Run auth_json2.sh first${NC}" >&2
        exit 1
    fi

    # JSON2 uses model/method in URL, parameters directly in body (no args/kwargs wrapper)
    # For JSON2, we need to convert [[ids]] format to {"ids": [ids]} format

    # Check if ARGS contains IDs array format like [[1,2,3]]
    if [[ "$ARGS" =~ ^\[\[([0-9,]+)\]\]$ ]]; then
        # Extract IDs from [[ids]] format
        IDS="${BASH_REMATCH[1]}"
        BODY="{\"ids\": [${IDS}]"

        # Merge with kwargs if provided
        if [[ "$KWARGS" != "{}" ]]; then
            # Remove opening and closing braces from KWARGS
            KWARGS_CONTENT="${KWARGS#\{}"
            KWARGS_CONTENT="${KWARGS_CONTENT%\}}"
            if [[ -n "$KWARGS_CONTENT" ]]; then
                BODY="${BODY}, ${KWARGS_CONTENT}}"
            else
                BODY="${BODY}}"
            fi
        else
            BODY="${BODY}}"
        fi

        RESPONSE=$(curl -s -X POST "${ODOO_URL}/json/2/${MODEL}/${METHOD}" \
          -H 'Content-Type: application/json' \
          -H "Authorization: bearer ${ODOO_KEY}" \
          -H "X-Odoo-Database: ${ODOO_DB}" \
          -d "${BODY}")
    elif [[ "$KWARGS" != "{}" ]]; then
        # Use kwargs directly (most common for JSON2)
        RESPONSE=$(curl -s -X POST "${ODOO_URL}/json/2/${MODEL}/${METHOD}" \
          -H 'Content-Type: application/json' \
          -H "Authorization: bearer ${ODOO_KEY}" \
          -H "X-Odoo-Database: ${ODOO_DB}" \
          -d "${KWARGS}")
    else
        # Empty body for methods with no parameters
        RESPONSE=$(curl -s -X POST "${ODOO_URL}/json/2/${MODEL}/${METHOD}" \
          -H 'Content-Type: application/json' \
          -H "Authorization: bearer ${ODOO_KEY}" \
          -H "X-Odoo-Database: ${ODOO_DB}" \
          -d "{}")
    fi
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
                \"${METHOD}\",
                ${ARGS},
                ${KWARGS}
            ]
        },
        \"id\": null
      }")
fi

# Check for errors
if echo "$RESPONSE" | grep -q '"error"'; then
    ERROR_MSG=$(echo "$RESPONSE" | grep -o '"message":"[^"]*"' | head -1 | cut -d'"' -f4)
    echo -e "${RED}Method execution failed: ${ERROR_MSG}${NC}" >&2
    exit 1
fi

# Extract result based on protocol
if [[ "$PROTOCOL" == "json2" ]]; then
    # JSON2 returns data directly
    RESULT="$RESPONSE"
else
    # JSON-RPC wraps data in "result" field
    # Extract everything after "result": and before ", "id":" or final "}"
    RESULT=$(echo "$RESPONSE" | sed -n 's/.*"result":[[:space:]]*//p' | sed 's/,[[:space:]]*"id".*$//' | sed 's/}$//')
fi

if [[ -z "$RESULT" ]]; then
    echo -e "${YELLOW}Method executed but returned no result${NC}" >&2
    echo "null"
    exit 0
fi

echo -e "${GREEN}✓ Method executed successfully${NC}" >&2

# Output result to stdout
echo "$RESULT"
