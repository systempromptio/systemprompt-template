
# Odoo Studio Customizations

Create fields, models, menus, and views following Odoo Studio conventions.

## Prerequisites

Authenticate with Odoo first (via odoo-pilot):
```bash
export ODOO_URL="https://your-instance.odoo.com"
export ODOO_DB="your_database"
export ODOO_USER="admin"
export ODOO_KEY="your-api-key-or-password"
```

## Scripts Overview

| Script | Purpose |
|--------|---------|
| `create_model.py` | Create new models |
| `create_field.py` | Create single field |
| `create_fields_batch.py` | Batch field creation |
| `create_menu.py` | Create menus and actions |
| `create_page.py` | Create tabs/pages in forms |
| `add_to_view.py` | Position fields in views |
| `validate_studio.py` | Validate conventions |
| `export_customizations.py` | Export as module |
| `validate_zip.py` | Pre-import zip validator |
| `import_studio_zip.py` | Import Studio zip |

---

## Create Models

```bash
# Create a new model with default fields (x_name, x_active)
./scripts/create_model.py "Vehicle" '{}'

# Create model without mail integration
./scripts/create_model.py "Simple Record" '{"inherit_mail": false}'

# Create transient model (wizard)
./scripts/create_model.py "Import Wizard" '{"transient": true}'
```

**Options:**
- `description`: Model description
- `inherit_mail`: Inherit mail.thread (default: true)
- `inherit_activity`: Inherit mail.activity.mixin (default: true)
- `add_name`: Add x_name field (default: true)
- `add_active`: Add x_active field (default: true)
- `transient`: Create transient model (default: false)

---

## Create Fields

```bash
# Basic field types
./scripts/create_field.py res.partner "Is VIP" boolean '{}'
./scripts/create_field.py res.partner "Notes" text '{}'
./scripts/create_field.py sale.order "Priority Score" integer '{}'

# Selection field
./scripts/create_field.py res.partner "Customer Type" selection '{
    "selection_options": [
        {"value": "individual", "name": "Individual"},
        {"value": "business", "name": "Business"}
    ]
}'

# Relational fields
./scripts/create_field.py res.partner "Account Manager" many2one '{"relation": "res.users"}'
./scripts/create_field.py res.partner "Tags" many2many '{"relation": "res.partner.category"}'

# Monetary field
./scripts/create_field.py sale.order "Custom Price" monetary '{"currency_field": "currency_id"}'
```

**Field Types:** char, text, boolean, integer, float, date, datetime, selection, many2one, many2many, one2many, html, monetary

**Options:**
- `required`: true/false
- `readonly`: true/false
- `help`: Help text
- `relation`: Target model (for relational fields)
- `auto_suffix`: Add _1, _2 if field exists (default: true)

---

## Create Menus

```bash
# Create menu for a model (auto-creates action)
./scripts/create_menu.py x_vehicle "Vehicles" '{}'

# Create menu under a parent
./scripts/create_menu.py crm.lead "My Leads" '{"parent_menu": "crm.crm_menu_root"}'

# List available menus (find parent IDs)
./scripts/create_menu.py list "CRM"
```

**Options:**
- `parent_menu`: Parent menu XML ID or name
- `parent_id`: Parent menu ID
- `sequence`: Menu order (default: 10)
- `view_mode`: View modes (default: "tree,form")
- `domain`: Filter records
- `groups`: List of group XML IDs

---

## Add Fields to Views

```bash
# Position after a field
./scripts/add_to_view.py res.partner form x_studio_is_vip after phone '{}'

# Position inside a Studio group
./scripts/add_to_view.py crm.lead form x_studio_field inside studio_group_abc_left '{}'

# Add with widget
./scripts/add_to_view.py res.partner form x_studio_phone2 after phone '{"widget": "phone"}'
```

**Positions:** after, before, inside

**Note:** When adding multiple fields to the same position, they are grouped in the same xpath (Studio convention).

---

## Create Pages/Tabs

```bash
# Create a new tab with left/right groups
./scripts/create_page.py crm.lead "Extra Info" '{}'

# Create after specific page
./scripts/create_page.py crm.lead "Details" '{"after_page": "internal_notes"}'
```

---

## Batch Field Creation

```bash
# From JSON array
./scripts/create_fields_batch.py crm.lead '[
    {"name": "Field 1", "type": "char"},
    {"name": "Field 2", "type": "boolean"},
    {"name": "Status", "type": "selection", "options": {"selection_options": [{"value": "new", "name": "New"}]}}
]'

# With view positioning
./scripts/create_fields_batch.py crm.lead fields.json --add-to-view studio_group_abc_left
```

---

## Validation & Export

```bash
# Validate Studio conventions
./scripts/validate_studio.py crm.lead
./scripts/validate_studio.py res.partner --json

# Export as installable module
./scripts/export_customizations.py crm.lead --output ./my_module
./scripts/export_customizations.py crm.lead -o ./export -n custom_crm_fields
```

---

## Import Studio Zip

Validate and import Studio `customization.zip` files from another instance.

```bash
# Validate a zip (offline checks: structure, XML, deprecated fields, stray files)
./scripts/validate_zip.py customization.zip

# Validate with online checks (XML ID conflicts, dependencies, models)
./scripts/validate_zip.py customization.zip --online

# Auto-fix common issues and create a fixed zip
./scripts/validate_zip.py customization.zip --fix -o customization_fixed.zip

# Import into target instance
./scripts/import_studio_zip.py customization_fixed.zip

# Force import (overwrite existing)
./scripts/import_studio_zip.py customization.zip --force

# File-by-file import (isolates errors per data file)
./scripts/import_studio_zip.py customization.zip --file-by-file

# Validate only, no import
./scripts/import_studio_zip.py customization.zip --validate-only --json
```

**Pre-import workflow:** Validate -> Fix -> Validate online -> Import

---

## See Also

- [studio-conventions.md](references/studio-conventions.md) - Internal conventions
- [improvements-proposal.md](references/improvements-proposal.md) - Roadmap
- [xpath-patterns.md](references/xpath-patterns.md) - View positioning details
- [import-studio-zip.md](references/import-studio-zip.md) - Import guide
- [import-troubleshooting.md](references/import-troubleshooting.md) - Import error catalog
