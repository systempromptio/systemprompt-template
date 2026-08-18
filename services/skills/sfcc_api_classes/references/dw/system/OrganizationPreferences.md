# dw.system.OrganizationPreferences

## Overview
Container for custom organization-level (global) attributes accessible across all sites.

## Description
Extensible object for merchant-defined custom attributes with organization-wide scope. Obtain via System.getPreferences(). Has no system attributes—purely for custom data. Access custom values via getCustom(). Handles sensitive security data; observe PCI DSS v3 requirements 2, 4, and 12. Does not expose BM "Global Preferences" (locale, timezone, etc.).

```ts
declare class OrganizationPreferences extends ExtensibleObject {
}
```
