# dw.catalog.ProductSearchRefinementValue

## Overview
Represents a single refinement value for product search results, including price-range bounds when applicable.

## Description
A ProductSearchRefinementValue holds display information and hit counts for a single refinement option. For price refinements it includes numeric bounds (`valueFrom`, `valueTo`). It inherits general refinement value behavior (display value, hit count, id) from SearchRefinementValue.

```ts
declare class ProductSearchRefinementValue extends SearchRefinementValue {
    /** Lower bound for price refinements (e.g. 50.00). */
    readonly valueFrom: number
    /** Upper bound for price refinements (e.g. 99.99). */
    readonly valueTo: number

    /** Returns the lower bound for price refinements. */
    getValueFrom(): number
    /** Returns the upper bound for price refinements. */
    getValueTo(): number
}
```
