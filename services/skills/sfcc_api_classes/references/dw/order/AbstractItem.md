# dw.order.AbstractItem

## Overview
An item based on an OrderItem; provides access to the underlying OrderItem, the extended LineItem, the parent Order, item-level prices, id, and supports custom properties.

## Description
An item which references an <code>OrderItem</code>. Provides methods to access the OrderItem, the extended <code>LineItem</code>, and the <code>Order</code>. Also exposes item-level prices and identifiers. Supports custom properties.

## All Known Subclasses
AppeasementItem, InvoiceItem, ReturnCaseItem, ReturnItem, ShippingOrderItem

```ts
declare class AbstractItem extends Extensible {
	/** Gross price of item. */
	readonly grossPrice: Money

	/** The item-id used for referencing between items. */
	readonly itemID: string

	/** The Order Product- or Shipping- LineItem associated with this item. Should never return null. */
	readonly lineItem: LineItem

	/** Net price of item. */
	readonly netPrice: Money

	/** The order item extensions related to this item. Should never return null. */
	readonly orderItem: OrderItem

	/** The order-item-id used for referencing the OrderItem. */
	readonly orderItemID: string

	/** Total tax for item. */
	readonly tax: Money

	/** Price of entire item on which tax calculation is based. */
	readonly taxBasis: Money

	/** Tax items representing a tax breakdown. */
	readonly taxItems: Collection

	/** Gross price of item. */
	getGrossPrice(): Money

	/** The item-id used for referencing between items. */
	getItemID(): string

	/** Returns the Order Product- or Shipping- LineItem associated with this item. Should never return null. */
	getLineItem(): LineItem

	/** Net price of item. */
	getNetPrice(): Money

	/** Returns the order item extensions related to this item. Should never return null. */
	getOrderItem(): OrderItem

	/** The order-item-id used for referencing the OrderItem. */
	getOrderItemID(): string

	/** Total tax for item. */
	getTax(): Money

	/** Price of entire item on which tax calculation is based. */
	getTaxBasis(): Money

	/** Tax items representing a tax breakdown. */
	getTaxItems(): Collection
}
```
