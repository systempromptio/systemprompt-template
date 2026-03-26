#!/bin/bash
# Wrapper for add_to_view.py
# Usage: add_to_view.sh <model> <view_type> <field_name> <position> <reference> [attrs_json]
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
exec python3 "${SCRIPT_DIR}/add_to_view.py" "$@"
