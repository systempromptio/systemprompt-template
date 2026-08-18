# dw.campaign.TotalFixedPriceDiscount

## Overview
Represents a fixed total price discount across a group of products (e.g., buy N items for $X total).

## Description
Subclass of `dw.campaign.Discount` that exposes the total fixed price amount for the discount tier.

```ts
declare class TotalFixedPriceDiscount extends dw.campaign.Discount {
    /** The total fixed price amount for the discount. */
    readonly totalFixedPrice: number

    /** Returns the total fixed price amount. */
    getTotalFixedPrice(): number
}
```

## All Known Subclasses
None
