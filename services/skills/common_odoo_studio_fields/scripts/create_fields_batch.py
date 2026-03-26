#!/usr/bin/env python3
"""
Create multiple fields in batch from a JSON file or inline JSON.

Usage:
    create_fields_batch.py <model> <fields_json_or_file> [--add-to-view <group_name>]

Examples:
    # From inline JSON
    create_fields_batch.py crm.lead '[{"name": "Field 1", "type": "char"}, {"name": "Field 2", "type": "boolean"}]'

    # From JSON file
    create_fields_batch.py crm.lead fields.json

    # With view positioning
    create_fields_batch.py crm.lead fields.json --add-to-view studio_group_abc_left

JSON format:
    [
        {"name": "Field Name", "type": "char", "options": {}},
        {"name": "Is Active", "type": "boolean"},
        {"name": "Category", "type": "selection", "options": {
            "selection_options": [{"value": "a", "name": "A"}]
        }}
    ]
"""

import sys
import os
import json
import argparse
from odoo_api import OdooAPI, OdooAPIError

# Import create_field function
from create_field import create_field, find_available_name, log_info, log_success, log_error

# Colors
RED = '\033[0;31m'
GREEN = '\033[0;32m'
YELLOW = '\033[1;33m'
NC = '\033[0m'


def load_fields_definition(fields_input: str) -> list:
    """Load fields definition from JSON string or file."""
    # Check if it's a file path
    if os.path.isfile(fields_input):
        with open(fields_input, 'r', encoding='utf-8') as f:
            data = json.load(f)
    else:
        # Try to parse as JSON
        data = json.loads(fields_input)

    # Handle both list and dict with "fields" key
    if isinstance(data, dict) and 'fields' in data:
        return data['fields']
    elif isinstance(data, list):
        return data
    else:
        raise ValueError("Invalid format: expected list of fields or dict with 'fields' key")


def add_field_to_view(api: OdooAPI, model: str, field_name: str, group_name: str):
    """Add a field to a view group."""
    from add_to_view import add_to_view
    try:
        add_to_view(model, 'form', field_name, 'inside', group_name, {})
    except Exception as e:
        log_error(f"Failed to add {field_name} to view: {e}")


def create_fields_batch(model: str, fields: list, add_to_view_group: str = None) -> dict:
    """
    Create multiple fields in batch.

    Returns dict with created fields and any errors.
    """
    api = OdooAPI()

    results = {
        'created': [],
        'errors': [],
        'total': len(fields)
    }

    log_info(f"Creating {len(fields)} fields on model '{model}'...")
    print()

    for i, field_def in enumerate(fields, 1):
        name = field_def.get('name')
        ftype = field_def.get('type', 'char')
        options = field_def.get('options', {})

        if not name:
            results['errors'].append({'index': i, 'error': 'Missing field name'})
            continue

        try:
            # Enable auto_suffix by default for batch
            if 'auto_suffix' not in options:
                options['auto_suffix'] = True

            field_id = create_field(model, name, ftype, options)

            # Get the actual field name (might have suffix)
            base_name = api.generate_studio_name(name)
            actual_name = find_available_name(api, model, base_name, False)
            # Since field was just created, we need to find the latest one
            field_info = api.call('ir.model.fields', 'read', [[field_id]], {'fields': ['name']})
            actual_name = field_info[0]['name'] if field_info else base_name

            results['created'].append({
                'name': name,
                'field_name': actual_name,
                'field_id': field_id,
                'type': ftype
            })

            # Add to view if specified
            if add_to_view_group:
                add_field_to_view(api, model, actual_name, add_to_view_group)

            print(f"  [{i}/{len(fields)}] {GREEN}✓{NC} {actual_name}")

        except Exception as e:
            results['errors'].append({
                'index': i,
                'name': name,
                'error': str(e)
            })
            print(f"  [{i}/{len(fields)}] {RED}✗{NC} {name}: {e}")

    print()
    log_success(f"Created {len(results['created'])}/{len(fields)} fields")
    if results['errors']:
        log_error(f"Failed: {len(results['errors'])} fields")

    return results


def main():
    parser = argparse.ArgumentParser(
        description='Create multiple fields in batch'
    )
    parser.add_argument('model', help='Model name (e.g., crm.lead)')
    parser.add_argument('fields', help='JSON array of fields or path to JSON file')
    parser.add_argument('--add-to-view', dest='view_group', help='Add fields to this Studio group')

    args = parser.parse_args()

    try:
        fields = load_fields_definition(args.fields)
        results = create_fields_batch(args.model, fields, args.view_group)
        print(json.dumps(results, indent=2))
        sys.exit(0 if not results['errors'] else 1)
    except json.JSONDecodeError as e:
        log_error(f"Invalid JSON: {e}")
        sys.exit(1)
    except FileNotFoundError as e:
        log_error(f"File not found: {e}")
        sys.exit(1)
    except OdooAPIError as e:
        log_error(str(e))
        sys.exit(1)
    except Exception as e:
        log_error(str(e))
        sys.exit(1)


if __name__ == '__main__':
    main()
