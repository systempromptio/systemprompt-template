# dw.order.ShippingOrderItem

## Overview
One or more ShippingOrderItems are contained in a ShippingOrder and reference an OrderItem/LineItem.

## Description
Represents a shipping-specific order item. ShippingOrderItem exposes constants for statuses, read-only properties like `basePrice`, `quantity`, `shippingOrderNumber`, `status`, and `trackingRefs`, and methods to manipulate tracking refs, pricing, status and parent relationships.

```ts
declare class ShippingOrderItem extends AbstractItem {
	/** Constant for Order Item Status CANCELLED */
	static STATUS_CANCELLED: 'CANCELLED'

	/** Constant for Order Item Status CONFIRMED */
	static STATUS_CONFIRMED: 'CONFIRMED'

	/** Constant for Order Item Status SHIPPED */
	static STATUS_SHIPPED: 'SHIPPED'

	/** Constant for Order Item Status WAREHOUSE */
	static STATUS_WAREHOUSE: 'WAREHOUSE'

	/** Price of a single unit before discount application. */
	getBasePrice(): Money

	/** Returns null or the parent item. */
	getParentItem(): ShippingOrderItem

	/** The quantity of the shipping order item. */
	getQuantity(): Quantity

	/** The mandatory shipping order number of the related ShippingOrder. */
	getShippingOrderNumber(): string

	/** Gets the order item status (EnumValue). */
	getStatus(): EnumValue

	/** Gets the tracking refs (TrackingRef collection) assigned to this item. */
	getTrackingRefs(): FilteringCollection

	/**
	 * Assign a tracking info id with a quantity to this shipping order item.
	 * @param trackingInfoID the id of the tracking info
	 * @param quantity the quantity assigned to the tracking info (optional)
	 * @returns the new TrackingRef
	 */
	addTrackingRef(trackingInfoID: string, quantity: Quantity): TrackingRef

	/** Apply a (factor/divisor) rate to prices in this item. */
	applyPriceRate(factor: Decimal, divisor: Decimal, roundUp: boolean): void

	/** Set a parent item (null allowed). */
	setParentItem(parentItem: ShippingOrderItem): void

	/** Set the status for this shipping order item. */
	setStatus(status: string): void

	/** Split this shipping order item by quantity; returns new or same item. */
	split(quantity: Quantity, splitOrderItem?: boolean): ShippingOrderItem
}
```
