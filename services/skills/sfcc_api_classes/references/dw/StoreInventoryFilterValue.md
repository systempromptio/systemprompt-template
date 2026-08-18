# dw.catalog.StoreInventoryFilterValue

## Overview
Maps a semantic inventory identifier to a real inventory list ID for use in store inventory filtering.

## Description
Encapsulates the pair (semanticInventoryID, inventoryListID). Used as entries in `StoreInventoryFilter` objects.

## All Known Subclasses
None

```ts
declare class StoreInventoryFilterValue  {
    /** The real inventory list ID used for filtering. */
    inventoryListID: string

    /** The semantic inventory ID (user-facing). */
    semanticInventoryID: string

    /** Creates a new StoreInventoryFilterValue(semanticInventoryID, inventoryListID). */
    constructor(semanticInventoryID: string, inventoryListID: string)

    /** Returns the inventory list ID. */
    getInventoryListID(): string

    /** Returns the semantic inventory ID. */
    getSemanticInventoryID(): string
}
```
