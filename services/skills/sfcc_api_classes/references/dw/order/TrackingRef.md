# dw.order.TrackingRef

## Overview
Provides information linking a ShippingOrderItem to a TrackingInfo, including quantity.

## Description
Represents the assignment of a shipping order item to a tracking information record.

```ts
declare class TrackingRef  {
    /**
     * Gets the quantity the shipping order item is assigned to the tracking info.
     */
    quantity: dw.value.Quantity

    /**
     * Gets the shipping order item which is assigned to the tracking info.
     * @readonly
     */
    shippingOrderItem: dw.order.ShippingOrderItem

    /**
     * Gets the tracking info the shipping order item is assigned to.
     * @readonly
     */
    trackingInfo: dw.order.TrackingInfo

    /**
     * Gets the quantity the shipping order item is assigned to the tracking info.
     * @returns {dw.value.Quantity}
     */
    getQuantity(): dw.value.Quantity

    /**
     * Gets the shipping order item which is assigned to the tracking info.
     * @returns {dw.order.ShippingOrderItem}
     */
    getShippingOrderItem(): dw.order.ShippingOrderItem

    /**
     * Gets the tracking info the shipping order item is assigned to.
     * @returns {dw.order.TrackingInfo}
     */
    getTrackingInfo(): dw.order.TrackingInfo

    /**
     * Sets the quantity the shipping order item is assigned to the tracking info.
     * @param {dw.value.Quantity} quantity
     */
    setQuantity(quantity: dw.value.Quantity): void
}
```
