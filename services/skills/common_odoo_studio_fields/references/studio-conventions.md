# Studio Conventions Reference

How Odoo Studio works internally and the conventions this skill follows.

## Why Studio Conventions?

### Migration Benefits

Fields created following Studio conventions:
- Are automatically migrated during Odoo version upgrades
- Don't require custom migration scripts
- Are handled by Odoo's upgrade service
- Survive database transfers between instances

### How It Works

1. **Fields in Database**: Stored in `ir.model.fields` table
2. **Views in Database**: Stored as `ir.ui.view` records with inheritance
3. **Metadata Tracking**: Registered in `ir.model.data` with `studio=True`
4. **No Code Files**: Everything lives in the database

## Naming Conventions

### Field Names

| Pattern | Example | Use Case |
|---------|---------|----------|
| `x_studio_{name}` | `x_studio_phone2` | User-named fields |
| `x_studio_{type}_field_{hash}` | `x_studio_char_field_abc123` | Auto-generated names |

This skill always uses the user-friendly pattern: `x_studio_{snake_case_name}`

### View Names

```
Odoo Studio: {original_view_name} customization
```

Example:
```
Odoo Studio: res.partner.form customization
```

### XML IDs (ir.model.data)

```
studio_customization.{type}_{name}_{uuid}
```

Examples:
```
studio_customization.field_x_studio_phone2_a1b2c3d4
studio_customization.view_res_partner_form_e5f6g7h8
```

## Database Structure

### ir.model.fields

Key fields for Studio-created fields:

| Field | Value | Description |
|-------|-------|-------------|
| `name` | `x_studio_*` | Technical name |
| `field_description` | User label | Display name |
| `model` | `res.partner` | Target model |
| `ttype` | `char`, `boolean`, etc. | Field type |
| `state` | `manual` | Indicates dynamic field |
| `store` | `True` | Stored in database |

### ir.ui.view

Key fields for Studio view modifications:

| Field | Value | Description |
|-------|-------|-------------|
| `name` | `Odoo Studio: ...` | View name |
| `model` | `res.partner` | Target model |
| `type` | `form` | View type |
| `mode` | `extension` | Inheriting view |
| `inherit_id` | Parent view ID | Base view |
| `arch_db` | XML content | View definition |
| `priority` | `99` | Load order |

### ir.model.data

Key fields for tracking:

| Field | Value | Description |
|-------|-------|-------------|
| `name` | Unique identifier | Record reference |
| `module` | `studio_customization` | Module name |
| `model` | `ir.model.fields` | Target model |
| `res_id` | Record ID | Target record |
| `studio` | `True` | Studio flag |

## View Inheritance

Studio uses xpath to modify existing views:

```xml
<data>
  <xpath expr="//field[@name='phone']" position="after">
    <field name="x_studio_phone2"/>
  </xpath>
</data>
```

### Position Options

| Position | Effect |
|----------|--------|
| `after` | Insert after matched element |
| `before` | Insert before matched element |
| `inside` | Insert as child of matched element |
| `replace` | Replace matched element |
| `attributes` | Modify attributes of matched element |

## Field States

### state = 'manual'

- Field created via Studio or API
- Not defined in Python code
- Stored in database only
- Survives module updates

### state = 'base'

- Field inherited from another model
- Or field defined in Python code
- Cannot be modified via Studio

## Selection Fields

Selection options are stored in two places:

1. **Field definition** (`ir.model.fields.selection`):
   - Individual option records
   - Include value, name, sequence

2. **Field attribute** (`ir.model.fields.selection`):
   - String representation: `[('value', 'Label'), ...]`
   - Updated when options change

## Export/Import

### Studio Export

Studio creates a module `studio_customization` containing:
- Field definitions (Python)
- View definitions (XML)
- Data records (XML)

### This Skill's Approach

Instead of exporting to files:
- Fields created directly in database
- Views created directly in database
- Metadata tracked in `ir.model.data`

Result: Same database state as using Studio UI.

## Comparison with Custom Modules

| Aspect | Studio/This Skill | Custom Module |
|--------|-------------------|---------------|
| Storage | Database | Files |
| Migration | Automatic | Manual scripts |
| Versioning | Database backup | Git |
| Complexity | Simple fields | Any logic |
| Performance | Same | Same |

## Best Practices

### Do Use For:

- Simple data fields
- Dropdown selections
- Basic relations (Many2one)
- View positioning

### Don't Use For:

- Complex business logic
- Computed fields with dependencies
- Automated workflows
- Security rules
- Reports

### Naming Tips

1. Use descriptive names: "Customer Category" not "Cat"
2. Be consistent across models
3. Prefix related fields: "VIP Status", "VIP Since", "VIP Level"
4. Avoid special characters

## Cleanup

To remove Studio fields:

1. Delete the view inheritance (if any)
2. Delete the field from `ir.model.fields`
3. Clean `ir.model.data` entries

**Warning**: Deleting fields removes all stored data!
