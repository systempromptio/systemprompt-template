#!/usr/bin/env python3
"""
Export Studio customizations as an Odoo module.

Usage:
    export_customizations.py <model> --output <directory> [--module-name <name>]

Examples:
    export_customizations.py crm.lead --output ./my_crm_customization
    export_customizations.py res.partner --output ./partner_fields --module-name partner_custom_fields
"""

import sys
import os
import json
import argparse
from datetime import datetime
from odoo_api import OdooAPI, OdooAPIError

# Colors
RED = '\033[0;31m'
GREEN = '\033[0;32m'
YELLOW = '\033[1;33m'
NC = '\033[0m'


def log_error(msg):
    print(f"{RED}Error: {msg}{NC}", file=sys.stderr)


def log_success(msg):
    print(f"{GREEN}{msg}{NC}")


def log_info(msg):
    print(f"{YELLOW}{msg}{NC}")


def sanitize_module_name(name: str) -> str:
    """Convert to valid Odoo module name."""
    return name.lower().replace('.', '_').replace('-', '_').replace(' ', '_')


def field_type_to_python(ttype: str) -> str:
    """Convert Odoo field type to Python fields import."""
    mapping = {
        'char': 'Char',
        'text': 'Text',
        'html': 'Html',
        'integer': 'Integer',
        'float': 'Float',
        'monetary': 'Monetary',
        'boolean': 'Boolean',
        'date': 'Date',
        'datetime': 'Datetime',
        'binary': 'Binary',
        'selection': 'Selection',
        'many2one': 'Many2one',
        'one2many': 'One2many',
        'many2many': 'Many2many',
    }
    return mapping.get(ttype, 'Char')


