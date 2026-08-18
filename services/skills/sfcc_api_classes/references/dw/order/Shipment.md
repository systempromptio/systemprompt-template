# dw.order.Shipment

## Overview
Represents an order shipment and provides access to shipping line items, costs, status, tracking and helper totals.

## Description
Shipment encapsulates shipping details such as shipping method, tracking references, totals (gross/net/tax), and collections of related line items and adjustments. Use its factory methods to create shipping addresses, shipping line items, and shipping price adjustments; inspect totals and status via the provided getters.

```ts
declare class Shipment extends dw.object.PersistentObject {
	/** Deprecated: Shipment shipping status representing 'Not shipped'. */
	static SHIPMENT_NOTSHIPPED: 0

	/** Deprecated: Shipment shipping status representing 'Shipped'. */
	static SHIPMENT_SHIPPED: 2

	/** Shipment shipping status representing 'Not shipped'. */
	static SHIPPING_STATUS_NOTSHIPPED: 0

	/** Shipment shipping status representing 'Shipped'. */
	static SHIPPING_STATUS_SHIPPED: 2

	/** The adjusted total gross price, including tax (read-only). */
	getAdjustedMerchandizeTotalGrossPrice(): dw.value.Money

	/** The adjusted total net price excluding tax (read-only). */
	getAdjustedMerchandizeTotalNetPrice(): dw.value.Money

	/** The product total price after product discounts (read-only). */
	getAdjustedMerchandizeTotalPrice(): dw.value.Money

	/** The total adjusted product tax (read-only). */
	getAdjustedMerchandizeTotalTax(): dw.value.Money

	/** The adjusted sum of shipping line items including tax (read-only). */
	getAdjustedShippingTotalGrossPrice(): dw.value.Money

	/** The adjusted sum of shipping line items excluding tax (read-only). */
	getAdjustedShippingTotalNetPrice(): dw.value.Money

	/** The adjusted shipping total price (read-only). */
	getAdjustedShippingTotalPrice(): dw.value.Money

	/** The adjusted shipping total tax (read-only). */
	getAdjustedShippingTotalTax(): dw.value.Money

	/** Collection of all line items related to the shipment (read-only). */
	getAllLineItems(): dw.util.Collection

	/** True if this shipment is the default shipment (read-only). */
	isDefault(): boolean

	/** True if this shipment is marked as gift. */
	isGift(): boolean

	/** Collection of gift certificate line items (read-only). */
	getGiftCertificateLineItems(): dw.util.Collection

	/** Gift message value or null. */
	getGiftMessage(): string

	/** ID of this shipment ("me" for default) (read-only). */
	getID(): string

	/** Gross product subtotal in purchase currency (read-only). */
	getMerchandizeTotalGrossPrice(): dw.value.Money

	/** Net product subtotal in purchase currency (read-only). */
	getMerchandizeTotalNetPrice(): dw.value.Money

	/** Product total price (read-only). */
	getMerchandizeTotalPrice(): dw.value.Money

	/** Collection of price adjustments applied to totals (read-only, deprecated). */
	getMerchandizeTotalPriceAdjustments(): dw.util.Collection

	/** Total product tax in purchase currency (read-only). */
	getMerchandizeTotalTax(): dw.value.Money

	/** Collection of product line items (read-only). */
	getProductLineItems(): dw.util.Collection

	/** Total product price including prorated order-level adjustments (read-only). */
	getProratedMerchandizeTotalPrice(): dw.value.Money

	/** Shipment number for this shipment (read-only). */
	getShipmentNo(): string

	/** The shipping address or null if none is set (read-only). */
	getShippingAddress(): dw.order.OrderAddress

	/** Collection of shipping line items (read-only). */
	getShippingLineItems(): dw.util.Collection

	/** The shipping method or null if none is set. */
	getShippingMethod(): dw.order.ShippingMethod

	/** Shipping method ID or null (read-only). */
	getShippingMethodID(): string

	/** Collection of shipping price adjustments (read-only). */
	getShippingPriceAdjustments(): dw.util.Collection

	/** Shipping status EnumValue (read-only). */
	getShippingStatus(): dw.value.EnumValue

	/** Sum of all shipping line items including tax (read-only). */
	getShippingTotalGrossPrice(): dw.value.Money

	/** Sum of all shipping line items excluding tax (read-only). */
	getShippingTotalNetPrice(): dw.value.Money

	/** Shipping total price (read-only). */
	getShippingTotalPrice(): dw.value.Money

	/** Tax of all shipping line items before adjustments (read-only). */
	getShippingTotalTax(): dw.value.Money

	/** Convenience to return the standard shipping line item (read-only). */
	getStandardShippingLineItem(): dw.order.ShippingLineItem

	/** Total gross price of the shipment (read-only). */
	getTotalGrossPrice(): dw.value.Money

	/** Total net price of the shipment (read-only). */
	getTotalNetPrice(): dw.value.Money

	/** Total tax for the shipment (read-only). */
	getTotalTax(): dw.value.Money

	/** Tracking number string. */
	getTrackingNumber(): string

	/** Creates or replaces the shipping address for the shipment. */
	createShippingAddress(): dw.order.OrderAddress

	/** Creates a new shipping line item identified by the specified ID. */
	createShippingLineItem(id: string): dw.order.ShippingLineItem

	/** Creates a shipping price adjustment (deprecated; prefer ShippingLineItem.createShippingPriceAdjustment). */
	createShippingPriceAdjustment(promotionID: string): dw.order.PriceAdjustment

	/** Returns the shipping line item by ID or null if not found. */
	getShippingLineItem(id: string): dw.order.ShippingLineItem | null

	/** Returns the shipping price adjustment associated with the specified promotion ID. */
	getShippingPriceAdjustmentByPromotionID(promotionID: string): dw.order.PriceAdjustment | null

	/** Removes the specified shipping line item and its dependent shipping price adjustments. */
	removeShippingLineItem(shippingLineItem: dw.order.ShippingLineItem): void

	/** Removes the specified shipping price adjustment from the shipment. */
	removeShippingPriceAdjustment(priceAdjustment: dw.order.PriceAdjustment): void

	/** Controls whether this shipment is a gift. */
	setGift(isGift: boolean): void

	/** Sets the gift message value. */
	setGiftMessage(message: string): void

	/** Sets the specified shipping method for the shipment. */
	setShippingMethod(method: dw.order.ShippingMethod): void

	/** Sets the shipping status for the shipment. */
	setShippingStatus(status: number): void

	/** Sets the tracking number of this shipment. */
	setTrackingNumber(aValue: string): void

}
```

