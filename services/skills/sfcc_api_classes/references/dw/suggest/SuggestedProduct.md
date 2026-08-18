# dw.suggest.SuggestedProduct

## Overview
Represents a suggested product based on search input.

## Description
Provides access to a suggested product. Use getProductSearchHit() to retrieve the actual ProductSearchHit object.

```ts
declare class SuggestedProduct  {
	/**
	 * The actual ProductSearchHit object corresponding to this suggested product.
	 * @readonly
	 */
	readonly productSearchHit: ProductSearchHit

	/**
	 * Returns the actual ProductSearchHit object corresponding to this suggested product.
	 * @returns The product search hit
	 */
	getProductSearchHit(): ProductSearchHit
}
```
