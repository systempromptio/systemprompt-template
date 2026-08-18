# dw.catalog.StoreInventoryFilter

## Overview
Represents a semantic filter that maps URL parameter values to inventory list IDs for product search.

## Description
Used with `ProductSearchModel` to generate user-friendly URL parameters (e.g., city) while filtering by mapped inventory list IDs. Constructed with a semantic parameter name and a list of `StoreInventoryFilterValue` entries.

## All Known Subclasses
None

```ts
declare class StoreInventoryFilter  {
    /** The semantic URL parameter used for this filter (e.g., "city"). */
    semanticURLParameter: string

    /** List of StoreInventoryFilterValue instances used by this filter. */
    storeInventoryFilterValues: dw.util.List

    /** Creates a new StoreInventoryFilter. */
    constructor(semanticURLParameter: string, storeFilterValues: dw.util.List)

    /** Returns the semantic URL parameter. */
    getSemanticURLParameter(): string

    /** Returns the list of StoreInventoryFilterValue instances. */
    getStoreInventoryFilterValues(): dw.util.List
}
```
