# dw.catalog.StoreInventoryFilterValue

## Overview
Represents a store inventory filter value that maps semantic inventory IDs to real inventory list IDs for product search filtering.

## Description
This class provides the mapping between a semantic value (e.g., store1,store2 or Burlington,Boston) and the related real inventory list ID. It can be used with StoreInventoryFilter to filter search results by one or more store inventory list IDs via ProductSearchModel.setStoreInventoryFilter(). Compared to ProductSearchModel.setInventoryListIDs(), the store inventory filter allows customization of the inventory parameter name and inventory list ID values for URL generation.

```ts
declare class StoreInventoryFilterValue  {
  /**
   * The real inventory list ID of this store inventory filter value.
   */
  readonly inventoryListID: string

  /**
   * The semantic inventory ID of this store inventory filter value.
   */
  readonly semanticInventoryID: string

  /**
   * Creates a new StoreInventoryFilterValue instance for the semantic inventory ID and real inventory list ID.
   * @param semanticInventoryListID - The semantic inventory list ID
   * @param inventoryListID - The real inventory list ID to filter the search result on
   * @throws NullArgumentException in case of missing required parameter
   */
  constructor(semanticInventoryListID: string, inventoryListID: string)

  /**
   * Returns the real inventory list ID of this store inventory filter value.
   */
  getInventoryListID(): string

  /**
   * Returns the semantic inventory ID of this store inventory filter value.
   */
  getSemanticInventoryID(): string
}
```
