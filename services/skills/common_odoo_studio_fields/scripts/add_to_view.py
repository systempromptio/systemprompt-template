#!/usr/bin/env python3
"""
Add a field to an Odoo view using Studio-style inheritance.

Follows Studio convention:
- Uses absolute xpaths (like //form[1]/sheet[1]/group[1]/group[2]/field[@name='phone'])
- Reuses existing Studio customization view if available
- Positions relative to the combined view (not base view)

Usage:
    add_to_view.py <model> <view_type> <field_name> <position> <reference> [attrs_json]

Examples:
    add_to_view.py res.partner form x_studio_phone2 after phone '{}'
    add_to_view.py res.partner form x_studio_is_vip before email '{"invisible":"not is_company"}'
"""

import sys
import json
import argparse
from xml.etree import ElementTree as ET
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


def log_warning(msg):
    print(f"{YELLOW}Warning: {msg}{NC}", file=sys.stderr)


def get_element_xpath(root: ET.Element, target: ET.Element, prefer_named: bool = True) -> str:
    """
    Build an xpath to an element, similar to how Studio does it.

    Studio prefers named elements (e.g., //div[@name='vat_vies_container'])
    over positional xpaths (e.g., //form[1]/sheet[1]/group[1]/div[1]).

    Args:
        root: The root element of the view
        target: The target element to build xpath for
        prefer_named: If True, use named element xpath when available

    Returns xpath like:
        - //div[@name='vat_vies_container'] (if element has name attribute)
        - //form[1]/sheet[1]/group[1]/group[2]/field[@name='phone'] (positional)
    """
    # Build parent map
    parent_map = {child: parent for parent in root.iter() for child in parent}

    # Strategy 1: Check if target or a close ancestor has a 'name' attribute (preferred by Studio)
    if prefer_named:
        # Check target itself
        if target.tag != 'field' and target.get('name'):
            return f"//{target.tag}[@name='{target.get('name')}']"

        # Check parent (for fields inside named divs)
        parent = parent_map.get(target)
        if parent is not None and parent.tag == 'div' and parent.get('name'):
            return f"//div[@name='{parent.get('name')}']"

        # Check grandparent
        if parent is not None:
            grandparent = parent_map.get(parent)
            if grandparent is not None and grandparent.tag == 'div' and grandparent.get('name'):
                return f"//div[@name='{grandparent.get('name')}']"

    # Strategy 2: Build positional xpath
    path_parts = []
    current = target

    while current is not None:
        parent = parent_map.get(current)

        if current.tag == 'field':
            # For fields, use @name attribute
            name = current.get('name', '')
            path_parts.insert(0, f"field[@name='{name}']")
        elif current.tag in ('page', 'notebook', 'group', 'div', 'sheet', 'form', 'tree'):
            # For structural elements, count position among siblings of same type
            if parent is not None:
                siblings = [c for c in parent if c.tag == current.tag]
                if len(siblings) > 1:
                    idx = siblings.index(current) + 1
                    path_parts.insert(0, f"{current.tag}[{idx}]")
                else:
                    path_parts.insert(0, f"{current.tag}[1]")
            else:
                path_parts.insert(0, f"{current.tag}[1]")
        else:
            # For other elements, just use tag with position
            if parent is not None:
                siblings = [c for c in parent if c.tag == current.tag]
                if len(siblings) > 1:
                    idx = siblings.index(current) + 1
                    path_parts.insert(0, f"{current.tag}[{idx}]")
                else:
                    path_parts.insert(0, current.tag)
            else:
                path_parts.insert(0, current.tag)

        current = parent

    return '//' + '/'.join(path_parts)


