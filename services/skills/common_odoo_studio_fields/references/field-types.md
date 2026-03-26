# Field Types Reference

Complete reference for Odoo field types and their options.

## Basic Types

### char (Short Text)
```json
{"size": 100}
```

### text (Long Text)
No special options.

### boolean (Checkbox)
No special options.

### integer (Whole Number)
No special options.

### float (Decimal)
```json
{"digits": [16, 2]}
```

### date (Date Only)
No special options.

### datetime (Date and Time)
No special options.

### html (Rich Text)
No special options.

### binary (File/Image)
No special options.

### monetary (Currency)
```json
{"currency_field": "currency_id"}
```

## Selection (Dropdown)

```json
{
    "selection_options": [
        {"value": "draft", "name": "Draft"},
        {"value": "confirmed", "name": "Confirmed"},
        {"value": "done", "name": "Done"}
    ]
}
```

## Relational Types

### many2one (Link to Another Record)
```json
{
    "relation": "res.users",
    "domain": "[('share','=',False)]"
}
```

Common relations:
- `res.users` - Users
- `res.partner` - Contacts
- `res.company` - Companies
- `res.country` - Countries
- `res.currency` - Currencies
- `product.product` - Products
- `account.account` - Accounts

## Type Aliases

These aliases are automatically converted:

| Alias | Odoo Type |
|-------|-----------|
| string, varchar | char |
| number, int | integer |
| decimal, double | float |
| checkbox, bool | boolean |
| select, dropdown | selection |
| relation, link | many2one |

## Common Options (All Types)

```json
{
    "required": true,
    "readonly": false,
    "copied": true,
    "help": "Tooltip help text"
}
```

## Naming Convention

Fields are automatically prefixed with `x_studio_`:

| Input | Output |
|-------|--------|
| "My Field" | `x_studio_my_field` |
| "Cliente VIP" | `x_studio_cliente_vip` |
| "Phone 2" | `x_studio_phone_2` |

## Not Supported (v1.0)

- Computed fields
- Related fields
- One2many / Many2many
- Creating new models
