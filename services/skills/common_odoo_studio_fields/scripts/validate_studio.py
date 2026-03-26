#!/usr/bin/env python3
"""
Validate that fields and views follow Odoo Studio conventions.

Usage:
    validate_studio.py <model> [--fix]

Examples:
    validate_studio.py crm.lead
    validate_studio.py res.partner --fix
"""

import sys
import json
import argparse
import re
from odoo_api import OdooAPI, OdooAPIError

# Colors
RED = '\033[0;31m'
GREEN = '\033[0;32m'
YELLOW = '\033[1;33m'
BLUE = '\033[0;34m'
NC = '\033[0m'


def log_error(msg):
    print(f"{RED}✗ {msg}{NC}")


def log_success(msg):
    print(f"{GREEN}✓ {msg}{NC}")


def log_warning(msg):
    print(f"{YELLOW}⚠ {msg}{NC}")


def log_info(msg):
    print(f"{BLUE}ℹ {msg}{NC}")


class StudioValidator:
    """Validates Studio conventions."""

    def __init__(self, api: OdooAPI):
        self.api = api
        self.issues = []
        self.warnings = []

    def validate_field(self, field: dict) -> bool:
        """Validate a single field."""
        name = field['name']
        model = field['model']
        valid = True

        # Check prefix
        if name.startswith('x_') and not name.startswith('x_studio_'):
            self.warnings.append({
                'type': 'naming',
                'field': name,
                'model': model,
                'message': f"Custom field '{name}' doesn't follow x_studio_ convention"
            })

        # Check state for x_studio_ fields
        if name.startswith('x_studio_'):
            if field.get('state') != 'manual':
                self.issues.append({
                    'type': 'state',
                    'field': name,
                    'model': model,
                    'message': f"Studio field '{name}' has state='{field.get('state')}' instead of 'manual'"
                })
                valid = False

            # Check if registered in ir.model.data
            data = self.api.call('ir.model.data', 'search_count', [
                [['model', '=', 'ir.model.fields'],
                 ['res_id', '=', field['id']],
                 ['module', '=', 'studio_customization']]
            ])
            if data == 0:
                self.warnings.append({
                    'type': 'metadata',
                    'field': name,
                    'model': model,
                    'message': f"Studio field '{name}' not registered in ir.model.data"
                })

        return valid

    def validate_view(self, view: dict) -> bool:
        """Validate a Studio view."""
        valid = True
        name = view['name']

        # Check naming convention
        if 'studio' in name.lower():
            if view.get('mode') != 'extension':
                self.issues.append({
                    'type': 'view_mode',
                    'view': name,
                    'view_id': view['id'],
                    'message': f"Studio view '{name}' has mode='{view.get('mode')}' instead of 'extension'"
                })
                valid = False

            # Check arch structure
            arch = view.get('arch_db', '')
            if not arch.startswith('<data>'):
                self.warnings.append({
                    'type': 'view_arch',
                    'view': name,
                    'view_id': view['id'],
                    'message': f"Studio view '{name}' arch doesn't start with <data>"
                })

            # Check for proper xpath usage
            if '<xpath' not in arch and '<field' in arch:
                self.warnings.append({
                    'type': 'view_xpath',
                    'view': name,
                    'view_id': view['id'],
                    'message': f"Studio view '{name}' contains fields without xpath wrapper"
                })

        return valid

    def validate_model(self, model: str) -> dict:
        """Validate all Studio elements for a model."""
        print(f"\n{'='*60}")
        print(f"Validating Studio conventions for: {model}")
        print(f"{'='*60}\n")

        # Get all custom fields
        fields = self.api.call('ir.model.fields', 'search_read', [
            [['model', '=', model], ['name', 'like', 'x_%']]
        ], {'fields': ['id', 'name', 'model', 'state', 'ttype', 'field_description']})

        print(f"Found {len(fields)} custom fields (x_*)")

        studio_fields = [f for f in fields if f['name'].startswith('x_studio_')]
        other_custom = [f for f in fields if f['name'].startswith('x_') and not f['name'].startswith('x_studio_')]

        print(f"  - x_studio_* fields: {len(studio_fields)}")
        print(f"  - Other x_* fields: {len(other_custom)}")
        print()

        # Validate fields
        print("Validating fields...")
        for field in fields:
            self.validate_field(field)
            if field['name'].startswith('x_studio_'):
                log_success(f"{field['name']} ({field['ttype']})")
            elif field['name'].startswith('x_'):
                log_warning(f"{field['name']} ({field['ttype']}) - not using x_studio_ prefix")

        # Get Studio views
        print(f"\nValidating views...")
        views = self.api.call('ir.ui.view', 'search_read', [
            [['model', '=', model], ['name', 'ilike', 'studio']]
        ], {'fields': ['id', 'name', 'type', 'mode', 'inherit_id', 'arch_db', 'priority']})

        print(f"Found {len(views)} Studio views")

        for view in views:
            self.validate_view(view)
            inherit = view['inherit_id'][1] if view['inherit_id'] else 'N/A'
            if view.get('mode') == 'extension' and view.get('priority') == 99:
                log_success(f"{view['name']} ({view['type']}) - inherits: {inherit}")
            else:
                log_warning(f"{view['name']} ({view['type']}) - mode={view.get('mode')}, priority={view.get('priority')}")

        # Summary
        print(f"\n{'='*60}")
        print("SUMMARY")
        print(f"{'='*60}")

        print(f"\nTotal custom fields: {len(fields)}")
        print(f"Studio views: {len(views)}")
        print(f"Issues found: {len(self.issues)}")
        print(f"Warnings: {len(self.warnings)}")

        if self.issues:
            print(f"\n{RED}Issues:{NC}")
            for issue in self.issues:
                print(f"  - {issue['message']}")

        if self.warnings:
            print(f"\n{YELLOW}Warnings:{NC}")
            for warning in self.warnings:
                print(f"  - {warning['message']}")

        return {
            'model': model,
            'fields_count': len(fields),
            'studio_fields': len(studio_fields),
            'views_count': len(views),
            'issues': self.issues,
            'warnings': self.warnings,
            'valid': len(self.issues) == 0
        }


def main():
    parser = argparse.ArgumentParser(
        description='Validate Studio conventions for a model'
    )
    parser.add_argument('model', help='Model name (e.g., crm.lead)')
    parser.add_argument('--json', action='store_true', help='Output as JSON')

    args = parser.parse_args()

    try:
        api = OdooAPI()
        validator = StudioValidator(api)
        result = validator.validate_model(args.model)

        if args.json:
            print(json.dumps(result, indent=2))

        sys.exit(0 if result['valid'] else 1)

    except OdooAPIError as e:
        log_error(str(e))
        sys.exit(1)
    except Exception as e:
        log_error(str(e))
        sys.exit(1)


if __name__ == '__main__':
    main()