def find_reference_element(root: ET.Element, reference: str, view_type: str):
    """
    Find the reference element in the view.
    Returns (element, parent) tuple.
    For form views, finds the field inside a group (main form area).
    """
    # Build parent map
    parent_map = {child: parent for parent in root.iter() for child in parent}

    if reference in ('sheet', 'form', 'tree'):
        elem = root.find(f".//{reference}")
        if elem is not None:
            return elem, parent_map.get(elem)
        return None, None

    # For fields, find all occurrences
    fields = root.findall(f".//field[@name='{reference}']")

    if not fields:
        return None, None

    if view_type == 'form':
        # For form views, prefer fields that are inside a group (main form area)
        # and NOT inside notebook/page (those are usually detail tabs)
        for field in fields:
            parent = parent_map.get(field)
            # Check ancestors to find if it's in a standard group
            ancestors = []
            current = parent
            while current is not None:
                ancestors.append(current.tag)
                current = parent_map.get(current)

            # Prefer fields in group but not in notebook
            if 'group' in ancestors and 'notebook' not in ancestors[:3]:
                return field, parent

        # If no ideal match, return first field in a group
        for field in fields:
            parent = parent_map.get(field)
            if parent is not None and parent.tag == 'group':
                return field, parent

    # Default: return first match
    return fields[0], parent_map.get(fields[0])


def find_position_element(root: ET.Element, reference: str, position: str, view_type: str):
    """
    Find the element to position relative to.
    For 'after' position, we might need to find the parent div/wrapper.

    Studio logic:
    1. If the field is inside a named div (div[@name='...']), use that div
    2. If the field is inside any div wrapper, use that div
    3. Otherwise, use the field itself

    Returns the xpath expression to use.
    """
    parent_map = {child: parent for parent in root.iter() for child in parent}

    ref_elem, ref_parent = find_reference_element(root, reference, view_type)

    if ref_elem is None:
        return None

    # Strategy 1: Check if field is inside a named container
    # This is what Studio prefers (e.g., //div[@name='vat_vies_container'])
    current = ref_elem
    while current is not None:
        parent = parent_map.get(current)
        if parent is not None:
            # Check if parent is a named div
            if parent.tag == 'div' and parent.get('name'):
                log_debug(f"Found named container: div[@name='{parent.get('name')}']")
                return f"//div[@name='{parent.get('name')}']"
            # Stop at group level - don't go higher
            if parent.tag == 'group':
                break
        current = parent

    # Strategy 2: Check if the field is wrapped in a div (common in Odoo 17+)
    if position in ('after', 'before') and ref_parent is not None:
        # The field is directly inside a div
        if ref_parent.tag == 'div':
            return get_element_xpath(root, ref_parent)

        # Check if there's a div that contains just this field (wrapper pattern)
        siblings = list(ref_parent)
        try:
            field_idx = siblings.index(ref_elem)
            for sib in siblings:
                if sib.tag == 'div':
                    # Check if this div contains our reference field
                    for child in sib.iter():
                        if child.tag == 'field' and child.get('name') == reference:
                            return get_element_xpath(root, sib)
        except ValueError:
            pass

    # Strategy 3: Use the field itself with positional xpath
    return get_element_xpath(root, ref_elem)


def build_field_xml(field_name: str, attrs: dict) -> str:
    """Build the XML for the field element (simple version like Studio)."""
    # Studio uses minimal attributes
    field_parts = [f'<field name="{field_name}"']

    for attr in ['invisible', 'readonly', 'required', 'widget', 'placeholder', 'string']:
        if attrs.get(attr):
            field_parts.append(f' {attr}="{attrs[attr]}"')

    if attrs.get('options'):
        opts = attrs['options'].replace('"', '&quot;')
        field_parts.append(f' options="{opts}"')

    field_parts.append('/>')
    return ''.join(field_parts)


def find_studio_view(api: OdooAPI, model: str, view_type: str, base_view_id: int) -> dict:
    """
    Find an existing Studio customization view for the given base view.
    Returns the view dict if found, None otherwise.
    """
    views = api.call('ir.ui.view', 'search_read', [
        [['model', '=', model],
         ['type', '=', view_type],
         ['inherit_id', '=', base_view_id],
         ['name', 'ilike', 'studio']]
    ], {'fields': ['id', 'name', 'arch_db'], 'order': 'id', 'limit': 1})

    return views[0] if views else None


