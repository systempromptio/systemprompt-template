# dw.order.AbstractItemCtnr

## Overview
Base for item-based objects created from a single Order (for example Invoice). Holds a collection of items related to OrderItems and exposes subtotals and grand total.

## Description
Basis for item-based objects stemming from a single <code>Order</code>. Provides access to the originating order, an unsorted collection of items, product and service subtotals, and the grand total as <code>SumItem</code> instances. Supports custom properties.

## All Known Subclasses
Appeasement, Invoice, Return, ReturnCase, ShippingOrder

```ts
declare class AbstractItemCtnr extends Extensible {
	/** Created by this user. */
	readonly createdBy: string

	/** The time of creation. */
	readonly creationDate: Date

	/** The sum-item representing the grand total for all items. */
	readonly grandTotal: SumItem

	/** The unsorted collection of items. */
	readonly items: FilteringCollection

	/** The last modification time. */
	readonly lastModified: Date

	/** Last modified by this user. */
	readonly modifiedBy: string

	/** The Order this object was created for. */
	readonly order: Order

	/** The sum-item representing the subtotal for product items. */
	readonly productSubtotal: SumItem

	/** The sum-item representing the subtotal for service items such as shipping. */
	readonly serviceSubtotal: SumItem

	getCreatedBy(): string
	getCreationDate(): Date
	getGrandTotal(): SumItem
	getItems(): FilteringCollection
	getLastModified(): Date
	getModifiedBy(): string
	getOrder(): Order
	getProductSubtotal(): SumItem
	getServiceSubtotal(): SumItem
}
```
