#!/bin/bash
# Wrapper for create_field.py
# Usage: create_field.sh <model> <field_name> <field_type> [options_json]
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
exec python3 "${SCRIPT_DIR}/create_field.py" "$@"
