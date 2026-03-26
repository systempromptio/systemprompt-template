#!/usr/bin/env python3
"""
Create a field in Odoo following Studio conventions.

Usage:
    create_field.py <model> <field_name> <field_type> [options_json]

Examples:
    create_field.py res.partner "Secondary Phone" char '{}'
    create_field.py res.partner "Is VIP" boolean '{"required":true}'
    create_field.py sale.order "Priority" selection '{"selection_options":[{"value":"low","name":"Low"}]}'
"""

import sys
import json
import argparse
from odoo_api import OdooAPI, OdooAPIError

# Colors for output
RED = '\033[0;31m'
GREEN = '\033[0;32m'
YELLOW = '\033[1;33m'
NC = '\033[0m'


def log_error(msg):
    print(f"{RED}Error: {msg}{NC}", file=sys.stderr)


def log_success(msg):
    print(f"{GREEN}{msg}{NC}", file=sys.stderr)


def log_info(msg):
    print(f"{YELLOW}{msg}{NC}", file=sys.stderr)


def create_field(model: str, field_name: str, field_type: str, options: dict) -> int:
    """Create a field and return its ID."""
    api = OdooAPI()

    # Generate studio name
    studio_name = api.generate_studio_name(field_name)
    ttype = api.map_field_type(field_type)

    log_info(f"Creating field '{studio_name}' on model '{model}'...")

    # Validate model exists
    if not api.model_exists(model):
        raise OdooAPIError(f"Model '{model}' does not exist")

    # Check field doesn't already exist
    if api.field_exists(model, studio_name):
        raise OdooAPIError(f"Field '{studio_name}' already exists on model '{model}'")

    # Get model ID
    model_id = api.get_model_id(model)
    if not model_id:
        raise OdooAPIError(f"Could not find model ID for '{model}'")

    # Build field values
    field_values = {
        "name": studio_name,
        "field_description": field_name,
        "model_id": model_id,
        "model": model,
        "ttype": ttype,
        "state": "manual",
        "store": True,
        "readonly": options.get('readonly', False),
        "required": options.get('required', False),
        "copied": options.get('copied', True),
    }

    # Add help text
    if options.get('help'):
        field_values['help'] = options['help']

    # Handle many2one
    if ttype == 'many2one' and options.get('relation'):
        field_values['relation'] = options['relation']
        if options.get('domain'):
            field_values['domain'] = options['domain']

    # Handle selection
    if ttype == 'selection' and options.get('selection_options'):
        sel_list = [(opt['value'], opt['name']) for opt in options['selection_options']]
        field_values['selection'] = str(sel_list)

    # Create the field
    field_id = api.call('ir.model.fields', 'create', [field_values])

    if not field_id:
        raise OdooAPIError("Failed to create field")

    log_success(f"Created field {studio_name} (ID: {field_id})")

    # Create selection options in ir.model.fields.selection
    if ttype == 'selection' and options.get('selection_options'):
        log_info("Creating selection options...")
        for seq, opt in enumerate(options['selection_options']):
            api.call('ir.model.fields.selection', 'create', [{
                'field_id': field_id,
                'value': opt['value'],
                'name': opt['name'],
                'sequence': seq
            }])
        log_success(f"Created {len(options['selection_options'])} selection options")

    # Register in ir.model.data for Studio tracking
    log_info("Registering Studio metadata...")
    uuid_suffix = api.generate_studio_uuid()
    api.call('ir.model.data', 'create', [{
        'name': f"field_{studio_name}_{uuid_suffix}",
        'module': 'studio_customization',
        'model': 'ir.model.fields',
        'res_id': field_id,
        'studio': True
    }])

    log_success(f"Field '{studio_name}' created successfully on '{model}'")
    return field_id


def main():
    parser = argparse.ArgumentParser(
        description='Create a field in Odoo following Studio conventions'
    )
    parser.add_argument('model', help='Target model (e.g., res.partner)')
    parser.add_argument('field_name', help='Descriptive field name')
    parser.add_argument('field_type', help='Field type (char, boolean, selection, etc.)')
    parser.add_argument('options', nargs='?', default='{}', help='JSON options')

    args = parser.parse_args()

    try:
        options = json.loads(args.options)
        field_id = create_field(args.model, args.field_name, args.field_type, options)
        print(field_id)  # Output field ID to stdout
        sys.exit(0)
    except json.JSONDecodeError as e:
        log_error(f"Invalid JSON options: {e}")
        sys.exit(1)
    except OdooAPIError as e:
        log_error(str(e))
        sys.exit(1)
    except Exception as e:
        log_error(str(e))
        sys.exit(1)


if __name__ == '__main__':
    main()