def add_field_to_existing_xpath(arch_db: str, xpath_expr: str, position: str,
                                 field_xml: str) -> str:
    """
    Try to find an existing xpath with the same expression and position,
    and add the field inside it instead of creating a new xpath.

    This follows Studio's behavior of grouping fields in the same xpath
    when they share the same reference point.

    Returns the modified arch, or None if no matching xpath was found.
    """
    import re

    # Escape special regex characters in xpath_expr
    escaped_expr = re.escape(xpath_expr)

    # Pattern to find xpath with same expr and position
    # Matches: <xpath expr="..." position="after">...</xpath>
    pattern = rf'(<xpath\s+expr="{escaped_expr}"\s+position="{position}"[^>]*>)(.*?)(</xpath>)'

    match = re.search(pattern, arch_db, re.DOTALL)

    if match:
        # Found matching xpath - add field inside it
        opening_tag = match.group(1)
        content = match.group(2)
        closing_tag = match.group(3)

        # Add the new field after existing content (preserve indentation)
        # Detect indentation from existing content
        lines = content.split('\n')
        indent = '    '  # default
        for line in lines:
            if '<field' in line:
                indent = line[:len(line) - len(line.lstrip())]
                break

        new_content = content.rstrip() + f'\n{indent}{field_xml}\n  '
        new_xpath = f'{opening_tag}{new_content}{closing_tag}'

        return arch_db[:match.start()] + new_xpath + arch_db[match.end():]

    return None


def add_xpath_to_existing_view(api: OdooAPI, view_id: int, arch_db: str,
                               xpath_expr: str, position: str, field_xml: str) -> bool:
    """
    Add a field to an existing Studio view's arch.

    First tries to add to an existing xpath with the same expression/position.
    If no matching xpath exists, creates a new one.

    Returns True if added to existing xpath, False if new xpath created.
    """
    # Try to add to existing xpath first
    modified_arch = add_field_to_existing_xpath(arch_db, xpath_expr, position, field_xml)

    if modified_arch:
        # Successfully added to existing xpath
        api.call('ir.ui.view', 'write', [[view_id], {'arch_db': modified_arch}])
        return True

    # No matching xpath found - create new one
    xpath_element = f'<xpath expr="{xpath_expr}" position="{position}">\n    {field_xml}\n  </xpath>'

    if '</data>' in arch_db:
        new_arch = arch_db.replace('</data>', f'  {xpath_element}\n</data>')
    else:
        # Wrap existing content
        new_arch = f'<data>\n  {arch_db}\n  {xpath_element}\n</data>'

    api.call('ir.ui.view', 'write', [[view_id], {'arch_db': new_arch}])
    return False


