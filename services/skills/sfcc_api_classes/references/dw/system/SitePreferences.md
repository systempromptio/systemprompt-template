# dw.system.SitePreferences

## Overview
Container for custom site-level attributes, accessible via Site.getPreferences().

## Description
SitePreferences is a container for custom site-level attributes. Corresponds with system object type "SitePreferences". Has no system attributes and exists only as a place for merchants to define custom attributes which need to be available for each site.

Logically there is only one SitePreferences instance per site. The instance is obtained by calling Site.getPreferences(). Once an instance is obtained, it is possible to read/write site preference values using the usual syntax for ExtensibleObject instances.

Handles sensitive security-related data. Pay special attention to PCI DSS v3 requirements 2, 4, and 12.

Commerce Cloud Digital defines many site-level preferences (baskets, timezone, locales, customers, etc.) which can be managed in "Site Preferences" module of Business Manager, but these preferences are not accessible through this object (SourceCodeURLParameterName is the one exception).

```ts
declare class SitePreferences extends ExtensibleObject {
  /**
   * Name of source code URL parameter configured for the site.
   */
  readonly sourceCodeURLParameterName: string

  /**
   * Returns name of source code URL parameter configured for the site.
   */
  getSourceCodeURLParameterName(): string
}
```
