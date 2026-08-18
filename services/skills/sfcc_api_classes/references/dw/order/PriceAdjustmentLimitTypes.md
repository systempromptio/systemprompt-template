# dw.order.PriceAdjustmentLimitTypes

## Overview
Helper class containing price adjustment limit types.

## Description
Helper class that defines constant types used to indicate where a price adjustment limit was created (item, order, shipping).

```ts
declare class PriceAdjustmentLimitTypes  {
    /** Constant for Price Adjustment Limit Type Item. The price adjustment limit was created at the item level. */
    static TYPE_ITEM: 'ITEM'

    /** Constant for Price Adjustment Limit Type Order. The price adjustment limit was created at the order level. */
    static TYPE_ORDER: 'ORDER'

    /** Constant for Price Adjustment Limit Type Shipping. The price adjustment limit was created at the shipping item level. */
    static TYPE_SHIPPING: 'SHIPPING'
}
```
