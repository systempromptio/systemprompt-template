# XPath Patterns for View Modifications

Reference for positioning fields in Odoo views using xpath.

## Basic Syntax

```bash
./scripts/add_to_view.sh <model> <view_type> <field_name> <position> <reference> [attrs]
```

## Position Values

| Position | Description | Use Case |
|----------|-------------|----------|
| `after` | Insert after the reference element | Most common, add field after another |
| `before` | Insert before the reference element | Add field before another |
| `inside` | Insert inside the reference element | Add to a group or container |

## Reference Patterns

### By Field Name

Most common - reference another field:

```bash
# After phone field
./scripts/add_to_view.sh res.partner form x_studio_mobile2 after phone '{}'

# Before email field
./scripts/add_to_view.sh res.partner form x_studio_alt_email before email '{}'
```

### By Container

Reference structural elements:

```bash
# Inside the sheet (main form area)
./scripts/add_to_view.sh res.partner form x_studio_field inside sheet '{}'

# Inside form root
./scripts/add_to_view.sh res.partner form x_studio_field inside form '{}'
```

## Common Field References by Model

### res.partner (Contacts)

| Field | Description |
|-------|-------------|
| `name` | Contact name |
| `is_company` | Is a Company checkbox |
| `parent_id` | Parent company |
| `street` | Street address |
| `city` | City |
| `country_id` | Country |
| `phone` | Phone |
| `mobile` | Mobile |
| `email` | Email |
| `website` | Website |
| `vat` | Tax ID |
| `user_id` | Salesperson |

### sale.order (Sales Orders)

| Field | Description |
|-------|-------------|
| `name` | Order reference |
| `partner_id` | Customer |
| `date_order` | Order date |
| `validity_date` | Expiration |
| `payment_term_id` | Payment terms |
| `user_id` | Salesperson |
| `team_id` | Sales team |
| `order_line` | Order lines |
| `note` | Terms and conditions |

### purchase.order (Purchase Orders)

| Field | Description |
|-------|-------------|
| `name` | Order reference |
| `partner_id` | Vendor |
| `date_order` | Order date |
| `date_planned` | Receipt date |
| `user_id` | Buyer |
| `order_line` | Order lines |

### product.template (Products)

| Field | Description |
|-------|-------------|
| `name` | Product name |
| `default_code` | Internal reference |
| `barcode` | Barcode |
| `categ_id` | Category |
| `list_price` | Sales price |
| `standard_price` | Cost |
| `type` | Product type |
| `description` | Description |

### account.move (Invoices)

| Field | Description |
|-------|-------------|
| `name` | Number |
| `partner_id` | Customer/Vendor |
| `invoice_date` | Invoice date |
| `invoice_date_due` | Due date |
| `payment_reference` | Payment reference |
| `invoice_line_ids` | Invoice lines |

## Field Attributes

### Display Attributes

```json
{
    "string": "Custom Label",
    "placeholder": "Enter value..."
}
```

### Visibility Conditions

```json
{
    "invisible": "field_name == 'value'"
}
```

**Common patterns:**

```json
// Hide if not a company
{"invisible": "not is_company"}

// Hide if state is done
{"invisible": "state == 'done'"}

// Hide if field is empty
{"invisible": "not partner_id"}

// Complex condition
{"invisible": "state != 'draft' or not is_company"}
```

### Readonly Conditions

```json
{
    "readonly": "state != 'draft'"
}
```

### Required Conditions

```json
{
    "required": "is_company"
}
```

### Widget Selection

Common widgets for different field types:

| Widget | Field Type | Description |
|--------|------------|-------------|
| `phone` | char | Phone number with call link |
| `email` | char | Email with mailto link |
| `url` | char | Clickable URL |
| `image` | binary | Image display |
| `boolean_toggle` | boolean | Toggle switch |
| `radio` | selection | Radio buttons |
| `selection_badge` | selection | Badge style |
| `monetary` | float | Currency display |
| `percentage` | float | Percentage display |
| `many2one_avatar` | many2one | With avatar |

**Example:**

```bash
./scripts/add_to_view.sh res.partner form x_studio_website2 after website '{
    "widget": "url",
    "placeholder": "https://..."
}'
```

## Complete Examples

### Add phone field after mobile

```bash
./scripts/add_to_view.sh res.partner form x_studio_phone_work after mobile '{
    "string": "Work Phone",
    "widget": "phone"
}'
```

### Add VIP checkbox visible only for companies

```bash
./scripts/add_to_view.sh res.partner form x_studio_is_vip after is_company '{
    "invisible": "not is_company",
    "widget": "boolean_toggle"
}'
```

### Add selection to sales order

```bash
./scripts/add_to_view.sh sale.order form x_studio_priority after partner_id '{
    "string": "Priority",
    "widget": "selection_badge"
}'
```

### Add field to tree view

```bash
./scripts/add_to_view.sh res.partner tree x_studio_customer_type after name '{}'
```

## Important: XPath Selection in Form Views

For form views, the script automatically uses `//group/field[@name='...']` instead of just `//field[@name='...']`. This ensures:

1. **Proper label rendering** - Labels appear in bold, aligned with other labels
2. **Correct row positioning** - Field appears in its own row, not inline
3. **Standard group layout** - Uses Odoo's 2-column layout (label | field)

### Fields in Special Groups

Some fields are in special groups with custom layouts (like the address group in res.partner). The script now **auto-detects** these fields and:
1. Uses `//field[@name='...']` instead of `//group/field[@name='...']`
2. Adds explicit `<label>` element for proper label display
3. Shows a warning about potential layout issues

**Known special fields (auto-detected):**
- `res.partner`: `vat`, `street`, `street2`, `city`, `zip`, `state_id`, `country_id`

**Recommended reference fields for res.partner:**
- `phone`, `mobile`, `email` - Standard group layout (best results)
- `function` - First field in contact info group
- `website`, `lang` - Standard fields

## Troubleshooting

### "Could not find primary view"

The model might not have a standard view of that type. Check:
1. Is the app installed?
2. Use the correct view type (`form`, not `formview`)

### Field not appearing

1. Check the reference field exists in the view
2. Verify invisible condition syntax
3. Clear browser cache and reload

### Field appears without label

1. The `string` attribute should be set automatically from field_description
2. If missing, provide it explicitly in attrs: `{"string": "My Label"}`
3. Ensure the field was created with a proper `field_description`

### Field appears inline (not in its own row)

This happens when positioning relative to fields in special layout groups:
1. Choose a different reference field in a standard group
2. See "Fields in Special Groups" section above

### Wrong position

If field appears in wrong place:
1. Check the exact field name (case-sensitive)
2. Use browser dev tools to inspect the view structure
3. Try a different reference field
4. For form views, verify the reference field is inside a `<group>` element
