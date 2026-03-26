#!/usr/bin/env python3
"""
Create a new page/tab in an Odoo form view following Studio conventions.

Usage:
    create_page.py <model> "<page_title>" [options_json]

Examples:
    create_page.py crm.lead "Mi Pestaña" '{}'
    create_page.py res.partner "Extra Info" '{"after_page": "internal_notes"}'
    create_page.py sale.order "Custom Data" '{"invisible": "state != 'sale'"}'
"""

import sys
import json
import argparse
from odoo_api import OdooAPI, OdooAPIError

# Colors for output
RED = '\033[0;31m'
GREEN = '\033[0;32m'
YELLOW = '\033[1;33m'
BLUE = '\033[0;34m'
NC = '\033[0m'


def log_error(msg):
    print(f"{RED}Error: {msg}{NC}", file=sys.stderr)


def log_success(msg):
    print(f"{GREEN}{msg}{NC}", file=sys.stderr)


def log_info(msg):
    print(f"{YELLOW}{msg}{NC}", file=sys.stderr)


def log_debug(msg):
    print(f"{BLUE}{msg}{NC}", file=sys.stderr)


def find_studio_view(api: OdooAPI, model: str, base_view_id: int) -> dict:
    """Find an existing Studio customization view."""
    views = api.call('ir.ui.view', 'search_read', [
        [['model', '=', model],
         ['type', '=', 'form'],
         ['inherit_id', '=', base_view_id],
         ['name', 'ilike', 'studio']]
    ], {'fields': ['id', 'name', 'arch_db'], 'order': 'id', 'limit': 1})
    return views[0] if views else None


def create_page(model: str, page_title: str, options: dict) -> dict:
    """
    Create a new page in a form view.

    Returns dict with page_name, group_name, left_group, right_group
    """
    api = OdooAPI()

    # Generate unique names
    uuid_suffix = api.generate_studio_uuid()
    page_name = f"studio_page_{uuid_suffix}"
    group_name = f"studio_group_{uuid_suffix}"
    left_group = f"{group_name}_left"
    right_group = f"{group_name}_right"

    log_info(f"Creating page '{page_title}' on model '{model}'...")

    # Validate model exists
    if not api.model_exists(model):
        raise OdooAPIError(f"Model '{model}' does not exist")

    # Get primary form view
    view = api.call('ir.ui.view', 'search_read', [
        [['model', '=', model], ['type', '=', 'form'], ['mode', '=', 'primary']]
    ], {'fields': ['id', 'name'], 'limit': 1})

    if not view:
        raise OdooAPIError(f"Could not find primary form view for model '{model}'")

    base_view_id = view[0]['id']
    base_view_name = view[0]['name']
    log_debug(f"Found base view: {base_view_name} (ID: {base_view_id})")

    # Build page XML
    invisible_attr = f' invisible="{options["invisible"]}"' if options.get('invisible') else ''

    page_xml = f'''<page string="{page_title}" name="{page_name}"{invisible_attr}>
      <group name="{group_name}">
        <group name="{left_group}"/>
        <group name="{right_group}"/>
      </group>
    </page>'''

    # Determine position xpath
    after_page = options.get('after_page')
    if after_page:
        xpath_expr = f"//page[@name='{after_page}']"
        position = "after"
    else:
        # Default: add at the end of notebook
        xpath_expr = "//notebook"
        position = "inside"

    xpath_element = f'<xpath expr="{xpath_expr}" position="{position}">{page_xml}</xpath>'
    log_debug(f"XPath: {xpath_element[:100]}...")

    # Check for existing Studio view
    studio_view = find_studio_view(api, model, base_view_id)

    if studio_view:
        # Append to existing view
        log_info(f"Found existing Studio view (ID: {studio_view['id']}), appending...")
        arch = studio_view['arch_db']
        new_arch = arch.replace('</data>', f'  {xpath_element}\n</data>')
        api.call('ir.ui.view', 'write', [[studio_view['id']], {'arch_db': new_arch}])
        view_id = studio_view['id']
    else:
        # Create new Studio view
        log_info("Creating new Studio customization view...")
        arch_xml = f"<data>\n  {xpath_element}\n</data>"

        view_values = {
            "name": f"Odoo Studio: {base_view_name} customization",
            "model": model,
            "type": "form",
            "mode": "extension",
            "inherit_id": base_view_id,
            "arch_db": arch_xml,
            "priority": 99
        }

        view_id = api.call('ir.ui.view', 'create', [view_values])

        # Register in ir.model.data
        api.call('ir.model.data', 'create', [{
            'name': f"view_{model.replace('.', '_')}_form_{uuid_suffix}",
            'module': 'studio_customization',
            'model': 'ir.ui.view',
            'res_id': view_id,
            'studio': True
        }])

    log_success(f"Page '{page_title}' created successfully!")
    log_success(f"  Page name: {page_name}")
    log_success(f"  Left group: {left_group}")
    log_success(f"  Right group: {right_group}")

    return {
        'view_id': view_id,
        'page_name': page_name,
        'group_name': group_name,
        'left_group': left_group,
        'right_group': right_group
    }


def main():
    parser = argparse.ArgumentParser(
        description='Create a new page/tab in an Odoo form view'
    )
    parser.add_argument('model', help='Model name (e.g., crm.lead)')
    parser.add_argument('page_title', help='Page title to display')
    parser.add_argument('options', nargs='?', default='{}', help='JSON options')

    args = parser.parse_args()

    try:
        options = json.loads(args.options)
        result = create_page(args.model, args.page_title, options)
        # Output JSON result to stdout
        print(json.dumps(result))
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
