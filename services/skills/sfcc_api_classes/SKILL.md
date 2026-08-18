# SFCC Script API & Documentation

## API Class Reference

- **Full index**: [references/dw/index.md](references/dw/index.md) — complete class list
- **Global objects & functions**: [references/global.md](references/global.md)

### Path to Class Docs

`references/dw/<packageName>/<ClassName>.md`

Examples: `dw.catalog.Product` → `references/dw/catalog/Product.md`

Tip: First 15 lines contain Overview and Description for quick info.

B2C Commerce Script version 25.10

## CLI Documentation Search

Quick lookups when the offline corpus above does not have what you need. For the complete `b2c docs` command reference (download, JSON output, all flags), see the `b2c_docs` skill.

```bash
b2c docs search ProductMgr              # Search by class name
b2c docs search "catalog product"       # Partial match
b2c docs read ProductMgr                # Read class docs
b2c docs read dw.catalog.ProductMgr     # Fully qualified name
b2c docs read ProductMgr --raw          # Raw markdown output
b2c docs schema catalog                 # Read XSD schema
b2c docs schema --list                  # List all schemas
```

Common schemas: catalog, order, customer, inventory, pricebook, promotion, coupon, jobs

## Custom Attributes

**ALWAYS run the extraction script first** — don't manually parse XML (files are 10k+ lines).

```bash
node <skill-path>/scripts/getCustomAttributeDefinition.mjs <path-to-system-objecttype-extensions.xml> <ClassName>
```

Output: attribute IDs, display names, line ranges.

- Location: `*/site_template/meta/system-objecttype-extensions.xml` or `*/meta/system-objecttype-extensions.xml`
- Access pattern: `.custom` property on business objects (extend `ExtensibleObject`)

## Custom Objects

**ALWAYS run the extraction script first** — don't infer from code.

```bash
node <skill-path>/scripts/getCustomObjectDefinition.mjs <path-to-custom-objecttype-definitions.xml> <customObjectType>
```

- Location: `*/site_template/meta/custom-objecttype-definitions.xml`
- Access: `dw.object.CustomObjectMgr.getCustomObject(type, key)`

## OCAPI Data API Resources

Full resource reference with paths and methods: [references/ocapi-data-resources.md](references/ocapi-data-resources.md)

Read this reference BEFORE making any OCAPI Data API call to verify the resource exists and get the correct endpoint path.
