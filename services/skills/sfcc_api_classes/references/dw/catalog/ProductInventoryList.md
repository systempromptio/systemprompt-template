# dw.catalog.ProductInventoryList

## Overview
Provides access to inventory list metadata and product-specific inventory records. Used by Omnichannel Inventory (OCI) to represent Locations and Location Groups.

## Description
The ProductInventoryList exposes the list ID, description, and default in-stock flag. It also allows lookup of inventory records by `Product` or product ID. When using OCI, each B2C Commerce ProductInventoryList maps to an external Location or Location Group; IDs must match External Reference and be 2–128 characters using letters, digits, hyphens, or underscores.


```ts
declare class ProductInventoryList extends dw.object.ExtensibleObject {
	/** The default in-stock flag of the inventory list. */
	static defaultInStockFlag: boolean

	/** The description of the inventory list. */
	static description: string

	/** The ID of the inventory list. */
	static ID: string

	/** Returns the default in-stock flag of the inventory list. */
	getDefaultInStockFlag(): boolean

	/** Returns the description of the inventory list. */
	getDescription(): string

	/** Returns the ID of the inventory list. */
	getID(): string

	/**
	 * Returns the inventory record for the specified product or null if none.
	 * @param product
	 */
	getRecord(product: dw.catalog.Product): dw.catalog.ProductInventoryRecord

	/**
	 * Returns the inventory record for the specified product ID or null if none.
	 * @param productID
	 */
	getRecord(productID: string): dw.catalog.ProductInventoryRecord
}
```
