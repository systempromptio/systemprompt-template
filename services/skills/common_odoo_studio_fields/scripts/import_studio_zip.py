#!/usr/bin/env python3
"""
Automated import of Odoo Studio customization zip files.

Validates the zip, imports via base.import.module wizard, and optionally
falls back to file-by-file import when the full import fails.

Usage:
    import_studio_zip.py <zip_path>
    import_studio_zip.py <zip_path> --force
    import_studio_zip.py <zip_path> --validate-only
    import_studio_zip.py <zip_path> --file-by-file
    import_studio_zip.py <zip_path> --skip-validation --force

Examples:
    import_studio_zip.py customization.zip
    import_studio_zip.py customization.zip --force --json
    import_studio_zip.py customization_fixed.zip --file-by-file
"""

import sys
import os
import json
import argparse
import base64
import zipfile
import ast
import tempfile
import shutil
import xml.etree.ElementTree as ET
from typing import Optional

# Add scripts dir to path
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from odoo_api import OdooAPI, OdooAPIError

# Colors
RED = '\033[0;31m'
GREEN = '\033[0;32m'
YELLOW = '\033[1;33m'
BLUE = '\033[0;34m'
NC = '\033[0m'


def log_error(msg):
    print(f"{RED}\u2717 {msg}{NC}", file=sys.stderr)


def log_success(msg):
    print(f"{GREEN}\u2713 {msg}{NC}", file=sys.stderr)


def log_warning(msg):
    print(f"{YELLOW}\u26a0 {msg}{NC}", file=sys.stderr)


def log_info(msg):
    print(f"{BLUE}\u2139 {msg}{NC}", file=sys.stderr)


