# dw.order.ShippingLineItem

## Overview
Represents a specific line item in a shipment and defines shipping cost-related values and adjustments.

## Description
Provides access to adjusted prices/taxes, shipping price adjustments collection and helper methods to create/remove adjustments.

```ts
declare class ShippingLineItem extends LineItem {
    /** Constant used to get the standard shipping line item. */
    static STANDARD_SHIPPING_ID: 'STANDARD_SHIPPING'

    /** The price of this shipping line item including tax after shipping adjustments have been applied. @readonly */
    adjustedGrossPrice: Money

    /** The price of this shipping line item excluding tax after shipping adjustments have been applied. @readonly */
    adjustedNetPrice: Money

    /** The adjusted price of this shipping line item. @readonly */
    adjustedPrice: Money

    /** The tax of this shipping line item after shipping adjustments. @readonly */
    adjustedTax: Money

    /** The ID of this ShippingLineItem. @readonly */
    ID: string

    /** The order-item extension for this item, or null. @readonly */
    orderItem: OrderItem | null

    /** The collection of shipping price adjustments applied to this shipping line item. @readonly */
    shippingPriceAdjustments: Collection

    /** Creates a shipping price adjustment to be applied to the shipping line item. @param promotionID: string */
    createShippingPriceAdjustment(promotionID: string): PriceAdjustment

    /** Creates a shipping price adjustment with explicit discount. @param promotionID: string @param discount: Discount */
    createShippingPriceAdjustment(promotionID: string, discount: Discount): PriceAdjustment

    /** Returns adjusted gross price. */
    getAdjustedGrossPrice(): Money

    /** Returns adjusted net price. */
    getAdjustedNetPrice(): Money

    /** Returns adjusted price. */
    getAdjustedPrice(): Money

    /** Returns adjusted tax. */
    getAdjustedTax(): Money

    /** Returns the ID. */
    getID(): string

    /** Returns the orderItem extension or null. */
    getOrderItem(): OrderItem | null

    /** Returns shipping price adjustments collection. */
    getShippingPriceAdjustments(): Collection

    /** Removes the specified shipping price adjustment. @param priceAdjustment: PriceAdjustment */
    removeShippingPriceAdjustment(priceAdjustment: PriceAdjustment): void
}
```
