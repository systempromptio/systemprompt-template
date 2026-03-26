# Import Studio Customization Zip

Guide for importing Odoo Studio `customization.zip` files into a target instance.

## Zip Structure

A Studio export zip contains a single Odoo module directory:

```
studio_customization/
  __init__.py              # Empty or minimal
  __manifest__.py          # Module metadata, dependencies, data file list
  data/
    ir_model.xml           # Custom model definitions (x_*)
    ir_model_fields.xml    # Custom field definitions (x_studio_*)
    ir_ui_view.xml         # View customizations (extension views with xpath)
    ir_actions_act_window.xml
    ir_actions_act_window_view.xml
    ir_actions_report.xml
    ir_ui_menu.xml
    ir_model_access.xml    # Access control rules
    ir_default.xml         # Default field values
```

**Key points:**
- All records use XML IDs with UUID suffixes (e.g., `new_casilla_de_verif_6582bc9f-...`)
- Fields use `context="{'studio': True}"` to mark Studio origin
- Views are `mode='extension'` with `priority=99`
- The module name is typically `studio_customization`

## Import Methods

### 1. Automated Script (Recommended)

```bash
# Set connection
export ODOO_URL="http://localhost:8069"
export ODOO_DB="mydb"
export ODOO_USER="admin"
export ODOO_KEY="admin-password-or-api-key"

# Validate first
python3 scripts/validate_zip.py customization.zip --online

# Fix issues automatically
python3 scripts/validate_zip.py customization.zip --fix -o customization_fixed.zip

# Import
python3 scripts/import_studio_zip.py customization_fixed.zip

# Import with force (overwrite existing)
python3 scripts/import_studio_zip.py customization_fixed.zip --force

# File-by-file (isolate errors)
python3 scripts/import_studio_zip.py customization.zip --file-by-file
```

### 2. Odoo UI

1. Go to **Settings > Technical > Base Import Module**
2. Upload the zip file
3. Check/uncheck "Force init" as needed
4. Click **Import Module**

### 3. Direct API (JSON-RPC)

```python
import base64, requests

# Authenticate
session = requests.post(f'{url}/web/session/authenticate', json={
    'jsonrpc': '2.0', 'method': 'call',
    'params': {'db': db, 'login': user, 'password': password},
    'id': 1
})
sid = session.cookies.get('session_id')

# Read and encode zip
with open('customization.zip', 'rb') as f:
    zip_b64 = base64.b64encode(f.read()).decode()

# Create wizard
resp = requests.post(f'{url}/web/dataset/call_kw', json={
    'jsonrpc': '2.0', 'method': 'call',
    'params': {
        'model': 'base.import.module',
        'method': 'create',
        'args': [{'module_file': zip_b64, 'state': 'init', 'force': False}],
        'kwargs': {}
    }, 'id': 2
}, cookies={'session_id': sid})
wizard_id = resp.json()['result']

# Execute import
requests.post(f'{url}/web/dataset/call_kw', json={
    'jsonrpc': '2.0', 'method': 'call',
    'params': {
        'model': 'base.import.module',
        'method': 'import_module',
        'args': [[wizard_id]],
        'kwargs': {}
    }, 'id': 3
}, cookies={'session_id': sid}, timeout=300)
```

## Force Mode

| Mode | `force=False` (default) | `force=True` |
|------|------------------------|--------------|
| Existing records | **Update** if XML ID matches | **Overwrite** always |
| New records | Create | Create |
| Missing XML ID match | **Error** (UniqueViolation) | Force-create |
| Use case | Re-importing same customizations | Fresh import or after cleanup |

**Recommendation:** Always try `force=False` first. Use `force=True` only when you've verified the zip content is correct and you want to overwrite.

## Pre-Validation Workflow

```
1. validate_zip.py customization.zip           # Offline checks
   |
   +--> Errors? ---> validate_zip.py --fix     # Auto-fix what's possible
   |                  |
   |                  +--> Still errors? -----> Manual fix (see troubleshooting)
   |
2. validate_zip.py customization.zip --online  # Check against target DB
   |
   +--> XML ID conflicts? ---> Fix UUIDs manually
   +--> Missing deps? -------> Install modules
   |
3. import_studio_zip.py customization.zip      # Import
```

## Post-Import Verification

After importing, verify the results:

```bash
# JSON output for programmatic checks
python3 scripts/import_studio_zip.py customization.zip --json

# Or verify manually via API
python3 scripts/odoo_api.py call ir.model.data search_count \
    --args '[[["module","=","studio_customization"]]]'
```

**Checklist:**
- [ ] Module state is `installed` (not `uninstalled` or missing)
- [ ] `ir.model.data` record count matches expected
- [ ] Custom models exist (`x_*` in `ir.model`)
- [ ] Studio fields present on target models
- [ ] Views render without errors in the browser
- [ ] Menu entries appear in the UI

## Common Patterns

### Fresh Instance (no prior Studio customizations)

```bash
# Simple: validate and import
python3 scripts/validate_zip.py customization.zip --online
python3 scripts/import_studio_zip.py customization.zip
```

### Existing Instance (Studio data already present)

```bash
# Validate online to detect UUID mismatches
python3 scripts/validate_zip.py customization.zip --online

# If XML ID conflicts exist, they must be fixed manually
# (match zip XML IDs to existing ir.model.data entries)

# Import with force if needed
python3 scripts/import_studio_zip.py customization.zip --force
```

### Cross-Version Migration (e.g., Odoo 16 -> 18)

```bash
# Full validation + auto-fix (handles deprecated fields, stray files)
python3 scripts/validate_zip.py customization.zip --fix -o customization_fixed.zip

# Validate the fixed zip online
python3 scripts/validate_zip.py customization_fixed.zip --online

# Import
python3 scripts/import_studio_zip.py customization_fixed.zip
```

### Debugging Failures

```bash
# File-by-file to isolate the failing XML
python3 scripts/import_studio_zip.py customization.zip --file-by-file --json

# Check Odoo server logs for the real error
# (transaction errors mask the actual cause)
```

## See Also

- [import-troubleshooting.md](import-troubleshooting.md) - Error catalog with 10 documented issues
- [studio-conventions.md](studio-conventions.md) - XML ID, field naming, view conventions

---

*Document created: 2026-02-10*
