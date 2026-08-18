# Metadata XML Authoring

This skill covers **writing** B2C Commerce metadata XML. To **deploy** it (`b2c job import/export`, archive directory structure, job status), use the `b2c_site_import_export` skill.

## System Object Extensions

Add custom attributes to existing system objects (Product, Order, Profile, Basket, SitePreferences, Category, Content).

```xml
<?xml version="1.0" encoding="UTF-8"?>
<metadata xmlns="http://www.demandware.com/xml/impex/metadata/2006-10-31">
    <type-extension type-id="Product">
        <custom-attribute-definitions>
            <attribute-definition attribute-id="vendorSKU">
                <display-name xml:lang="x-default">Vendor SKU</display-name>
                <type>string</type>
                <mandatory-flag>false</mandatory-flag>
                <externally-managed-flag>true</externally-managed-flag>
            </attribute-definition>
        </custom-attribute-definitions>
        <group-definitions>
            <attribute-group group-id="CustomAttributes">
                <display-name xml:lang="x-default">Custom Attributes</display-name>
                <attribute attribute-id="vendorSKU"/>
            </attribute-group>
        </group-definitions>
    </type-extension>
</metadata>
```

### Attribute Types

| Type | Description |
|------|-------------|
| `string` | Text (max 4000 chars) |
| `text` | Long text (unlimited) |
| `int` / `double` | Integer / Decimal |
| `boolean` | true/false |
| `date` / `datetime` | Date / Date+time |
| `email` / `password` | Email / Encrypted |
| `html` | HTML content |
| `enum-of-string` / `enum-of-int` | Single select |
| `set-of-string` / `set-of-int` | Multi-select |
| `image` | Image reference |

### Enum Values

```xml
<attribute-definition attribute-id="warrantyType">
    <display-name xml:lang="x-default">Warranty Type</display-name>
    <type>enum-of-string</type>
    <value-definitions>
        <value-definition>
            <value>none</value>
            <display xml:lang="x-default">No Warranty</display>
        </value-definition>
        <value-definition>
            <value>full</value>
            <display xml:lang="x-default">Full Warranty</display>
        </value-definition>
    </value-definitions>
</attribute-definition>
```

### Attribute Flags

| Flag | Purpose |
|------|---------|
| `localizable-flag` | Different values per locale |
| `mandatory-flag` | Required in BM |
| `externally-managed-flag` | Read-only in BM |
| `visible-flag` | Shown in BM |
| `site-specific-flag` | Different value per site |
| `order-required-flag` | Required for order export |
| `searchable-flag` | Indexed for search |

## Custom Object Definitions

```xml
<?xml version="1.0" encoding="UTF-8"?>
<metadata xmlns="http://www.demandware.com/xml/impex/metadata/2006-10-31">
    <custom-type type-id="APIConfiguration">
        <display-name xml:lang="x-default">API Configuration</display-name>
        <staging-mode>source-to-target</staging-mode>
        <storage-scope>site</storage-scope>
        <key-definition attribute-id="configId">
            <display-name xml:lang="x-default">Config ID</display-name>
            <type>string</type>
            <min-length>1</min-length>
        </key-definition>
        <attribute-definitions>
            <attribute-definition attribute-id="endpoint">
                <display-name xml:lang="x-default">API Endpoint</display-name>
                <type>string</type>
            </attribute-definition>
        </attribute-definitions>
    </custom-type>
</metadata>
```

### Custom Object Data

```xml
<?xml version="1.0" encoding="UTF-8"?>
<custom-objects xmlns="http://www.demandware.com/xml/impex/customobject/2006-10-31">
    <custom-object type-id="APIConfiguration" object-id="payment-gateway">
        <object-attribute attribute-id="endpoint">https://api.payment.com/v2</object-attribute>
    </custom-object>
</custom-objects>
```

## Site Preferences

### Metadata (meta/system-objecttype-extensions.xml)

Use `type-extension type-id="SitePreferences"` with custom-attribute-definitions (same as above).

### Values (sites/{SiteID}/preferences.xml)

```xml
<?xml version="1.0" encoding="UTF-8"?>
<preferences xmlns="http://www.demandware.com/xml/impex/preferences/2007-03-31">
    <custom-preferences>
        <all-instances>
            <preference preference-id="maxItemsPerPage">25</preference>
        </all-instances>
        <development>
            <preference preference-id="enableFeatureX">true</preference>
        </development>
        <production>
            <preference preference-id="enableFeatureX">false</preference>
        </production>
    </custom-preferences>
</preferences>
```

### Access in Code

```javascript
var pref = require('dw/system/Site').current.getCustomPreferenceValue('enableFeatureX');
```

## Troubleshooting

### Import reports OK but data is missing

The `sfcc-site-archive-import` job (triggered via OCAPI) **respects the OCAPI Data API permissions of the triggering client**. If the client lacks permission on a resource (e.g. `/sites/**`), the job silently skips that data — no error, status still OK, log says "Processed N elements successfully."

**Diagnosis steps:**
1. Check BM > Jobs history — note whether job was triggered "by OCAPI Client" or "by user@email"
2. Check the OCAPI client's Data API permissions (BM > Administration > Site Development > OCAPI Settings > Data API)
3. Compare with required resources — see the `dev_rules` skill → "CI Deployment: OCAPI Client Permissions"

**Common symptom:** metadata (custom attributes, custom objects) imports fine, but site preferences don't. This happens when the client has `/jobs/*/executions` but lacks `/sites/**`.

## Best Practices

1. Test imports on sandbox first
2. Use `--wait --show-log` for debugging
3. Prefix custom attributes with org name (e.g., `acme_myAttribute`)
4. Set `externally-managed-flag` for data from external systems
5. Use enums over strings for controlled vocabularies

## References

- [System Objects Reference](references/SYSTEM-OBJECTS.md)
- [XML Examples](references/XML-EXAMPLES.md)
- [Metadata XML Patterns](references/METADATA-XML.md)

## Related Skills

- `b2c_site_import_export` - Deploy this XML: `b2c job import/export`, archive directory layout, job status
- `b2c_custom_objects` - Reading/writing custom object instances at runtime
- `b2c_querying_data` - Querying custom attributes and objects in scripts
