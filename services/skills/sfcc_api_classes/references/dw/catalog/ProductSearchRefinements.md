# dw.catalog.ProductSearchRefinements

## Overview
Provides access to refinement definitions and values for product search results, used to render filtering UI and build refine/relax links.

## Description
Use ProductSearchRefinements to enumerate available refinement definitions (category, attribute, price, promotion), fetch the appropriate definition for a search result, and obtain refinement values and counts for rendering and interaction. It exposes helpers to get category-child refinements, all refinement values (including broadening options), and single-value lookups.

```ts
declare class ProductSearchRefinements extends SearchRefinements {
    /** The category refinement definition for the search result (if any). */
    readonly categoryRefinementDefinition: ProductSearchRefinementDefinition
    /** The price refinement definition for the search result (if any). */
    readonly priceRefinementDefinition: ProductSearchRefinementDefinition
    /** The promotion refinement definition for the search result (if any). */
    readonly promotionRefinementDefinition: ProductSearchRefinementDefinition

    /** Returns all refinement values for the passed definition, including broadening values. */
    getAllRefinementValues(definition: ProductSearchRefinementDefinition): Collection<ProductSearchRefinementValue>
    /** Returns the category refinement definition appropriate for the search result. */
    getCategoryRefinementDefinition(): ProductSearchRefinementDefinition
    /** Returns child category refinement values under the provided category. */
    getNextLevelCategoryRefinementValues(category: Category): Collection<ProductSearchRefinementValue>
    /** Returns the price refinement definition appropriate for the search result. */
    getPriceRefinementDefinition(): ProductSearchRefinementDefinition
    /** Returns the promotion refinement definition appropriate for the search result. */
    getPromotionRefinementDefinition(): ProductSearchRefinementDefinition
    /** Returns the refinement value for definition + value. */
    getRefinementValue(definition: ProductSearchRefinementDefinition, value: string): ProductSearchRefinementValue
    /** Returns the refinement value for attribute name + value. */
    getRefinementValue(name: string, value: string): ProductSearchRefinementValue
    /** Returns the refinement values that are present in the current search result for the given definition. */
    getRefinementValues(definition: ProductSearchRefinementDefinition): Collection<ProductSearchRefinementValue>
}
```
