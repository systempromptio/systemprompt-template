#!/bin/bash
# Add selection options to an existing selection field
# Usage: create_selection.sh <field_id> <options_json>
#
# Examples:
#   ./create_selection.sh 12345 '[{"value":"nuevo","name":"Nuevo"},{"value":"activo","name":"Activo"}]'
#
# This script is typically called automatically by create_field.sh for selection fields,
# but can be used standalone to add options to existing selection fields.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/helpers/common.sh"

# Validate arguments
if [[ $# -lt 2 ]]; then
    log_error "Missing required arguments"
    echo "Usage: $0 <field_id> <options_json>" >&2
    echo "" >&2
    echo "Arguments:" >&2
    echo "  field_id     - ID of the selection field in ir.model.fields" >&2
    echo "  options_json - Array of options: [{\"value\":\"key\",\"name\":\"Label\"},...]" >&2
    exit 1
fi

FIELD_ID="$1"
OPTIONS="$2"

# Validate environment
validate_env || exit 1

# Validate field exists and is selection type
FIELD_INFO=$(odoo_call "ir.model.fields" "search_read" \
    "[[\"id\",\"=\",${FIELD_ID}]]" \
    "{\"fields\":[\"name\",\"ttype\",\"model\"]}")

FIELD_TYPE=$(echo "$FIELD_INFO" | python3 -c "import sys,json; r=json.load(sys.stdin); print(r[0]['ttype'] if r else '')" 2>/dev/null)
FIELD_NAME=$(echo "$FIELD_INFO" | python3 -c "import sys,json; r=json.load(sys.stdin); print(r[0]['name'] if r else '')" 2>/dev/null)

if [[ -z "$FIELD_TYPE" ]]; then
    log_error "Field with ID ${FIELD_ID} not found"
    exit 1
fi

if [[ "$FIELD_TYPE" != "selection" ]]; then
    log_error "Field '${FIELD_NAME}' is not a selection field (type: ${FIELD_TYPE})"
    exit 1
fi

log_info "Adding selection options to field '${FIELD_NAME}'..."

# Get current max sequence
CURRENT_MAX=$(odoo_call "ir.model.fields.selection" "search_read" \
    "[[\"field_id\",\"=\",${FIELD_ID}]]" \
    "{\"fields\":[\"sequence\"],\"order\":\"sequence desc\",\"limit\":1}")

SEQUENCE=$(echo "$CURRENT_MAX" | python3 -c "import sys,json; r=json.load(sys.stdin); print(r[0]['sequence']+1 if r else 0)" 2>/dev/null || echo "0")

# Parse and create options
COUNT=0
for option in $(echo "$OPTIONS" | python3 -c "import sys,json; opts=json.load(sys.stdin); [print(json.dumps(o)) for o in opts]" 2>/dev/null); do
    VALUE=$(echo "$option" | python3 -c "import sys,json; print(json.load(sys.stdin)['value'])")
    NAME=$(echo "$option" | python3 -c "import sys,json; print(json.load(sys.stdin)['name'])")

    # Check if option already exists
    EXISTS=$(odoo_call "ir.model.fields.selection" "search_count" \
        "[[\"field_id\",\"=\",${FIELD_ID}],[\"value\",\"=\",\"${VALUE}\"]]" "{}")

    if [[ "$EXISTS" != "0" ]]; then
        log_info "Option '${VALUE}' already exists, skipping..."
        continue
    fi

    SEL_VALUES=$(cat <<EOF
{
    "field_id": ${FIELD_ID},
    "value": "${VALUE}",
    "name": "${NAME}",
    "sequence": ${SEQUENCE}
}
EOF
)

    OPTION_ID=$(odoo_call "ir.model.fields.selection" "create" "[${SEL_VALUES}]" "{}")

    if [[ -n "$OPTION_ID" ]] && [[ "$OPTION_ID" != "null" ]]; then
        log_debug "Created option: ${VALUE} -> ${NAME} (seq: ${SEQUENCE})"
        COUNT=$((COUNT + 1))
        SEQUENCE=$((SEQUENCE + 1))
    fi
done

# Update the selection field's selection attribute
log_info "Updating field selection attribute..."

ALL_OPTIONS=$(odoo_call "ir.model.fields.selection" "search_read" \
    "[[\"field_id\",\"=\",${FIELD_ID}]]" \
    "{\"fields\":[\"value\",\"name\"],\"order\":\"sequence\"}")

SELECTION_STR=$(echo "$ALL_OPTIONS" | python3 -c "
import sys, json
opts = json.load(sys.stdin)
sel_list = [(o['value'], o['name']) for o in opts]
print(str(sel_list))
" 2>/dev/null)

odoo_call "ir.model.fields" "write" "[[${FIELD_ID}],{\"selection\":\"${SELECTION_STR}\"}]" "{}" > /dev/null

log_success "Added ${COUNT} selection options to '${FIELD_NAME}'"

echo "$COUNT"
