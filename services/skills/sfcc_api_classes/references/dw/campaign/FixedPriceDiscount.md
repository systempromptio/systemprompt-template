# dw.campaign.FixedPriceDiscount

## Overview
Represents a fixed-price discount in a discount plan (for example, "Shipping only $0.99").

## Description
Represents a fix price discount in the discount plan, for example "Shipping only 0.99 all orders $25 or more." Use to create simple fixed-amount price adjustments (for example, fixed shipping price).

```ts
declare class FixedPriceDiscount extends dw.campaign.Discount {
    /**
     * The fixed price amount (read-only), for example 0.99 for a "Shipping only $0.99" discount.
     */
    readonly fixedPrice: number

    /**
     * Create a fixed-price discount instance. Can be used to create a custom price adjustment.
     * @param amount - fixed price e.g. 10.00
     */
    constructor(amount: number)

    /**
     * Returns the fixed price amount.
     * @returns Fixed price amount
     */
    getFixedPrice(): number
}
```
