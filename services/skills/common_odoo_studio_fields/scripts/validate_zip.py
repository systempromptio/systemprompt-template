#!/usr/bin/env python3
"""
Pre-import validator for Odoo Studio customization zip files.

Performs offline checks (zip structure, XML well-formedness, deprecated fields,
stray files, manifest consistency) and optional online checks (XML ID conflicts,
dependency availability, model/field existence on target instance).

Usage:
    validate_zip.py <zip_path>
    validate_zip.py <zip_path> --online
    validate_zip.py <zip_path> --fix -o /tmp/fixed.zip
    validate_zip.py <zip_path> --json

Examples:
    validate_zip.py customization.zip
    validate_zip.py customization.zip --online --json
    validate_zip.py customization.zip --fix -o /tmp/customization_fixed.zip
"""

import sys
import os
import json
import argparse
import zipfile
import ast
import re
import copy
import shutil
import tempfile
import xml.etree.ElementTree as ET
from dataclasses import dataclass, field, asdict
from typing import List, Optional
from pathlib import Path

# Add scripts dir to path for odoo_api import
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

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


# ==================== Data Classes ====================


@dataclass
class ValidationIssue:
    """A single validation issue found in the zip."""
    severity: str  # 'error', 'warning', 'info'
    code: str      # machine-readable code (e.g., 'DEPRECATED_FIELD')
    message: str   # human-readable description
    file: str = ''
    line: int = 0
    fix: str = ''  # description of auto-fix if available

    def to_dict(self):
        d = asdict(self)
        # Remove empty optional fields
        return {k: v for k, v in d.items() if v}


@dataclass
class ValidationResult:
    """Aggregated validation results."""
    valid: bool = True
    zip_path: str = ''
    module_name: str = ''
    issues: List[ValidationIssue] = field(default_factory=list)
    manifest: dict = field(default_factory=dict)
    file_count: int = 0
    record_count: int = 0

    @property
    def errors(self):
        return [i for i in self.issues if i.severity == 'error']

    @property
    def warnings(self):
        return [i for i in self.issues if i.severity == 'warning']

    def to_dict(self):
        return {
            'valid': self.valid,
            'zip_path': self.zip_path,
            'module_name': self.module_name,
            'errors': len(self.errors),
            'warnings': len(self.warnings),
            'issues': [i.to_dict() for i in self.issues],
            'file_count': self.file_count,
            'record_count': self.record_count,
        }


# ==================== Zip Validator ====================


# Deprecated/removed fields by Odoo version
DEPRECATED_FIELDS = {
    '__last_update': {
        'removed_in': '18.0',
        'reason': 'Magic field removed in Odoo 18',
    },
}

# Files that should never be in a Studio zip
STRAY_FILE_PATTERNS = [
    'Untitled',
    '*.pyc',
    '__pycache__',
    '.DS_Store',
    'Thumbs.db',
    '*.swp',
    '*.swo',
    '*~',
    '.gitkeep',
]


