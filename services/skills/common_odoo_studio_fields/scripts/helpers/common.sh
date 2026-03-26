#!/bin/bash
# Common functions for odoo-studio-fields scripts
# This file should be sourced by other scripts

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Log functions (all to stderr to keep stdout clean for data)
log_error() { echo -e "${RED}Error: $1${NC}" >&2; }
log_success() { echo -e "${GREEN}$1${NC}" >&2; }
log_info() { echo -e "${YELLOW}$1${NC}" >&2; }
log_debug() { echo -e "${BLUE}$1${NC}" >&2; }

# Validate required environment variables
validate_env() {
    local missing=()

    [[ -z "${ODOO_URL:-}" ]] && missing+=("ODOO_URL")
    [[ -z "${ODOO_DB:-}" ]] && missing+=("ODOO_DB")

    # Check for authentication (either session or API key)
    if [[ -z "${ODOO_SESSION_ID:-}" ]] && [[ -z "${ODOO_KEY:-}" ]]; then
        missing+=("ODOO_SESSION_ID or ODOO_KEY")
    fi

    if [[ ${#missing[@]} -gt 0 ]]; then
        log_error "Missing required environment variables: ${missing[*]}"
        log_info "Run 'eval \$(odoo-pilot/scripts/auth.sh)' first to authenticate"
        return 1
    fi

    return 0
}

# Ensure we have a valid session for JSON-RPC
# If ODOO_SESSION_ID is not set but ODOO_KEY is available, authenticate first
ensure_session() {
    if [[ -n "${ODOO_SESSION_ID:-}" ]]; then
        return 0
    fi

    if [[ -z "${ODOO_KEY:-}" ]] || [[ -z "${ODOO_USER:-}" ]]; then
        log_error "No session and no credentials available"
        return 1
    fi

    # Use a temp file for cookies to avoid mixing with JSON response
    local cookie_file="/tmp/odoo_session_$$.txt"

    # Authenticate to get session
    curl -s -X POST "${ODOO_URL}/web/session/authenticate" \
        -H 'Content-Type: application/json' \
        -c "$cookie_file" \
        -o /dev/null \
        -d "{
            \"jsonrpc\": \"2.0\",
            \"method\": \"call\",
            \"params\": {
                \"db\": \"${ODOO_DB}\",
                \"login\": \"${ODOO_USER}\",
                \"password\": \"${ODOO_KEY}\"
            },
            \"id\": 1
        }" 2>/dev/null

    # Extract session_id from cookie file
    if [[ -f "$cookie_file" ]]; then
        ODOO_SESSION_ID=$(grep session_id "$cookie_file" 2>/dev/null | awk '{print $NF}')
        rm -f "$cookie_file"
    fi

    if [[ -z "${ODOO_SESSION_ID:-}" ]]; then
        log_error "Failed to obtain session"
        return 1
    fi

    export ODOO_SESSION_ID
    return 0
}

# Execute JSON-RPC call to Odoo
# Usage: odoo_call <model> <method> <args_json> [kwargs_json]
odoo_call() {
    local model="$1"
    local method="$2"
    local args="$3"
    local kwargs="${4:-{}}"

    local protocol="${ODOO_PROTOCOL:-jsonrpc}"
    local response

    if [[ "$protocol" == "json2" ]]; then
        # JSON2 protocol (Odoo >= 19.0)
        response=$(curl -s -X POST "${ODOO_URL}/json/2/${model}/${method}" \
            -H 'Content-Type: application/json' \
            -H "Authorization: bearer ${ODOO_KEY}" \
            -H "X-Odoo-Database: ${ODOO_DB}" \
            -d "${kwargs}")
    else
        # JSON-RPC protocol (Odoo < 19.0)
        # Ensure we have a valid session
        ensure_session || return 1

        local payload=$(cat <<EOF
{
    "jsonrpc": "2.0",
    "method": "call",
    "params": {
        "model": "${model}",
        "method": "${method}",
        "args": ${args},
        "kwargs": ${kwargs}
    },
    "id": $(date +%s%N)
}
EOF
)
        response=$(curl -s -X POST "${ODOO_URL}/web/dataset/call_kw" \
            -H 'Content-Type: application/json' \
            -H "Cookie: session_id=${ODOO_SESSION_ID}" \
            -d "${payload}")
    fi

    # Check for errors
    if echo "$response" | grep -q '"error"'; then
        local error_msg=$(echo "$response" | python3 -c "import sys,json; r=json.load(sys.stdin); print(r.get('error',{}).get('data',{}).get('message', r.get('error',{}).get('message','Unknown error')))" 2>/dev/null)
        log_error "$error_msg"
        return 1
    fi

    # Extract result based on protocol
    if [[ "$protocol" == "json2" ]]; then
        echo "$response"
    else
        echo "$response" | python3 -c "import sys,json; r=json.load(sys.stdin); print(json.dumps(r.get('result')))" 2>/dev/null
    fi
}

# Generate Studio-compatible field name
# Usage: generate_studio_name "Mi Campo Personalizado" -> "x_studio_mi_campo_personalizado"
# If name already starts with x_studio_, just clean it up
generate_studio_name() {
    local name="$1"
    # Convert to lowercase, replace spaces with underscores, remove special chars
    local clean_name=$(echo "$name" | tr '[:upper:]' '[:lower:]' | tr ' ' '_' | sed 's/[^a-z0-9_]//g')

    # Check if already has x_studio_ prefix
    if [[ "$clean_name" == x_studio_* ]]; then
        echo "$clean_name"
    else
        echo "x_studio_${clean_name}"
    fi
}

# Generate a short UUID for Studio XML IDs
generate_studio_uuid() {
    python3 -c "import uuid; print(str(uuid.uuid4())[:8])"
}

# Check if a field already exists in a model
# Usage: field_exists <model> <field_name>
field_exists() {
    local model="$1"
    local field_name="$2"

    local result=$(odoo_call "ir.model.fields" "search_count" "[[\"model\",\"=\",\"${model}\"],[\"name\",\"=\",\"${field_name}\"]]" "{}")

    [[ "$result" == "0" ]] && return 1 || return 0
}

# Check if a model exists
# Usage: model_exists <model>
model_exists() {
    local model="$1"

    local result=$(odoo_call "ir.model" "search_count" "[[\"model\",\"=\",\"${model}\"]]" "{}")

    [[ "$result" == "0" ]] && return 1 || return 0
}

# Get the main view of a model by type
# Usage: get_main_view <model> <view_type>
get_main_view() {
    local model="$1"
    local view_type="$2"

    local result=$(odoo_call "ir.ui.view" "search_read" \
        "[[\"model\",\"=\",\"${model}\"],[\"type\",\"=\",\"${view_type}\"],[\"mode\",\"=\",\"primary\"]]" \
        "{\"fields\":[\"id\",\"name\",\"arch_db\"],\"limit\":1}")

    echo "$result"
}

# Create a record in ir.model.data to track Studio customizations
# Usage: register_studio_data <model> <res_id> <name_suffix>
register_studio_data() {
    local model="$1"
    local res_id="$2"
    local name_suffix="$3"

    local uuid=$(generate_studio_uuid)
    local name="${name_suffix}_${uuid}"

    local values=$(cat <<EOF
{
    "name": "${name}",
    "module": "studio_customization",
    "model": "${model}",
    "res_id": ${res_id},
    "studio": true
}
EOF
)

    odoo_call "ir.model.data" "create" "[${values}]" "{}"
}

# Map user-friendly field type to Odoo ttype
map_field_type() {
    local type="$1"
    case "$type" in
        char|text|boolean|integer|float|date|datetime|binary|html|monetary)
            echo "$type"
            ;;
        string|varchar)
            echo "char"
            ;;
        number|int)
            echo "integer"
            ;;
        decimal|double)
            echo "float"
            ;;
        checkbox|bool)
            echo "boolean"
            ;;
        select|dropdown)
            echo "selection"
            ;;
        relation|link)
            echo "many2one"
            ;;
        *)
            echo "$type"
            ;;
    esac
}
