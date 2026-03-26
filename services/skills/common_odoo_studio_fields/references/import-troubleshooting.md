# Import Studio Zip - Troubleshooting Guide

Error catalog for Odoo Studio `customization.zip` imports. Each entry documents the symptom, root cause, fix, and whether `validate_zip.py` detects it.

---

## 1. `__last_update` Deprecated Field

**Detected by `validate_zip.py`:** Yes (`DEPRECATED_FIELD`) | **Auto-fixed by `--fix`:** Yes

**Symptom:**
```
ValueError: Invalid field '__last_update' on model 'account.analytic.line'
```

**Root cause:** `__last_update` was a magic computed field in Odoo <= 17 that returned the record's last write date. It was removed in Odoo 18. Studio exports from Odoo 16/17 may include it in view definitions.

**Fix:** Remove `<field name="__last_update" .../>` from all view arch XML:
```xml
<!-- BEFORE -->
<xpath expr="//field[@name='date']" position="after">
    <field name="__last_update" optional="show"/>
    <field name="commercial_partner_id" optional="show"/>
</xpath>

<!-- AFTER -->
<xpath expr="//field[@name='date']" position="after">
    <field name="commercial_partner_id" optional="show"/>
</xpath>
```

---

## 2. Stray / Garbage Files

**Detected by `validate_zip.py`:** Yes (`STRAY_FILE`) | **Auto-fixed by `--fix`:** Yes

**Symptom:** Import succeeds but leaves orphan data, or fails with XML parse errors on non-XML files.

**Root cause:** Odoo Studio sometimes creates stray files in the export zip. Common offenders:
- `data/Untitled` - contains a single XML ID reference as plain text
- `.pyc` files from cached Python bytecode
- `.DS_Store` from macOS

**Example stray file content (`data/Untitled`):**
```
studio_customization.new_seleccion_lead_o_8f8d474d-5c7f-4668-a814-6b53569d3821
```

**Fix:** Delete the file and remove any reference from the manifest `data` list.

---

## 3. Missing Manifest Data Entry

**Detected by `validate_zip.py`:** Yes (`MANIFEST_MISSING_ENTRY`) | **Auto-fixed by `--fix`:** Yes

**Symptom:** A data XML file exists in the zip but its records are silently ignored during import.

**Root cause:** Odoo Studio sometimes creates additional XML files (e.g., `ir_ui_view_2.xml`) without adding them to `__manifest__.py`'s `data` list. Odoo only processes files listed in the manifest.

**Fix:** Add the missing file path to the manifest:
```python
'data': [
    'data/ir_model.xml',
    # ... existing entries ...
    'data/ir_ui_view_2.xml',  # <-- add missing file
],
```

---

## 4. Duplicate XML ID (UUID Mismatch)

**Detected by `validate_zip.py`:** Yes (`XML_ID_UUID_MISMATCH`, online mode) | **Auto-fixed by `--fix`:** No (requires DB lookup)

**Symptom:**
```
psycopg2.errors.UniqueViolation: llave duplicada viola restriccion de unicidad
«ir_model_fields_name_unique»
DETALLE: Ya existe la llave (model, name)=(res.partner, x_studio_comunicaciones).
```

**Root cause:** The field already exists in the target database under a **different** XML ID (UUID). For example:
- Zip contains: `studio_customization.new_casilla_de_verif_5494d499-...`
- Database has: `studio_customization.new_casilla_de_verif_6582bc9f-...`

Since the incoming XML ID does not match, Odoo treats it as a CREATE (not UPDATE), which fails on the unique constraint `(model, name)`.

This happens when Studio customizations were exported from a different instance or after Studio regenerated UUIDs.

**Fix:** Query the existing XML ID from `ir.model.data` and replace it in the zip:
```sql
SELECT name FROM ir_model_data
WHERE module = 'studio_customization'
  AND model = 'ir.model.fields'
  AND res_id = (SELECT id FROM ir_model_fields
                WHERE model = 'res.partner' AND name = 'x_studio_comunicaciones');
```
Then update the XML file's `record id=` to match the database value.

---

## 5. Full View Arch Replacement

**Detected by `validate_zip.py`:** Yes (`FULL_ARCH_REPLACEMENT`) | **Auto-fixed by `--fix`:** Yes

**Symptom:**
```
ValueError: 'contact_whatsapp' is not a valid action on model 'res.partner'
```
Or broken layout after import because all inherited views that extend the original view now conflict with the replaced arch.

**Root cause:** The zip contains a record like `<record id="base.view_partner_form" model="ir.ui.view">` with a complete `<form>` arch (no `<xpath>` expressions), which fully replaces the original view. Any modules with views that inherit from the original (e.g., a WhatsApp button extension) will break because the elements they reference no longer exist.