class ZipValidator:
    """Validates an Odoo Studio customization zip before import."""

    def __init__(self, zip_path: str, online: bool = False):
        self.zip_path = zip_path
        self.online = online
        self.issues: List[ValidationIssue] = []
        self.module_name = ''
        self.manifest = {}
        self.manifest_data_files = []
        self.zip_files = []
        self.xml_records = {}  # file -> list of (xml_id, model, record_element)
        self.api = None

    def _add_issue(self, severity: str, code: str, message: str,
                   file: str = '', line: int = 0, fix: str = ''):
        self.issues.append(ValidationIssue(
            severity=severity, code=code, message=message,
            file=file, line=line, fix=fix,
        ))

    def validate(self) -> ValidationResult:
        """Run all validation checks. Returns structured results."""
        result = ValidationResult(zip_path=self.zip_path)

        # Offline checks (no Odoo connection)
        if not self._check_zip_readable():
            result.valid = False
            result.issues = self.issues
            return result

        self._check_zip_structure()
        self._check_manifest()
        self._check_manifest_data_sync()
        self._check_xml_wellformed()
        self._check_deprecated_fields()
        self._check_stray_files()
        self._check_duplicate_xml_ids()
        self._check_full_arch_replacement()

        # Online checks (require Odoo connection)
        if self.online:
            self._init_api()
            if self.api:
                self._check_xml_id_conflicts()
                self._check_dependencies_installed()
                self._check_referenced_models()
                self._check_inherit_id_references()
                self._check_web_studio_installed()

        # Build result
        result.module_name = self.module_name
        result.manifest = self.manifest
        result.issues = self.issues
        result.file_count = len(self.zip_files)
        result.record_count = sum(len(recs) for recs in self.xml_records.values())
        result.valid = len([i for i in self.issues if i.severity == 'error']) == 0

        return result

    # ==================== Offline Checks ====================

    def _check_zip_readable(self) -> bool:
        """Check that the file exists and is a valid zip."""
        if not os.path.exists(self.zip_path):
            self._add_issue('error', 'ZIP_NOT_FOUND',
                            f"File not found: {self.zip_path}")
            return False

        if not zipfile.is_zipfile(self.zip_path):
            self._add_issue('error', 'ZIP_INVALID',
                            f"Not a valid zip file: {self.zip_path}")
            return False

        try:
            with zipfile.ZipFile(self.zip_path, 'r') as zf:
                bad = zf.testzip()
                if bad:
                    self._add_issue('error', 'ZIP_CORRUPT',
                                    f"Corrupt file in zip: {bad}")
                    return False
                self.zip_files = zf.namelist()
        except zipfile.BadZipFile as e:
            self._add_issue('error', 'ZIP_CORRUPT', f"Corrupt zip: {e}")
            return False

        return True

    def _check_zip_structure(self):
        """Check zip contains a single module directory with __manifest__.py."""
        # Find top-level directories
        top_dirs = set()
        for name in self.zip_files:
            parts = name.split('/')
            if len(parts) > 1 and parts[0]:
                top_dirs.add(parts[0])

        if len(top_dirs) == 0:
            self._add_issue('error', 'ZIP_EMPTY', 'Zip contains no module directory')
            return
        if len(top_dirs) > 1:
            self._add_issue('error', 'ZIP_MULTI_MODULE',
                            f"Zip contains multiple top-level directories: {', '.join(sorted(top_dirs))}. "
                            "Expected a single module.")
            return

        self.module_name = top_dirs.pop()

        # Check for __manifest__.py
        manifest_path = f"{self.module_name}/__manifest__.py"
        if manifest_path not in self.zip_files:
            self._add_issue('error', 'MANIFEST_MISSING',
                            f"No __manifest__.py found in {self.module_name}/")

    def _check_manifest(self):
        """Parse and validate __manifest__.py."""
        manifest_path = f"{self.module_name}/__manifest__.py"
        if manifest_path not in self.zip_files:
            return

        try:
            with zipfile.ZipFile(self.zip_path, 'r') as zf:
                content = zf.read(manifest_path).decode('utf-8')
                self.manifest = ast.literal_eval(content)
        except (SyntaxError, ValueError) as e:
            self._add_issue('error', 'MANIFEST_PARSE_ERROR',
                            f"Cannot parse __manifest__.py: {e}",
                            file=manifest_path)
            return

        if not isinstance(self.manifest, dict):
            self._add_issue('error', 'MANIFEST_INVALID',
                            "Manifest is not a dictionary",
                            file=manifest_path)
            return

        self.manifest_data_files = self.manifest.get('data', [])

        # Check required keys
        if 'name' not in self.manifest:
            self._add_issue('warning', 'MANIFEST_NO_NAME',
                            "Manifest missing 'name' key",
                            file=manifest_path)
        if 'depends' not in self.manifest:
            self._add_issue('warning', 'MANIFEST_NO_DEPENDS',
                            "Manifest missing 'depends' key",
                            file=manifest_path)

        log_success(f"Manifest parsed: {self.manifest.get('name', 'Unknown')} "
                    f"v{self.manifest.get('version', '?')}")

    def _check_manifest_data_sync(self):
        """Check that manifest data list matches actual XML/CSV files in zip."""
        if not self.module_name:
            return

        # Files in manifest data list
        manifest_files = set(self.manifest_data_files)

        # Actual data files in zip (XML and CSV under data/)
        actual_data_files = set()
        for name in self.zip_files:
            if name.startswith(f"{self.module_name}/") and not name.endswith('/'):
                rel_path = name[len(self.module_name) + 1:]
                if rel_path.startswith('data/') and rel_path.endswith(('.xml', '.csv')):
                    actual_data_files.add(rel_path)

        # Files in zip but not in manifest
        missing_from_manifest = actual_data_files - manifest_files
        for f in sorted(missing_from_manifest):
            self._add_issue('error', 'MANIFEST_MISSING_ENTRY',
                            f"Data file '{f}' exists in zip but is NOT listed in manifest 'data'",
                            file=f"{self.module_name}/__manifest__.py",
                            fix=f"Add '{f}' to manifest data list")

        # Files in manifest but not in zip
        missing_from_zip = manifest_files - actual_data_files
        for f in sorted(missing_from_zip):
            # Only flag XML/CSV data files, not security/ or other paths
            if f.endswith(('.xml', '.csv')):
                self._add_issue('error', 'MANIFEST_EXTRA_ENTRY',
                                f"Manifest lists '{f}' but file does not exist in zip",
                                file=f"{self.module_name}/__manifest__.py",
                                fix=f"Remove '{f}' from manifest data list")

    def _check_xml_wellformed(self):
        """Parse all XML files and check well-formedness."""
        with zipfile.ZipFile(self.zip_path, 'r') as zf:
            for name in self.zip_files:
                if not name.endswith('.xml'):
                    continue
                rel_path = name[len(self.module_name) + 1:] if name.startswith(f"{self.module_name}/") else name
                try:
                    content = zf.read(name)
                    if not content.strip():
                        self._add_issue('warning', 'XML_EMPTY',
                                        f"XML file is empty: {rel_path}",
                                        file=name)
                        continue
                    root = ET.fromstring(content)
                    # Collect records
                    records = []
                    for rec in root.iter('record'):
                        xml_id = rec.get('id', '')
                        model = rec.get('model', '')
                        records.append((xml_id, model, rec))
                    self.xml_records[rel_path] = records
                except ET.ParseError as e:
                    self._add_issue('error', 'XML_PARSE_ERROR',
                                    f"XML parse error in {rel_path}: {e}",
                                    file=name)

    def _check_deprecated_fields(self):
        """Detect deprecated/removed fields in view XML."""
        for rel_path, records in self.xml_records.items():
            for xml_id, model, rec in records:
                if model != 'ir.ui.view':
                    continue
                # Get the arch content
                arch_field = rec.find("field[@name='arch']")
                if arch_field is None:
                    continue
                arch_text = ET.tostring(arch_field, encoding='unicode', method='xml')
                for dep_field, info in DEPRECATED_FIELDS.items():
                    if f'name="{dep_field}"' in arch_text or f"name='{dep_field}'" in arch_text:
                        self._add_issue('error', 'DEPRECATED_FIELD',
                                        f"Deprecated field '{dep_field}' in view {xml_id} "
                                        f"(removed in Odoo {info['removed_in']}): {info['reason']}",
                                        file=rel_path,
                                        fix=f"Remove <field name=\"{dep_field}\"/> from the view arch")

    def _check_stray_files(self):
        """Detect stray/garbage files that should not be in the zip."""
        import fnmatch
        for name in self.zip_files:
            if name.endswith('/'):
                continue  # skip directories
            basename = os.path.basename(name)
            for pattern in STRAY_FILE_PATTERNS:
                if fnmatch.fnmatch(basename, pattern):
                    self._add_issue('warning', 'STRAY_FILE',
                                    f"Stray file detected: {name}",
                                    file=name,
                                    fix=f"Remove '{name}' from the zip")
                    break

    def _check_duplicate_xml_ids(self):
        """Check for duplicate XML IDs within the zip."""
        all_ids = {}  # xml_id -> file
        for rel_path, records in self.xml_records.items():
            for xml_id, model, rec in records:
                if not xml_id:
                    continue
                # Normalize: add module prefix if not present
                full_id = xml_id if '.' in xml_id else f"{self.module_name}.{xml_id}"
                if full_id in all_ids:
                    self._add_issue('error', 'DUPLICATE_XML_ID',
                                    f"Duplicate XML ID '{full_id}' in {rel_path} "
                                    f"(already defined in {all_ids[full_id]})",
                                    file=rel_path)
                else:
                    all_ids[full_id] = rel_path

    def _check_full_arch_replacement(self):
        """Detect full view arch replacements (non-inherited views that overwrite base views)."""
        for rel_path, records in self.xml_records.items():
            for xml_id, model, rec in records:
                if model != 'ir.ui.view':
                    continue
                # Check if this is an external XML ID from another module
                if '.' in xml_id:
                    module_part = xml_id.split('.')[0]
                    if module_part != self.module_name:
                        # This record overwrites an external view
                        # Check if it has inherit_id - if not, it's a full replacement
                        inherit_field = rec.find("field[@name='inherit_id']")
                        arch_field = rec.find("field[@name='arch']")
                        if inherit_field is None and arch_field is not None:
                            arch_text = ET.tostring(arch_field, encoding='unicode')
                            # Full replacement if arch doesn't use xpath
                            if '<xpath' not in arch_text:
                                self._add_issue('error', 'FULL_ARCH_REPLACEMENT',
                                                f"View '{xml_id}' is a full arch replacement of an external view. "
                                                f"This will overwrite the original view and break inherited views.",
                                                file=rel_path,
                                                fix=f"Remove the record '{xml_id}' or convert to an extension view with xpath")

    # ==================== Online Checks ====================

    def _init_api(self):
        """Initialize Odoo API connection for online checks."""
        try:
            from odoo_api import OdooAPI
            self.api = OdooAPI()
            log_success("Connected to Odoo instance")
        except Exception as e:
            self._add_issue('warning', 'API_CONNECTION_FAILED',
                            f"Cannot connect to Odoo for online checks: {e}",
                            fix="Set ODOO_URL, ODOO_DB, ODOO_USER, ODOO_KEY environment variables")
            self.api = None

    def _check_xml_id_conflicts(self):
        """Check for XML ID conflicts with existing ir.model.data records."""
        if not self.api:
            return

        log_info("Checking XML ID conflicts with target instance...")
        conflicts = 0

        for rel_path, records in self.xml_records.items():
            for xml_id, model, rec in records:
                if not xml_id or not model:
                    continue

                # Parse module.name from xml_id
                if '.' in xml_id:
                    id_module, id_name = xml_id.split('.', 1)
                else:
                    id_module = self.module_name
                    id_name = xml_id

                # Look up in ir.model.data
                try:
                    existing = self.api.call('ir.model.data', 'search_read', [
                        [['module', '=', id_module], ['name', '=', id_name]]
                    ], {'fields': ['id', 'res_id', 'model'], 'limit': 1})
                except Exception:
                    continue

                if not existing:
                    # XML ID does not exist - check for uniqueness conflicts
                    # For ir.model.fields, check if (model, name) already exists under different XML ID
                    if model == 'ir.model.fields':
                        field_name = self._get_field_value(rec, 'name')
                        field_model = self._get_field_value(rec, 'model')
                        if field_name and field_model:
                            self._check_field_xmlid_conflict(
                                xml_id, field_model, field_name, rel_path)
                            conflicts += 1

        if conflicts == 0:
            log_success("No XML ID conflicts found")

    def _check_field_xmlid_conflict(self, xml_id: str, field_model: str,
                                     field_name: str, rel_path: str):
        """Check if a field already exists under a different XML ID."""
        try:
            # Find the field
            fields = self.api.call('ir.model.fields', 'search_read', [
                [['model', '=', field_model], ['name', '=', field_name]]
            ], {'fields': ['id'], 'limit': 1})

            if not fields:
                return  # Field doesn't exist, no conflict

            field_id = fields[0]['id']

            # Check what XML ID it's registered under
            existing_data = self.api.call('ir.model.data', 'search_read', [
                [['model', '=', 'ir.model.fields'], ['res_id', '=', field_id]]
            ], {'fields': ['module', 'name'], 'limit': 1})

            if existing_data:
                existing_xmlid = f"{existing_data[0]['module']}.{existing_data[0]['name']}"
                if existing_xmlid != xml_id:
                    self._add_issue('error', 'XML_ID_UUID_MISMATCH',
                                    f"Field '{field_model}.{field_name}' exists in DB under "
                                    f"XML ID '{existing_xmlid}' but zip uses '{xml_id}'. "
                                    f"Import will try CREATE instead of UPDATE, causing UniqueViolation.",
                                    file=rel_path,
                                    fix=f"Change XML ID in zip from '{xml_id}' to '{existing_xmlid}'")
        except Exception:
            pass

    def _check_dependencies_installed(self):
        """Check that all manifest dependencies are installed on target."""
        if not self.api:
            return

        depends = self.manifest.get('depends', [])
        if not depends:
            return

        log_info(f"Checking {len(depends)} dependencies...")
        missing = []

        for dep in depends:
            try:
                modules = self.api.call('ir.module.module', 'search_read', [
                    [['name', '=', dep]]
                ], {'fields': ['state'], 'limit': 1})
                if not modules:
                    missing.append((dep, 'not found'))
                elif modules[0]['state'] != 'installed':
                    missing.append((dep, modules[0]['state']))
            except Exception:
                missing.append((dep, 'check failed'))

        for dep, state in missing:
            self._add_issue('error', 'DEPENDENCY_NOT_INSTALLED',
                            f"Dependency '{dep}' is {state} on target instance",
                            fix=f"Install module '{dep}' before importing")

        if not missing:
            log_success(f"All {len(depends)} dependencies installed")

    def _check_referenced_models(self):
        """Check that all referenced models exist on target."""
        if not self.api:
            return

        log_info("Checking referenced models...")
        models_to_check = set()

        for rel_path, records in self.xml_records.items():
            for xml_id, model, rec in records:
                if model == 'ir.model.fields':
                    field_model = self._get_field_value(rec, 'model')
                    relation = self._get_field_value(rec, 'relation')
                    if field_model:
                        models_to_check.add(field_model)
                    if relation:
                        models_to_check.add(relation)
                elif model == 'ir.ui.view':
                    view_model = self._get_field_value(rec, 'model')
                    if view_model:
                        models_to_check.add(view_model)

        missing = []
        for m in sorted(models_to_check):
            if m.startswith('x_'):
                # Custom models may be created by this same zip, skip
                continue
            try:
                if not self.api.model_exists(m):
                    missing.append(m)
            except Exception:
                pass

        for m in missing:
            self._add_issue('error', 'MODEL_NOT_FOUND',
                            f"Referenced model '{m}' does not exist on target instance",
                            fix=f"Install the module that provides model '{m}'")

        if not missing:
            log_success(f"All {len(models_to_check)} referenced models exist")

    def _check_inherit_id_references(self):
        """Check that all inherit_id references in views resolve."""
        if not self.api:
            return

        log_info("Checking inherit_id references...")
        unresolved = []

        for rel_path, records in self.xml_records.items():
            for xml_id, model, rec in records:
                if model != 'ir.ui.view':
                    continue
                inherit_field = rec.find("field[@name='inherit_id']")
                if inherit_field is None:
                    continue
                ref = inherit_field.get('ref', '')
                if not ref:
                    continue

                # Check if referenced view exists
                if '.' in ref:
                    ref_module, ref_name = ref.split('.', 1)
                else:
                    ref_module = self.module_name
                    ref_name = ref

                try:
                    exists = self.api.call('ir.model.data', 'search_count', [
                        [['module', '=', ref_module], ['name', '=', ref_name],
                         ['model', '=', 'ir.ui.view']]
                    ])
                    if not exists:
                        # Check if it's defined within the zip itself
                        zip_ids = set()
                        for rp, recs in self.xml_records.items():
                            for zid, zm, zr in recs:
                                if zm == 'ir.ui.view':
                                    zip_ids.add(zid)
                        if ref not in zip_ids and f"{self.module_name}.{ref}" not in zip_ids:
                            unresolved.append((ref, xml_id, rel_path))
                except Exception:
                    pass

        for ref, view_id, rel_path in unresolved:
            self._add_issue('error', 'INHERIT_ID_NOT_FOUND',
                            f"View '{view_id}' inherits from '{ref}' which does not exist",
                            file=rel_path,
                            fix=f"Remove or fix the inherit_id reference to '{ref}'")

        if not unresolved:
            log_success("All inherit_id references resolve")

    def _check_web_studio_installed(self):
        """Check if web_studio is installed when Studio context is present."""
        if not self.api:
            return

        # Check if any records use studio context
        has_studio_context = False
        for rel_path, records in self.xml_records.items():
            for xml_id, model, rec in records:
                context = rec.get('context', '')
                if 'studio' in context.lower():
                    has_studio_context = True
                    break
            if has_studio_context:
                break

        # Also check if web_studio is in dependencies
        depends = self.manifest.get('depends', [])
        if 'web_studio' in depends or has_studio_context:
            try:
                modules = self.api.call('ir.module.module', 'search_read', [
                    [['name', '=', 'web_studio']]
                ], {'fields': ['state'], 'limit': 1})
                if not modules or modules[0]['state'] != 'installed':
                    self._add_issue('warning', 'WEB_STUDIO_NOT_INSTALLED',
                                    "web_studio is not installed on target but the zip "
                                    "uses Studio context or declares it as dependency. "
                                    "Import may work but some features may not behave correctly.",
                                    fix="Install web_studio on target or remove it from dependencies")
            except Exception:
                pass

    # ==================== Helpers ====================

    @staticmethod
    def _get_field_value(record_element, field_name: str) -> str:
        """Extract a field value from a record XML element."""
        for f in record_element.findall('field'):
            if f.get('name') == field_name:
                # Check for ref attribute
                ref = f.get('ref')
                if ref:
                    return ref
                # Check for eval attribute
                ev = f.get('eval')
                if ev:
                    return ev
                # Text content
                return (f.text or '').strip()
        return ''

    # ==================== Fix Mode ====================

    def create_fixed_zip(self, output_path: str) -> str:
        """Create a fixed copy of the zip, applying auto-fixes for detected issues."""
        tmpdir = tempfile.mkdtemp(prefix='studio_fix_')
        try:
            # Extract
            with zipfile.ZipFile(self.zip_path, 'r') as zf:
                zf.extractall(tmpdir)

            module_dir = os.path.join(tmpdir, self.module_name)
            manifest_path = os.path.join(module_dir, '__manifest__.py')
            fixes_applied = []

            # Fix 1: Remove stray files
            import fnmatch
            for name in self.zip_files:
                if name.endswith('/'):
                    continue
                basename = os.path.basename(name)
                for pattern in STRAY_FILE_PATTERNS:
                    if fnmatch.fnmatch(basename, pattern):
                        full_path = os.path.join(tmpdir, name)
                        if os.path.exists(full_path):
                            os.remove(full_path)
                            fixes_applied.append(f"Removed stray file: {name}")
                        break

            # Fix 2: Remove deprecated fields from view archs
            for rel_path, records in self.xml_records.items():
                full_xml_path = os.path.join(module_dir, rel_path)
                if not os.path.exists(full_xml_path):
                    continue

                with open(full_xml_path, 'r', encoding='utf-8') as f:
                    content = f.read()

                modified = False
                for dep_field in DEPRECATED_FIELDS:
                    # Remove <field name="__last_update" .../> lines
                    pattern = rf'\s*<field\s+name="{re.escape(dep_field)}"[^/]*/>\s*\n?'
                    new_content = re.sub(pattern, '\n', content)
                    if new_content != content:
                        content = new_content
                        modified = True
                        fixes_applied.append(f"Removed deprecated field '{dep_field}' from {rel_path}")

                if modified:
                    with open(full_xml_path, 'w', encoding='utf-8') as f:
                        f.write(content)

            # Fix 3: Remove full arch replacement records
            for rel_path, records in self.xml_records.items():
                full_xml_path = os.path.join(module_dir, rel_path)
                if not os.path.exists(full_xml_path):
                    continue

                # Collect XML IDs to remove
                ids_to_remove = []
                for xml_id, model, rec in records:
                    if model != 'ir.ui.view':
                        continue
                    if '.' in xml_id:
                        module_part = xml_id.split('.')[0]
                        if module_part != self.module_name:
                            inherit_field = rec.find("field[@name='inherit_id']")
                            arch_field = rec.find("field[@name='arch']")
                            if inherit_field is None and arch_field is not None:
                                arch_text = ET.tostring(arch_field, encoding='unicode')
                                if '<xpath' not in arch_text:
                                    ids_to_remove.append(xml_id)

                if not ids_to_remove:
                    continue

                with open(full_xml_path, 'r', encoding='utf-8') as f:
                    content = f.read()

                modified = False
                for xml_id in ids_to_remove:
                    # Match <record id="xml_id" ...>...</record> with regex
                    escaped_id = re.escape(xml_id)
                    pattern = rf'\s*<record\s+id="{escaped_id}"[^>]*>.*?</record>\s*'
                    new_content = re.sub(pattern, '\n', content, count=1, flags=re.DOTALL)
                    if new_content != content:
                        content = new_content
                        modified = True
                        fixes_applied.append(
                            f"Removed full arch replacement '{xml_id}' from {rel_path}")

                if modified:
                    with open(full_xml_path, 'w', encoding='utf-8') as f:
                        f.write(content)

            # Fix 4: Sync manifest data list with actual files
            actual_data_files = set()
            data_dir = os.path.join(module_dir, 'data')
            if os.path.isdir(data_dir):
                for fname in sorted(os.listdir(data_dir)):
                    if fname.endswith(('.xml', '.csv')):
                        actual_data_files.add(f'data/{fname}')

            manifest_files_set = set(self.manifest_data_files)
            if actual_data_files != manifest_files_set:
                # Read and update manifest
                with open(manifest_path, 'r', encoding='utf-8') as f:
                    manifest_content = f.read()

                # Build new data list preserving order of existing entries
                new_data_list = []
                for f in self.manifest_data_files:
                    if f in actual_data_files:
                        new_data_list.append(f)
                # Add files not in manifest
                for f in sorted(actual_data_files):
                    if f not in manifest_files_set:
                        new_data_list.append(f)

                # Replace data list in manifest
                new_data_str = "    'data': [\n"
                for f in new_data_list:
                    new_data_str += f"        '{f}',\n"
                new_data_str += "    ]"

                manifest_content = re.sub(
                    r"'data'\s*:\s*\[.*?\]",
                    new_data_str,
                    manifest_content,
                    flags=re.DOTALL,
                )

                with open(manifest_path, 'w', encoding='utf-8') as f:
                    f.write(manifest_content)

                added = actual_data_files - manifest_files_set
                removed = manifest_files_set - actual_data_files
                if added:
                    fixes_applied.append(f"Added to manifest data: {', '.join(sorted(added))}")
                if removed:
                    fixes_applied.append(f"Removed from manifest data: {', '.join(sorted(removed))}")

            # Create the fixed zip
            os.makedirs(os.path.dirname(os.path.abspath(output_path)), exist_ok=True)
            with zipfile.ZipFile(output_path, 'w', zipfile.ZIP_DEFLATED) as zout:
                for root, dirs, files in os.walk(module_dir):
                    # Skip __pycache__
                    dirs[:] = [d for d in dirs if d != '__pycache__']
                    for fname in files:
                        full = os.path.join(root, fname)
                        arcname = os.path.relpath(full, tmpdir)
                        zout.write(full, arcname)

            return fixes_applied

        finally:
            shutil.rmtree(tmpdir, ignore_errors=True)


