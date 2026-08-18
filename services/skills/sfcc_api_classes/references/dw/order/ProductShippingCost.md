# dw.order.ProductShippingCost

## Overview
Instances of ProductShippingCost represent product specific shipping costs.

## Description
Instances of ProductShippingCost represent product specific shipping costs. Use ProductShippingModel.getShippingCost(ShippingMethod) to get the shipping cost for a specific product.

```ts
declare class ProductShippingCost {
    /** The shipping amount. */
    readonly amount: dw.value.Money

    /** Returns true if shipping cost is a fixed-price shipping cost. */
    readonly fixedPrice: boolean

    /** Returns true if shipping cost is a surcharge to the shipment shipping cost. */
    readonly surcharge: boolean

    /** Returns the shipping amount. */
    getAmount(): dw.value.Money

    /** Returns true if fixed-price shipping cost. */
    isFixedPrice(): boolean

    /** Returns true if surcharge shipping cost. */
    isSurcharge(): boolean
}
```