class StudioZipImporter:
    """Imports an Odoo Studio customization zip into a target instance."""

    def __init__(self, zip_path: str, api: OdooAPI, force: bool = False):
        self.zip_path = zip_path
        self.api = api
        self.force = force
        self.module_name = ''
        self.manifest = {}
        self._detect_module_name()

    def _detect_module_name(self):
        """Detect the module name and manifest from the zip."""
        with zipfile.ZipFile(self.zip_path, 'r') as zf:
            names = zf.namelist()
            top_dirs = set()
            for name in names:
                parts = name.split('/')
                if len(parts) > 1 and parts[0]:
                    top_dirs.add(parts[0])

            if len(top_dirs) == 1:
                self.module_name = top_dirs.pop()

            manifest_path = f"{self.module_name}/__manifest__.py"
            if manifest_path in names:
                content = zf.read(manifest_path).decode('utf-8')
                try:
                    self.manifest = ast.literal_eval(content)
                except (SyntaxError, ValueError):
                    pass

    def validate(self, online: bool = True) -> dict:
        """Run ZipValidator and return results."""
        from validate_zip import ZipValidator
        validator = ZipValidator(self.zip_path, online=online)
        result = validator.validate()
        return result.to_dict()

    def import_via_wizard(self) -> dict:
        """
        Import the zip via base.import.module transient model.

        This is the standard Odoo import method, equivalent to:
        Settings > Technical > Base Import Module
        """
        log_info(f"Importing {os.path.basename(self.zip_path)} via base.import.module...")
        log_info(f"Module: {self.module_name} | Force: {self.force}")

        # Read and encode the zip
        with open(self.zip_path, 'rb') as f:
            zip_b64 = base64.b64encode(f.read()).decode()

        # Step 1: Create the import wizard
        try:
            wizard_id = self.api.call('base.import.module', 'create', [{
                'module_file': zip_b64,
                'state': 'init',
                'force': self.force,
                'with_demo': False,
            }])
        except OdooAPIError as e:
            return {
                'success': False,
                'method': 'wizard',
                'error': f"Failed to create import wizard: {e}",
            }

        log_info(f"Wizard created (id={wizard_id}), executing import...")

        # Step 2: Execute the import
        try:
            result = self.api.call('base.import.module', 'import_module',
                                   [[wizard_id]])
        except OdooAPIError as e:
            error_msg = str(e)
            # Detect transaction abort (masks real error)
            if 'transaccion abortada' in error_msg.lower() or \
               'InFailedSqlTransaction' in error_msg or \
               'current transaction is aborted' in error_msg.lower():
                return {
                    'success': False,
                    'method': 'wizard',
                    'error': error_msg,
                    'hint': 'Transaction was aborted by a previous error. '
                            'Check Odoo server logs for the real error. '
                            'Common causes: duplicate XML ID (UniqueViolation), '
                            'missing field, or invalid view arch. '
                            'Try --file-by-file to isolate the failing file.',
                }
            return {
                'success': False,
                'method': 'wizard',
                'error': error_msg,
            }

        # Step 3: Verify import
        verification = self._verify_import()

        return {
            'success': True,
            'method': 'wizard',
            'module': self.module_name,
            'wizard_result': result,
            'verification': verification,
        }

    def import_file_by_file(self) -> dict:
        """
        Split the zip into single-file zips and import each one separately.

        This isolates errors to individual files, preventing one bad file
        from aborting the entire import via transaction rollback.
        """
        log_info(f"File-by-file import of {os.path.basename(self.zip_path)}...")

        if not self.manifest:
            return {
                'success': False,
                'method': 'file_by_file',
                'error': 'Cannot read manifest from zip',
            }

        data_files = self.manifest.get('data', [])
        if not data_files:
            return {
                'success': False,
                'method': 'file_by_file',
                'error': 'No data files listed in manifest',
            }

        tmpdir = tempfile.mkdtemp(prefix='studio_import_fbf_')
        results = {'imported': [], 'failed': [], 'skipped': []}

        try:
            # Extract original zip
            with zipfile.ZipFile(self.zip_path, 'r') as zf:
                zf.extractall(tmpdir)

            module_dir = os.path.join(tmpdir, self.module_name)
            manifest_path = os.path.join(module_dir, '__manifest__.py')

            # Read original manifest content
            with open(manifest_path, 'r', encoding='utf-8') as f:
                original_manifest = f.read()

            for i, data_file in enumerate(data_files):
                log_info(f"[{i+1}/{len(data_files)}] Importing {data_file}...")

                # Check file exists
                full_path = os.path.join(module_dir, data_file)
                if not os.path.exists(full_path):
                    results['skipped'].append({
                        'file': data_file,
                        'reason': 'File not found in zip',
                    })
                    log_warning(f"  Skipped: file not found")
                    continue

                # Count records
                record_count = 0
                if data_file.endswith('.xml'):
                    try:
                        tree = ET.parse(full_path)
                        record_count = len(tree.getroot().findall('.//record'))
                    except ET.ParseError:
                        results['failed'].append({
                            'file': data_file,
                            'error': 'XML parse error',
                        })
                        log_error(f"  Failed: XML parse error")
                        continue

                # Create single-file manifest
                single_manifest = self.manifest.copy()
                single_manifest['data'] = [data_file]

                import re
                modified_manifest = re.sub(
                    r"'data'\s*:\s*\[.*?\]",
                    f"'data': ['{data_file}']",
                    original_manifest,
                    flags=re.DOTALL,
                )

                with open(manifest_path, 'w', encoding='utf-8') as f:
                    f.write(modified_manifest)

                # Create single-file zip
                single_zip_path = os.path.join(tmpdir, f'import_{i}.zip')
                with zipfile.ZipFile(single_zip_path, 'w', zipfile.ZIP_DEFLATED) as zout:
                    # Always include manifest, __init__.py
                    zout.write(manifest_path, f'{self.module_name}/__manifest__.py')
                    init_path = os.path.join(module_dir, '__init__.py')
                    if os.path.exists(init_path):
                        zout.write(init_path, f'{self.module_name}/__init__.py')
                    # Include the single data file
                    zout.write(full_path, f'{self.module_name}/{data_file}')

                # Import via wizard
                try:
                    with open(single_zip_path, 'rb') as f:
                        zip_b64 = base64.b64encode(f.read()).decode()

                    wizard_id = self.api.call('base.import.module', 'create', [{
                        'module_file': zip_b64,
                        'state': 'init',
                        'force': self.force,
                        'with_demo': False,
                    }])

                    self.api.call('base.import.module', 'import_module',
                                  [[wizard_id]])

                    results['imported'].append({
                        'file': data_file,
                        'records': record_count,
                    })
                    log_success(f"  OK ({record_count} records)")

                except OdooAPIError as e:
                    results['failed'].append({
                        'file': data_file,
                        'error': str(e),
                        'records': record_count,
                    })
                    log_error(f"  Failed: {e}")

            # Restore original manifest
            with open(manifest_path, 'w', encoding='utf-8') as f:
                f.write(original_manifest)

        finally:
            shutil.rmtree(tmpdir, ignore_errors=True)

        # Verify
        verification = self._verify_import()

        total = len(data_files)
        ok = len(results['imported'])
        failed = len(results['failed'])
        skipped = len(results['skipped'])

        log_info(f"\nResults: {ok}/{total} imported, {failed} failed, {skipped} skipped")

        return {
            'success': failed == 0,
            'method': 'file_by_file',
            'module': self.module_name,
            'total_files': total,
            'imported': results['imported'],
            'failed': results['failed'],
            'skipped': results['skipped'],
            'verification': verification,
        }

    def _verify_import(self) -> dict:
        """Verify the import by checking ir.model.data and module state."""
        verification = {}

        try:
            # Check module state
            modules = self.api.call('ir.module.module', 'search_read', [
                [['name', '=', self.module_name]]
            ], {'fields': ['state', 'latest_version'], 'limit': 1})

            if modules:
                verification['module_state'] = modules[0]['state']
                verification['module_version'] = modules[0].get('latest_version', '')
            else:
                verification['module_state'] = 'not_found'

            # Count ir.model.data records
            total = self.api.call('ir.model.data', 'search_count', [
                [['module', '=', self.module_name]]
            ])
            verification['ir_model_data_count'] = total

            # Count by model type
            for model_type in ['ir.model.fields', 'ir.ui.view', 'ir.ui.menu',
                               'ir.model.access', 'ir.model']:
                count = self.api.call('ir.model.data', 'search_count', [
                    [['module', '=', self.module_name], ['model', '=', model_type]]
                ])
                key = model_type.replace('.', '_') + '_count'
                verification[key] = count

        except OdooAPIError as e:
            verification['error'] = str(e)

        return verification


