# dw.order.ProductShippingModel

## Overview
Provides access to product-level shipping information such as applicable/inapplicable shipping methods and product-level shipping costs.

## Description
Instances of ProductShippingModel provide access to product-level shipping information, such as applicable or inapplicable shipping methods and shipping cost defined for the product for a specified shipping method.

```ts
declare class ProductShippingModel {
    /** Active applicable shipping methods for the product. */
    readonly applicableShippingMethods: dw.util.Collection

    /** Active inapplicable shipping methods for the product. */
    readonly inapplicableShippingMethods: dw.util.Collection

    /** Active shipping methods for which product-level shipping cost is defined. */
    readonly shippingMethodsWithShippingCost: dw.util.Collection

    /** Returns applicable shipping methods. */
    getApplicableShippingMethods(): dw.util.Collection

    /** Returns inapplicable shipping methods. */
    getInapplicableShippingMethods(): dw.util.Collection

    /** Returns the product shipping cost for a shipping method or null. */
    getShippingCost(shippingMethod: dw.order.ShippingMethod): dw.order.ProductShippingCost

    /** Returns shipping methods with shipping cost. */
    getShippingMethodsWithShippingCost(): dw.util.Collection
}
```