**Fix:** Remove the full arch replacement record entirely. Studio extension views (which use `inherit_id` and xpath) are the correct way to modify views.

---

## 6. Transaction Error Masking (InFailedSqlTransaction)

**Detected by `validate_zip.py`:** No (runtime only) | **Auto-fixed:** No

**Symptom:**
```
psycopg2.errors.InFailedSqlTransaction: transaccion abortada, las ordenes
seran ignoradas hasta el fin de bloque de transaccion
```

**Root cause:** A previous error within the same database transaction (e.g., UniqueViolation from issue #4) caused PostgreSQL to mark the transaction as failed. All subsequent SQL operations in that transaction fail with this generic message, hiding the real error.

**Fix:**
1. Check the Odoo server logs for the **first** error in the traceback chain
2. Fix the underlying issue (usually a UniqueViolation or ValueError)
3. Use `import_studio_zip.py --file-by-file` to isolate which file causes the error

---

## 7. Missing Dependencies

**Detected by `validate_zip.py`:** Yes (`DEPENDENCY_NOT_INSTALLED`, online mode) | **Auto-fixed:** No

**Symptom:**
```
ModuleNotFoundError: No module named 'odoo.addons.account_budget'
```
Or the import simply fails to find required models/fields.

**Root cause:** The `__manifest__.py` declares dependencies that are not installed on the target instance (e.g., `account_budget`, `marketing_automation`, custom modules like `foodles_contact_update`).

**Fix:** Install missing dependencies before importing:
```python
# Check dependencies programmatically
for dep in manifest['depends']:
    module = env['ir.module.module'].search([('name', '=', dep)])
    if not module or module.state != 'installed':
        print(f"MISSING: {dep}")
```

---

## 8. inherit_id References Non-Existent View

**Detected by `validate_zip.py`:** Yes (`INHERIT_ID_NOT_FOUND`, online mode) | **Auto-fixed:** No

**Symptom:**
```
ValueError: External ID not found in the system: module.view_xml_id
```

**Root cause:** A Studio extension view references a parent view via `inherit_id ref="..."` that does not exist on the target. This can happen when:
- The module providing the parent view is not installed
- The view was renamed or removed in a version upgrade
- The view exists in a different database but not this one

**Fix:** Remove the extension view record or update the `inherit_id` to reference an existing view.

---

## 9. Model Does Not Exist

**Detected by `validate_zip.py`:** Yes (`MODEL_NOT_FOUND`, online mode) | **Auto-fixed:** No

**Symptom:**
```
KeyError: 'x_custom_model'
```
Or fields fail to create because the target model doesn't exist.

**Root cause:** The zip references a model that was either:
- Defined by a module not installed on target
- A custom model (`x_*`) that exists in the source instance but not the target
- Removed between Odoo versions

**Fix:** Ensure all required modules are installed. For custom models (`x_*`), check that `ir_model.xml` is processed before `ir_model_fields.xml` (the manifest `data` order matters).

---

## 10. web_studio Not Installed

**Detected by `validate_zip.py`:** Yes (`WEB_STUDIO_NOT_INSTALLED`, online mode) | **Auto-fixed:** No

**Symptom:** Import succeeds but Studio-specific features don't work (no Studio editor, `studio=True` context ignored, approval flows missing).

**Root cause:** `web_studio` is an Enterprise module. The zip was created on an Enterprise instance with Studio, but the target may be Community or hasn't installed `web_studio`.

**Fix:**
- **Enterprise target:** Install `web_studio` module
- **Community target:** Remove `web_studio` from the manifest `depends` list. The import will work for basic fields/views, but Studio-specific features (approval flows, PDF reports customization) won't be available.

---

## Quick Reference

| # | Issue | Code | Mode | Auto-fix |
|---|-------|------|------|----------|
| 1 | `__last_update` deprecated | `DEPRECATED_FIELD` | Offline | Yes |
| 2 | Stray files | `STRAY_FILE` | Offline | Yes |
| 3 | Missing manifest entry | `MANIFEST_MISSING_ENTRY` | Offline | Yes |
| 4 | XML ID UUID mismatch | `XML_ID_UUID_MISMATCH` | Online | No |
| 5 | Full arch replacement | `FULL_ARCH_REPLACEMENT` | Offline | Yes |
| 6 | Transaction masking | *(runtime only)* | - | No |
| 7 | Missing dependencies | `DEPENDENCY_NOT_INSTALLED` | Online | No |
| 8 | inherit_id not found | `INHERIT_ID_NOT_FOUND` | Online | No |
| 9 | Model not found | `MODEL_NOT_FOUND` | Online | No |
| 10 | web_studio missing | `WEB_STUDIO_NOT_INSTALLED` | Online | No |

---

*Document created: 2026-02-10*