def add_to_view(model: str, view_type: str, field_name: str,
                position: str, reference: str, attrs: dict) -> int:
    """Add a field to a view and return the view ID used."""
    api = OdooAPI()

    log_info(f"Adding field '{field_name}' to {model} {view_type} view...")

    # Validate inputs
    if view_type not in ('form', 'tree', 'kanban', 'search'):
        raise OdooAPIError(f"Invalid view type '{view_type}'. Must be: form, tree, kanban, search")

    if position not in ('after', 'before', 'inside'):
        raise OdooAPIError(f"Invalid position '{position}'. Must be: after, before, inside")

    # Find the primary view
    view = api.get_primary_view(model, view_type)
    if not view:
        raise OdooAPIError(f"Could not find primary {view_type} view for model '{model}'")

    base_view_id = view['id']
    base_view_name = view['name']
    log_debug(f"Found base view: {base_view_name} (ID: {base_view_id})")

    # Get the COMBINED view (this is key - Studio works on the combined view)
    log_debug("Getting combined view architecture...")
    result = api.call(model, 'get_view', [], {'view_id': base_view_id, 'view_type': view_type})

    if not result or 'arch' not in result:
        raise OdooAPIError("Could not get combined view architecture")

    combined_arch = result['arch']

    # Parse the combined view
    try:
        root = ET.fromstring(combined_arch)
    except ET.ParseError as e:
        raise OdooAPIError(f"Could not parse view XML: {e}")

    # Find the xpath expression for the reference element
    xpath_expr = find_position_element(root, reference, position, view_type)

    if not xpath_expr:
        raise OdooAPIError(f"Could not find reference element '{reference}' in the view")

    log_debug(f"Using xpath: {xpath_expr}")

    # Build the field XML
    field_xml = build_field_xml(field_name, attrs)
    log_debug(f"Field XML: {field_xml}")

    # Check for existing Studio customization view
    studio_view = find_studio_view(api, model, view_type, base_view_id)

    if studio_view:
        # Try to add to existing Studio view
        log_info(f"Found existing Studio view (ID: {studio_view['id']})")
        added_to_existing = add_xpath_to_existing_view(
            api, studio_view['id'], studio_view['arch_db'],
            xpath_expr, position, field_xml
        )
        view_id = studio_view['id']
        if added_to_existing:
            log_success(f"Added field to existing xpath in Studio view (ID: {view_id})")
        else:
            log_success(f"Created new xpath in Studio view (ID: {view_id})")
    else:
        # Create new Studio customization view
        log_info("No existing Studio view found, creating new one...")

        xpath_element = f'<xpath expr="{xpath_expr}" position="{position}">\n    {field_xml}\n  </xpath>'
        arch_xml = f"""<data>
  {xpath_element}
</data>"""

        view_values = {
            "name": f"Odoo Studio: {base_view_name} customization",
            "model": model,
            "type": view_type,
            "mode": "extension",
            "inherit_id": base_view_id,
            "arch_db": arch_xml,
            "priority": 99
        }

        view_id = api.call('ir.ui.view', 'create', [view_values])

        if not view_id:
            raise OdooAPIError("Failed to create inherited view")

        log_success(f"Created new Studio view (ID: {view_id})")

        # Register in ir.model.data only for new views
        log_info("Registering Studio metadata...")
        uuid_suffix = api.generate_studio_uuid()
        api.call('ir.model.data', 'create', [{
            'name': f"view_{model.replace('.', '_')}_{view_type}_{uuid_suffix}",
            'module': 'studio_customization',
            'model': 'ir.ui.view',
            'res_id': view_id,
            'studio': True
        }])

    log_success(f"Field '{field_name}' added to {model} {view_type} view")
    return view_id


def main():
    parser = argparse.ArgumentParser(
        description='Add a field to an Odoo view using Studio-style inheritance'
    )
    parser.add_argument('model', help='Model of the view (e.g., res.partner)')
    parser.add_argument('view_type', help='View type: form, tree, kanban, search')
    parser.add_argument('field_name', help='Field to add (e.g., x_studio_my_field)')
    parser.add_argument('position', help='Position: after, before, inside')
    parser.add_argument('reference', help='Field or element to position relative to')
    parser.add_argument('attrs', nargs='?', default='{}', help='JSON attributes')

    args = parser.parse_args()

    try:
        attrs = json.loads(args.attrs)
        view_id = add_to_view(
            args.model, args.view_type, args.field_name,
            args.position, args.reference, attrs
        )
        print(view_id)  # Output view ID to stdout
        sys.exit(0)
    except json.JSONDecodeError as e:
        log_error(f"Invalid JSON attributes: {e}")
        sys.exit(1)
    except OdooAPIError as e:
        log_error(str(e))
        sys.exit(1)
    except Exception as e:
        log_error(str(e))
        sys.exit(1)


if __name__ == '__main__':
    main()
