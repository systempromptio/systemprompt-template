# dw.campaign.FixedPriceShippingDiscount

## Overview
Represents a fixed-price shipping discount (for example, "Shipping only $0.99 for iPods").

## Description
Represents a fixed-price shipping discount in the discount plan. Use to create shipping-specific fixed-amount adjustments.

```ts
declare class FixedPriceShippingDiscount extends dw.campaign.Discount {
    /**
     * The fixed price amount (read-only), for example 0.99 for a "Shipping only $0.99" discount.
     */
    readonly fixedPrice: number

    /**
     * Create a fixed-price shipping discount instance. Can be used to create a custom price adjustment.
     * @param amount - fixed price for shipping e.g. 10.00
     */
    constructor(amount: number)

    /**
     * Returns the fixed price amount.
     * @returns Fixed price amount
     */
    getFixedPrice(): number
}
```
