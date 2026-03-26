#!/bin/bash
# auth_json2.sh - Authenticate with Odoo using JSON2 protocol (>= v19)
# Usage: ./auth_json2.sh
# Requires: ODOO_URL, ODOO_DB, ODOO_KEY environment variables
# Output: Prints uid and version to stdout (use with eval)
# Note: JSON2 uses Bearer token authentication (API Key in Authorization header)
#       Endpoint format: /json/2/{model}/{method}

set -euo pipefail

# Colors for output (to stderr)
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Check required environment variables
if [[ -z "${ODOO_URL:-}" ]] || [[ -z "${ODOO_DB:-}" ]] || [[ -z "${ODOO_KEY:-}" ]]; then
    echo -e "${RED}Error: Missing required environment variables${NC}" >&2
    echo "Required: ODOO_URL, ODOO_DB, ODOO_KEY" >&2
    echo "Note: ODOO_USER is not required for JSON2 (uses API key only)" >&2
    exit 1
fi

# Test authentication by calling search_read on res.users
# JSON2 format: POST /json/2/{model}/{method}
# Bearer token in Authorization header
# Database in X-Odoo-Database header
echo -e "${YELLOW}Authenticating with JSON2 API (Bearer token)...${NC}" >&2

RESPONSE=$(curl -s -X POST "${ODOO_URL}/json/2/res.users/search_read" \
  -H "Content-Type: application/json" \
  -H "Authorization: bearer ${ODOO_KEY}" \
  -H "X-Odoo-Database: ${ODOO_DB}" \
  -d '{
    "domain": [],
    "fields": ["id", "name", "login"],
    "limit": 1
  }' 2>&1)

# Check for errors
if echo "$RESPONSE" | grep -q '"error"'; then
    ERROR_MSG=$(echo "$RESPONSE" | grep -o '"message":"[^"]*"' | head -1 | cut -d'"' -f4)
    echo -e "${RED}Authentication failed: ${ERROR_MSG}${NC}" >&2
    echo -e "${YELLOW}Tip: Make sure you created an API key via: Preferences > Account Security > New API Key${NC}" >&2
    exit 1
fi

# Check if we got a 404 (endpoint not available)
if echo "$RESPONSE" | grep -qi "not found\|404"; then
    echo -e "${RED}JSON2 endpoint not available${NC}" >&2
    echo -e "${YELLOW}Your Odoo instance may not have JSON2 enabled. Try using XML-RPC instead.${NC}" >&2
    exit 1
fi

# Parse response - JSON2 returns array directly, not wrapped in "result"
# Expected format: [{"id": 1, "name": "...", "login": "..."}]
ODOO_UID=$(echo "$RESPONSE" | grep -o '"id":[[:space:]]*[0-9]*' | head -1 | grep -o '[0-9]*')
USERNAME=$(echo "$RESPONSE" | grep -o '"name":"[^"]*"' | head -1 | cut -d'"' -f4 || echo "unknown")
LOGIN=$(echo "$RESPONSE" | grep -o '"login":"[^"]*"' | head -1 | cut -d'"' -f4 || echo "unknown")

# Check if authentication was successful
if [[ -z "$ODOO_UID" ]]; then
    echo -e "${RED}Authentication failed: Could not retrieve user ID${NC}" >&2
    exit 1
fi

# Try to get Odoo version (may not work via JSON2, fallback to "unknown")
VERSION="19.0+"  # Default for JSON2

# Output results to stderr for user feedback
echo -e "${GREEN}✓ Authenticated successfully (JSON2 with Bearer token)${NC}" >&2
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
