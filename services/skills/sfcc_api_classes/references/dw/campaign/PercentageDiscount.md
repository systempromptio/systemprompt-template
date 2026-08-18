# dw.campaign.PercentageDiscount

## Overview
Represents a percentage-off discount in a discount plan (for example, "10% off all T-Shirts").

## Description
Represents a percentage discount value used in promotions. Can be constructed on the fly to create custom price adjustments.

```ts
declare class PercentageDiscount extends dw.campaign.Discount {
    /**
     * The percentage discount value (read-only), e.g. 10.00 for 10%.
     */
    readonly percentage: number

    /**
     * Create a percentage discount instance.
     * @param percentage - percentage value, e.g. 15.00 for 15%
     */
    constructor(percentage: number)

    /**
     * Returns the percentage discount value.
     * @returns Discount percentage value
     */
    getPercentage(): number
}
```
