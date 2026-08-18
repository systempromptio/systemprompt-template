# dw.catalog.ProductInventoryMgr

## Overview
Manager for inventory objects and integration mode. Exposes current inventory integration mode and site-assigned inventory list.

## Description
Provides access to inventory-related objects and constants describing integration modes: `INTEGRATIONMODE_B2C`, `INTEGRATIONMODE_OCI_CACHE`, and `INTEGRATIONMODE_OCI`. Allows retrieval of the inventory list assigned to the current site or lookup by ID.


```ts
declare class ProductInventoryMgr {
	/** Integration mode 'B2C' - using B2C inventory. */
	static INTEGRATIONMODE_B2C: 'B2C'

	/** Integration mode 'OCI' - integration with Omnichannel Inventory. */
	static INTEGRATIONMODE_OCI: 'OCI'

	/** Integration mode 'OCI_CACHE' - initializing cache for OCI integration. */
	static INTEGRATIONMODE_OCI_CACHE: 'OCI_CACHE'

	/** The current inventory integration mode. */
	static inventoryIntegrationMode: string

	/** The inventory list assigned to the current site or null. */
	static inventoryList: dw.catalog.ProductInventoryList

	/** Returns the current inventory integration mode constant string. */
	static getInventoryIntegrationMode(): string

	/** Returns the inventory list assigned to the current site or null. */
	static getInventoryList(): dw.catalog.ProductInventoryList

	/** Returns the inventory list for the given ID or null if not found. */
	static getInventoryList(listID: string): dw.catalog.ProductInventoryList
}
```
