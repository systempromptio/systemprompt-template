# dw.suggest.ProductSuggestions

## Overview
Container providing access to products found using suggested search terms.

## Description
Provides access to products found using suggested terms as search criteria. Product lookup executes in the current catalog and locale. Extends Suggestions to provide suggested terms access alongside product results.

```ts
declare class ProductSuggestions extends Suggestions {
	/**
	 * List of products found using suggested terms (Read Only)
	 */
	readonly suggestedProducts: Iterator
	
	/**
	 * Returns a list of products found using suggested terms as search criteria
	 * @returns iterator containing SuggestedProduct instances, may be empty
	 */
	getSuggestedProducts(): Iterator
}
```
