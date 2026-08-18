# dw.order.ShipmentShippingCost

## Overview
Represents the shipping cost associated with a shipment, including amounts and price adjustments.

## Description
Holds cost breakdowns for a shipment. Used to read pricing details and applied adjustments.

```ts
declare class ShipmentShippingCost {
    /**
     * The total shipping cost amount for the shipment.
     */
    total: number

    /**
     * Returns an array of `PriceAdjustment` objects applied to this shipping cost.
     */
    getPriceAdjustments(): Array<any>

    /**
     * Returns the currency code for the shipping cost.
     */
    getCurrencyCode(): string
}
```