# ==================== CLI ====================


def print_results(result: ValidationResult, json_output: bool = False):
    """Print validation results in human-readable or JSON format."""
    if json_output:
        print(json.dumps(result.to_dict(), indent=2))
        return

    print(f"\n{'='*60}", file=sys.stderr)
    print(f"Validation: {result.zip_path}", file=sys.stderr)
    print(f"Module: {result.module_name}", file=sys.stderr)
    print(f"Files: {result.file_count} | Records: {result.record_count}", file=sys.stderr)
    print(f"{'='*60}\n", file=sys.stderr)

    if not result.issues:
        log_success("No issues found - zip is ready for import")
        return

    # Group by severity
    errors = result.errors
    warnings = result.warnings

    if errors:
        print(f"\n{RED}Errors ({len(errors)}):{NC}", file=sys.stderr)
        for issue in errors:
            loc = f" [{issue.file}]" if issue.file else ""
            print(f"  {RED}\u2717{NC} [{issue.code}]{loc}", file=sys.stderr)
            print(f"    {issue.message}", file=sys.stderr)
            if issue.fix:
                print(f"    {BLUE}Fix: {issue.fix}{NC}", file=sys.stderr)

    if warnings:
        print(f"\n{YELLOW}Warnings ({len(warnings)}):{NC}", file=sys.stderr)
        for issue in warnings:
            loc = f" [{issue.file}]" if issue.file else ""
            print(f"  {YELLOW}\u26a0{NC} [{issue.code}]{loc}", file=sys.stderr)
            print(f"    {issue.message}", file=sys.stderr)
            if issue.fix:
                print(f"    {BLUE}Fix: {issue.fix}{NC}", file=sys.stderr)

    print(f"\n{'='*60}", file=sys.stderr)
    if result.valid:
        log_success(f"PASSED with {len(warnings)} warning(s)")
    else:
        log_error(f"FAILED: {len(errors)} error(s), {len(warnings)} warning(s)")


