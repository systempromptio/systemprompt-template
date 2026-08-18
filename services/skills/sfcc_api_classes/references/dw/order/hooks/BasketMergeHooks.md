# dw.order.hooks.BasketMergeHooks

## Overview
Hook interface for merging baskets (extension point dw.order.mergeBasket).

## Description
Represents script hooks that can be registered to merge baskets. Site cartridges register implementations that export the hook function.

```ts
declare class BasketMergeHooks {
    /**
     * The extension point name dw.order.mergeBasket.
     */
    static extensionPointMerge: "dw.order.mergeBasket"

    /**
     * Merges contents from a source basket into a destination basket.
     * @param {dw.order.Basket|null} source - the basket to merge from (may be null)
     * @param {dw.order.Basket} currentBasket - the destination basket to merge into
     * @returns {dw.system.Status}
     */
    mergeBasket(source: unknown, currentBasket: unknown): unknown
}
```
