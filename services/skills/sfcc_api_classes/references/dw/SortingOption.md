# dw.catalog.SortingOption

## Overview
Represents an option for sorting product search results in the storefront.

## Description
Provides locale-aware labels and links to an optional `SortingRule`. Used by search UI to present sorting choices.

## All Known Subclasses
None

```ts
declare class SortingOption extends dw.object.PersistentObject {
    /** The description of the sorting option for the current locale. */
    description: string

    /** The display name of the sorting option for the current locale. */
    displayName: string

    /** The ID of the sorting option. */
    ID: string

    /** The associated SortingRule or null if none. */
    sortingRule: dw.catalog.SortingRule | null

    /** Returns the description for the current locale. */
    getDescription(): string

    /** Returns the display name for the current locale. */
    getDisplayName(): string

    /** Returns the sorting option ID. */
    getID(): string

    /** Returns the associated SortingRule or null if none. */
    getSortingRule(): dw.catalog.SortingRule | null
}
```