# ==================== CLI ====================


def main():
    parser = argparse.ArgumentParser(
        description='Import an Odoo Studio customization zip into a target instance'
    )
    parser.add_argument('zip_path', help='Path to the customization zip file')
    parser.add_argument('--force', action='store_true',
                        help='Force import (overwrite existing data)')
    parser.add_argument('--validate-only', action='store_true',
                        help='Only validate, do not import')
    parser.add_argument('--skip-validation', action='store_true',
                        help='Skip pre-import validation')
    parser.add_argument('--file-by-file', action='store_true',
                        help='Import each data file separately (isolates errors)')
    parser.add_argument('--json', action='store_true',
                        help='Output results as JSON')

    args = parser.parse_args()

    # Check zip exists
    if not os.path.exists(args.zip_path):
        log_error(f"File not found: {args.zip_path}")
        sys.exit(1)

    try:
        api = OdooAPI()
    except OdooAPIError as e:
        log_error(f"Cannot connect to Odoo: {e}")
        sys.exit(1)

    importer = StudioZipImporter(args.zip_path, api, force=args.force)

    # Step 1: Validate
    if not args.skip_validation:
        log_info("Running pre-import validation...")
        validation = importer.validate(online=True)

        if args.json:
            if args.validate_only:
                print(json.dumps(validation, indent=2))
                sys.exit(0 if validation.get('errors', 0) == 0 else 1)
        else:
            errors = validation.get('errors', 0)
            warnings = validation.get('warnings', 0)
            if errors:
                log_error(f"Validation failed: {errors} error(s), {warnings} warning(s)")
                if not args.json:
                    for issue in validation.get('issues', []):
                        if issue.get('severity') == 'error':
                            log_error(f"  [{issue['code']}] {issue['message']}")
                if not args.force:
                    log_info("Use --force to import anyway, or fix the issues first")
                    log_info("Use validate_zip.py --fix to auto-fix common issues")
                    if args.json:
                        print(json.dumps({'success': False, 'validation': validation}, indent=2))
                    sys.exit(1)
                else:
                    log_warning("Proceeding with import despite validation errors (--force)")
            elif warnings:
                log_warning(f"Validation passed with {warnings} warning(s)")
            else:
                log_success("Validation passed")

        if args.validate_only:
            sys.exit(0)

    # Step 2: Import
    if args.file_by_file:
        result = importer.import_file_by_file()
    else:
        result = importer.import_via_wizard()

    # Output
    if args.json:
        print(json.dumps(result, indent=2))
    else:
        if result['success']:
            log_success(f"Import successful: {result.get('module', '')}")
            v = result.get('verification', {})
            if v:
                log_info(f"  Module state: {v.get('module_state', '?')}")
                log_info(f"  ir.model.data records: {v.get('ir_model_data_count', '?')}")
                log_info(f"  Fields: {v.get('ir_model_fields_count', '?')}")
                log_info(f"  Views: {v.get('ir_ui_view_count', '?')}")
        else:
            log_error(f"Import failed: {result.get('error', 'Unknown error')}")
            hint = result.get('hint')
            if hint:
                log_info(f"Hint: {hint}")

    sys.exit(0 if result.get('success') else 1)


if __name__ == '__main__':
    main()