class ModuleExporter:
    """Export Studio customizations as Odoo module."""

    def __init__(self, api: OdooAPI):
        self.api = api

    def get_studio_fields(self, model: str) -> list:
        """Get all Studio fields for a model."""
        return self.api.call('ir.model.fields', 'search_read', [
            [['model', '=', model], ['name', 'like', 'x_studio_%']]
        ], {'fields': [
            'name', 'field_description', 'ttype', 'required', 'readonly',
            'help', 'relation', 'relation_field', 'selection', 'currency_field',
            'domain', 'copied', 'store'
        ]})

    def get_studio_views(self, model: str) -> list:
        """Get all Studio views for a model."""
        return self.api.call('ir.ui.view', 'search_read', [
            [['model', '=', model], ['name', 'ilike', 'studio']]
        ], {'fields': ['name', 'type', 'arch_db', 'inherit_id', 'priority', 'mode']})

    def generate_manifest(self, module_name: str, model: str, fields_count: int, views_count: int) -> str:
        """Generate __manifest__.py content."""
        return f'''# -*- coding: utf-8 -*-
{{
    'name': '{module_name.replace("_", " ").title()}',
    'version': '18.0.1.0.0',
    'category': 'Customizations',
    'summary': 'Studio customizations for {model}',
    'description': """
Studio Customizations Export
============================

Exported from Odoo Studio on {datetime.now().strftime('%Y-%m-%d')}.

Contains:
- {fields_count} custom fields
- {views_count} view customizations

Model: {model}
    """,
    'author': 'Studio Export',
    'depends': ['base'],
    'data': [
        'views/{model.replace(".", "_")}_views.xml',
    ],
    'installable': True,
    'application': False,
    'auto_install': False,
    'license': 'LGPL-3',
}}
'''

    def generate_init(self) -> str:
        """Generate __init__.py content."""
        return '''# -*- coding: utf-8 -*-
from . import models
'''

    def generate_models_init(self, model: str) -> str:
        """Generate models/__init__.py content."""
        model_file = model.replace('.', '_')
        return f'''# -*- coding: utf-8 -*-
from . import {model_file}
'''

    def generate_model_file(self, model: str, fields: list) -> str:
        """Generate Python model file with fields."""
        model_file = model.replace('.', '_')
        model_class = ''.join(word.title() for word in model.split('.'))

        # Collect field types needed
        field_types = set()
        for field in fields:
            field_types.add(field_type_to_python(field['ttype']))

        imports = ', '.join(sorted(field_types))

        lines = [
            '# -*- coding: utf-8 -*-',
            f'from odoo import models, fields',
            '',
            '',
            f"class {model_class}(models.Model):",
            f"    _inherit = '{model}'",
            '',
        ]

        for field in fields:
            field_def = self._generate_field_definition(field)
            lines.append(f"    {field['name']} = {field_def}")

        return '\n'.join(lines) + '\n'

    def _generate_field_definition(self, field: dict) -> str:
        """Generate a single field definition."""
        ftype = field_type_to_python(field['ttype'])
        args = [f"string='{field['field_description']}'"]

        if field.get('required'):
            args.append('required=True')
        if field.get('readonly'):
            args.append('readonly=True')
        if field.get('help'):
            args.append(f"help='{field['help']}'")
        if not field.get('copied', True):
            args.append('copy=False')

        # Type-specific args
        if field['ttype'] == 'selection' and field.get('selection'):
            args.append(f"selection={field['selection']}")
        if field['ttype'] in ('many2one', 'many2many', 'one2many') and field.get('relation'):
            args.append(f"comodel_name='{field['relation']}'")
        if field['ttype'] == 'one2many' and field.get('relation_field'):
            args.append(f"inverse_name='{field['relation_field']}'")
        if field['ttype'] == 'monetary' and field.get('currency_field'):
            args.append(f"currency_field='{field['currency_field']}'")
        if field.get('domain'):
            args.append(f"domain={field['domain']}")

        return f"fields.{ftype}({', '.join(args)})"

    def generate_views_file(self, model: str, views: list) -> str:
        """Generate XML views file."""
        model_file = model.replace('.', '_')

        lines = [
            '<?xml version="1.0" encoding="utf-8"?>',
            '<odoo>',
        ]

        for view in views:
            inherit_id = view['inherit_id'][1] if view['inherit_id'] else None
            inherit_ref = f"ref('{inherit_id}')" if inherit_id else ''

            # Clean up arch - remove <data> wrapper if present
            arch = view['arch_db']
            if arch.strip().startswith('<data>'):
                # Extract xpath content
                import re
                xpaths = re.findall(r'<xpath[^>]*>.*?</xpath>', arch, re.DOTALL)
                arch_content = '\n            '.join(xpaths)
            else:
                arch_content = arch

            view_id = f"view_{model_file}_{view['type']}_studio_{view['id']}"

            lines.extend([
                '',
                f"    <!-- {view['name']} -->",
                f"    <record id=\"{view_id}\" model=\"ir.ui.view\">",
                f"        <field name=\"name\">{view['name']}</field>",
                f"        <field name=\"model\">{model}</field>",
                f"        <field name=\"inherit_id\" ref=\"{inherit_ref}\"/>",
                f"        <field name=\"priority\">{view.get('priority', 99)}</field>",
                f"        <field name=\"arch\" type=\"xml\">",
                f"            <data>",
                f"                {arch_content}",
                f"            </data>",
                f"        </field>",
                f"    </record>",
            ])

        lines.extend([
            '',
            '</odoo>',
        ])

        return '\n'.join(lines)

    def export(self, model: str, output_dir: str, module_name: str = None) -> dict:
        """Export customizations to module directory."""
        if not module_name:
            module_name = f"studio_{sanitize_module_name(model)}"

        log_info(f"Exporting customizations for {model}...")

        # Get data
        fields = self.get_studio_fields(model)
        views = self.get_studio_views(model)

        log_info(f"Found {len(fields)} fields and {len(views)} views")

        if not fields and not views:
            log_error("No Studio customizations found for this model")
            return {'success': False, 'error': 'No customizations found'}

        # Create directory structure
        module_dir = os.path.join(output_dir, module_name)
        models_dir = os.path.join(module_dir, 'models')
        views_dir = os.path.join(module_dir, 'views')

        os.makedirs(models_dir, exist_ok=True)
        os.makedirs(views_dir, exist_ok=True)

        # Generate files
        model_file = model.replace('.', '_')

        # __manifest__.py
        with open(os.path.join(module_dir, '__manifest__.py'), 'w') as f:
            f.write(self.generate_manifest(module_name, model, len(fields), len(views)))
        log_success(f"Created __manifest__.py")

        # __init__.py
        with open(os.path.join(module_dir, '__init__.py'), 'w') as f:
            f.write(self.generate_init())
        log_success(f"Created __init__.py")

        # models/__init__.py
        with open(os.path.join(models_dir, '__init__.py'), 'w') as f:
            f.write(self.generate_models_init(model))
        log_success(f"Created models/__init__.py")

        # models/<model>.py
        if fields:
            with open(os.path.join(models_dir, f'{model_file}.py'), 'w') as f:
                f.write(self.generate_model_file(model, fields))
            log_success(f"Created models/{model_file}.py ({len(fields)} fields)")

        # views/<model>_views.xml
        if views:
            with open(os.path.join(views_dir, f'{model_file}_views.xml'), 'w') as f:
                f.write(self.generate_views_file(model, views))
            log_success(f"Created views/{model_file}_views.xml ({len(views)} views)")

        log_success(f"\nModule exported to: {module_dir}")

        return {
            'success': True,
            'module_name': module_name,
            'module_path': module_dir,
            'fields_count': len(fields),
            'views_count': len(views)
        }


def main():
    parser = argparse.ArgumentParser(
        description='Export Studio customizations as Odoo module'
    )
    parser.add_argument('model', help='Model name (e.g., crm.lead)')
    parser.add_argument('--output', '-o', required=True, help='Output directory')
    parser.add_argument('--module-name', '-n', help='Module name (default: studio_<model>)')

    args = parser.parse_args()

    try:
        api = OdooAPI()
        exporter = ModuleExporter(api)
        result = exporter.export(args.model, args.output, args.module_name)

        if args.module_name:
            print(json.dumps(result, indent=2))

        sys.exit(0 if result['success'] else 1)

    except OdooAPIError as e:
        log_error(str(e))
        sys.exit(1)
    except Exception as e:
        log_error(str(e))
        sys.exit(1)


if __name__ == '__main__':
    main()
