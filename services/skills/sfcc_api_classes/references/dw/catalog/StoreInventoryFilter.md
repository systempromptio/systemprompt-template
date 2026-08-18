# dw.catalog.StoreInventoryFilter

## Overview
Represents a store inventory filter for customizing search result filtering by store inventories with semantic URL parameters.

## Description
This class represents a store inventory filter, which can be used at ProductSearchModel.setStoreInventoryFilter(StoreInventoryFilter) to filter the search result by one or more store inventories. Compared to the default parameter 'ilids' (Inventory List IDs), the store inventory filter allows customization of the parameter name and the inventory list ID parameter values for URL generations via all URLRefine and URLRelax methods.

```ts
declare class StoreInventoryFilter  {
	/**
	 * Semantic URL parameter of this StoreInventoryFilter
	 */
	readonly semanticURLParameter: string

	/**
	 * List of StoreInventoryFilterValue instances used by this StoreInventoryFilter
	 */
	readonly storeInventoryFilterValues: List

	/**
	 * Creates a new StoreInventoryFilter instance for the given semantic URL parameter and list of StoreInventoryFilterValue instances.
	 * @param semanticURLParameter - The semantic URL parameter for URL generation instead of 'ilids'
	 * @param storeFilterValues - List of StoreInventoryFilterValue instances with store inventory values and related inventory list IDs
	 */
	constructor(semanticURLParameter: string, storeFilterValues: List)

	/**
	 * Returns the semantic URL parameter of this StoreInventoryFilter.
	 */
	getSemanticURLParameter(): string

	/**
	 * Returns a list of StoreInventoryFilterValue instances used by this StoreInventoryFilter.
	 */
	getStoreInventoryFilterValues(): List
}
```
