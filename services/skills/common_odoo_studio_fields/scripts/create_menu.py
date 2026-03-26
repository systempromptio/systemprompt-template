#!/usr/bin/env python3
"""
Create menus and actions in Odoo following Studio conventions.

Creates:
- ir.actions.act_window (action to open model)
- ir.ui.menu (menu entry)
- Registers both in ir.model.data with studio=True

Usage:
    create_menu.py <model> <menu_name> [options_json]

Examples:
    # Create menu for existing model
    create_menu.py x_custom_project "Custom Projects" '{}'

    # Create menu under a parent menu
    create_menu.py x_vehicle "Vehicles" '{"parent_menu": "contacts.menu_contacts"}'

    # Create menu with custom icon and sequence
    create_menu.py crm.lead "My Leads" '{"parent_id": 123, "sequence": 10}'
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
    print(f"{GREEN}✓ {msg}{NC}", file=sys.stderr)


def log_info(msg):
    print(f"{YELLOW}{msg}{NC}", file=sys.stderr)


def log_debug(msg):
    print(f"{BLUE}{msg}{NC}", file=sys.stderr)


def find_parent_menu(api: OdooAPI, reference: str) -> int:
    """
    Find parent menu by reference (XML ID or name).

    Args:
        api: OdooAPI instance
        reference: Can be:
            - XML ID: "contacts.menu_contacts"
            - Menu name: "Contacts"
            - Menu ID: 123

    Returns:
        Menu ID
    """
    # If it's already an integer
    if isinstance(reference, int):
        return reference

    # Try as XML ID first
    if '.' in reference:
        module, name = reference.split('.', 1)
        result = api.call('ir.model.data', 'search_read', [
            [['module', '=', module], ['name', '=', name], ['model', '=', 'ir.ui.menu']]
        ], {'fields': ['res_id'], 'limit': 1})
        if result:
            return result[0]['res_id']

    # Try as menu name
    result = api.call('ir.ui.menu', 'search', [
        [['name', 'ilike', reference]]
    ], {'limit': 1})

    if result:
        return result[0]

    raise OdooAPIError(f"Could not find parent menu: {reference}")


def create_action(api: OdooAPI, model: str, name: str, options: dict) -> int:
    """
    Create an ir.actions.act_window for the model.

    Returns:
        Action ID
    """
    action_values = {
        'name': name,
        'res_model': model,
        'type': 'ir.actions.act_window',
        'view_mode': options.get('view_mode', 'tree,form'),
        'target': options.get('target', 'current'),
    }

    # Optional: domain filter
    if options.get('domain'):
        action_values['domain'] = options['domain']

    # Optional: context
    if options.get('context'):
        action_values['context'] = options['context']

    # Optional: specific views
    if options.get('view_id'):
        action_values['view_id'] = options['view_id']

    log_debug(f"Creating action: {action_values}")

    action_id = api.call('ir.actions.act_window', 'create', [action_values])

    if not action_id:
        raise OdooAPIError("Failed to create action")

    # Register in ir.model.data
    studio_uuid = api.generate_studio_uuid()
    model_clean = model.replace('.', '_')
    api.call('ir.model.data', 'create', [{
        'name': f'action_{model_clean}_{studio_uuid}',
        'module': 'studio_customization',
        'model': 'ir.actions.act_window',
        'res_id': action_id,
        'studio': True
    }])

    return action_id


def create_menu(model: str, menu_name: str, options: dict) -> dict:
    """
    Create a menu entry with associated action.

    Args:
        model: Model to open (e.g., "crm.lead", "x_custom_project")
        menu_name: Display name for the menu
        options: Dictionary with options:
            - parent_menu: Parent menu reference (XML ID, name, or ID)
            - parent_id: Direct parent menu ID
            - sequence: Menu order (default: 10)
            - groups: List of group XML IDs for access control
            - view_mode: View modes (default: "tree,form")
            - domain: Domain filter for records
            - create_action: Whether to create action (default: True)
            - action_id: Use existing action ID instead of creating

    Returns:
        Dictionary with menu_id, action_id, and details
    """
    api = OdooAPI()

    log_info(f"Creating menu '{menu_name}' for model '{model}'...")

    # Verify model exists
    if not api.model_exists(model):
        raise OdooAPIError(f"Model '{model}' does not exist")

    # Resolve parent menu
    parent_id = None
    if options.get('parent_id'):
        parent_id = options['parent_id']
    elif options.get('parent_menu'):
        parent_id = find_parent_menu(api, options['parent_menu'])
        log_debug(f"Found parent menu ID: {parent_id}")

    # Create or use existing action
    action_id = None
    if options.get('action_id'):
        action_id = options['action_id']
        log_debug(f"Using existing action ID: {action_id}")
    elif options.get('create_action', True):
        log_info("Creating action...")
        action_id = create_action(api, model, menu_name, options)
        log_success(f"Action created (ID: {action_id})")

    # Create menu
    menu_values = {
        'name': menu_name,
        'sequence': options.get('sequence', 10),
    }

    if parent_id:
        menu_values['parent_id'] = parent_id

    if action_id:
        menu_values['action'] = f'ir.actions.act_window,{action_id}'

    # Handle groups
    if options.get('groups'):
        group_ids = []
        for group_ref in options['groups']:
            if '.' in group_ref:
                module, name = group_ref.split('.', 1)
                result = api.call('ir.model.data', 'search_read', [
                    [['module', '=', module], ['name', '=', name], ['model', '=', 'res.groups']]
                ], {'fields': ['res_id'], 'limit': 1})
                if result:
                    group_ids.append(result[0]['res_id'])
        if group_ids:
            menu_values['groups_id'] = [(6, 0, group_ids)]

    log_debug(f"Creating menu: {menu_values}")

    menu_id = api.call('ir.ui.menu', 'create', [menu_values])

    if not menu_id:
        raise OdooAPIError("Failed to create menu")

    log_success(f"Menu created (ID: {menu_id})")

    # Register in ir.model.data
    studio_uuid = api.generate_studio_uuid()
    model_clean = model.replace('.', '_')
    api.call('ir.model.data', 'create', [{
        'name': f'menu_{model_clean}_{studio_uuid}',
        'module': 'studio_customization',
        'model': 'ir.ui.menu',
        'res_id': menu_id,
        'studio': True
    }])
    log_success("Registered in ir.model.data")

    return {
        'menu_id': menu_id,
        'action_id': action_id,
        'menu_name': menu_name,
        'model': model,
        'parent_id': parent_id
    }


def list_menus(api: OdooAPI, search_term: str = None) -> list:
    """
    List available menus (useful for finding parent menus).

    Args:
        api: OdooAPI instance
        search_term: Optional search term

    Returns:
        List of menus with id, name, parent
    """
    domain = []
    if search_term:
        domain = [['name', 'ilike', search_term]]

    menus = api.call('ir.ui.menu', 'search_read', [domain], {
        'fields': ['id', 'name', 'parent_id', 'sequence'],
        'order': 'parent_id, sequence',
        'limit': 50
    })

    return menus


def main():
    parser = argparse.ArgumentParser(
        description='Create Odoo menus following Studio conventions'
    )

    subparsers = parser.add_subparsers(dest='command', help='Commands')

    # create command
    create_parser = subparsers.add_parser('create', help='Create a new menu')
    create_parser.add_argument('model', help='Model to open (e.g., crm.lead)')
    create_parser.add_argument('menu_name', help='Display name for the menu')
    create_parser.add_argument('options', nargs='?', default='{}', help='JSON options')
    create_parser.add_argument('--parent', '-p', help='Parent menu (XML ID or name)')
    create_parser.add_argument('--sequence', '-s', type=int, help='Menu sequence/order')
    create_parser.add_argument('--no-action', action='store_true', help='Do not create action')

    # list command
    list_parser = subparsers.add_parser('list', help='List menus')
    list_parser.add_argument('search', nargs='?', help='Search term')

    # Default behavior: create
    parser.add_argument('model', nargs='?', help='Model to open')
    parser.add_argument('menu_name', nargs='?', help='Menu name')
    parser.add_argument('options', nargs='?', default='{}', help='JSON options')

    args = parser.parse_args()

    try:
        api = OdooAPI()

        # Handle subcommands
        if args.command == 'list':
            menus = list_menus(api, args.search)
            for menu in menus:
                parent = menu['parent_id'][1] if menu['parent_id'] else 'ROOT'
                print(f"{menu['id']:5d} | {menu['name']:40s} | Parent: {parent}")
            sys.exit(0)

        elif args.command == 'create':
            options = json.loads(args.options)
            if args.parent:
                options['parent_menu'] = args.parent
            if args.sequence:
                options['sequence'] = args.sequence
            if args.no_action:
                options['create_action'] = False

            result = create_menu(args.model, args.menu_name, options)
            print(json.dumps(result))
            sys.exit(0)

        # Default behavior (no subcommand)
        elif args.model and args.menu_name:
            options = json.loads(args.options)
            result = create_menu(args.model, args.menu_name, options)
            print(json.dumps(result))
            sys.exit(0)

        else:
            parser.print_help()
            sys.exit(1)

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
