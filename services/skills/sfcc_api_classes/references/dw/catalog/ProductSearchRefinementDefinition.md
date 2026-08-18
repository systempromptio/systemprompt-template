# dw.catalog.ProductSearchRefinementDefinition

## Overview
Describes a single refinement option available for product search (category, price, or promotion type).

## Description
Represents metadata about a refinement used by the product search UI and API. It indicates whether the refinement is a category refinement, a price refinement, or a promotion refinement. Inherits generic refinement attributes (attribute ID, display name, cutoff thresholds) from SearchRefinementDefinition.

```ts
declare class ProductSearchRefinementDefinition extends SearchRefinementDefinition {
    /** True when this refinement is a category refinement. */
    readonly categoryRefinement: boolean
    /** True when this refinement is a price refinement. */
    readonly priceRefinement: boolean
    /** True when this refinement is a promotion refinement. */
    readonly promotionRefinement: boolean

    /** Returns true if this is a category refinement. */
    isCategoryRefinement(): boolean
    /** Returns true if this is a price refinement. */
    isPriceRefinement(): boolean
    /** Returns true if this is a promotion refinement. */
    isPromotionRefinement(): boolean
}
```