def main():
    parser = argparse.ArgumentParser(
        description='Validate an Odoo Studio customization zip before import'
    )
    parser.add_argument('zip_path', help='Path to the customization zip file')
    parser.add_argument('--online', action='store_true',
                        help='Run online checks against target Odoo instance '
                             '(requires ODOO_URL, ODOO_DB, ODOO_USER, ODOO_KEY)')
    parser.add_argument('--json', action='store_true',
                        help='Output results as JSON')
    parser.add_argument('--fix', action='store_true',
                        help='Create a fixed copy of the zip')
    parser.add_argument('-o', '--output',
                        help='Output path for fixed zip (used with --fix)')

    args = parser.parse_args()

    # Validate
    validator = ZipValidator(args.zip_path, online=args.online)
    result = validator.validate()

    # Print results
    print_results(result, json_output=args.json)

    # Fix mode
    if args.fix:
        if not args.output:
            # Auto-generate output path
            base = os.path.splitext(args.zip_path)[0]
            args.output = f"{base}_fixed.zip"

        log_info(f"Creating fixed zip: {args.output}")
        fixes = validator.create_fixed_zip(args.output)
        if fixes:
            print(f"\n{GREEN}Fixes applied:{NC}", file=sys.stderr)
            for fix in fixes:
                print(f"  {GREEN}\u2713{NC} {fix}", file=sys.stderr)
            log_success(f"Fixed zip created: {args.output}")
        else:
            log_info("No auto-fixes were needed")

    sys.exit(0 if result.valid else 1)


if __name__ == '__main__':
    main()
