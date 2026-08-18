# dw.suggest.SuggestedCategory

## Overview
Represents a suggested catalog category based on search input.

## Description
Provides access to a suggested catalog category. Use getCategory() to retrieve the actual Category object.

```ts
declare class SuggestedCategory  {
	/**
	 * The actual Category object corresponding to this suggested category.
	 * @readonly
	 */
	readonly category: Category

	/**
	 * Returns the actual Category object corresponding to this suggested category.
	 * @returns The category object
	 */
	getCategory(): Category
}
```
