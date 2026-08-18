# dw.order.hooks.CalculateHooks

## Overview
Hook interface for customizing order and basket calculation (extension points dw.order.calculate, dw.order.calculateShipping, dw.order.calculateTax).

## Description
Represents script hooks to customize order and basket calculation. Site cartridges register implementations that export the hook functions.

```ts
declare class CalculateHooks {
    /**
     * The extension point name dw.order.calculate.
     */
    static extensionPointCalculate: "dw.order.calculate"

    /**
     * The extension point name dw.order.calculateShipping.
     */
    static extensionPointCalculateShipping: "dw.order.calculateShipping"

    /**
     * The extension point name dw.order.calculateTax.
     */
    static extensionPointCalculateTax: "dw.order.calculateTax"

    /**
     * Provides a single place for the line item container calculation.
     * @param {dw.order.LineItemCtnr} lineItemCtnr - the line item container to be (re)calculated
     * @returns {dw.system.Status}
     */
    calculate(lineItemCtnr: unknown): unknown

    /**
     * Provides a single place for shipping calculation during line item container calculation.
     * @param {dw.order.LineItemCtnr} lineItemCtnr
     * @returns {dw.system.Status}
     */
    calculateShipping(lineItemCtnr: unknown): unknown

    /**
     * Provides a single place for tax calculation during line item container calculation.
     * @param {dw.order.LineItemCtnr} lineItemCtnr
     * @returns {dw.system.Status}
     */
    calculateTax(lineItemCtnr: unknown): unknown
}
```
