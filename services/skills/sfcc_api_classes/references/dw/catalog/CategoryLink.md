# dw.catalog.CategoryLink

## Overview
Represents a directed relationship between two catalog categories for marketing similar or related product groups.

## Description
A CategoryLink represents a directed relationship between two catalog categories. Merchants create category links in order to market similar or related groups of products.

```
Object
  dw.catalog.CategoryLink
```

```ts
declare class CategoryLink  {
	/**
	 * Represents an accessory category link.
	 */
	static LINKTYPE_ACCESSORY: 2

	/**
	 * Represents a cross-sell category link.
	 */
	static LINKTYPE_CROSS_SELL: 4

	/**
	 * Represents a miscellaneous category link.
	 */
	static LINKTYPE_OTHER: 1

	/**
	 * Represents a spare part category link.
	 */
	static LINKTYPE_SPARE_PART: 6

	/**
	 * Represents an up-sell category link.
	 */
	static LINKTYPE_UP_SELL: 5

	/**
	 * The object for the relation 'sourceCategory'.
	 * @readonly
	 */
	readonly sourceCategory: Category

	/**
	 * The object for the relation 'targetCategory'.
	 * @readonly
	 */
	readonly targetCategory: Category

	/**
	 * The type of this category link (see constants).
	 * @readonly
	 */
	readonly typeCode: number

	/**
	 * Returns the object for the relation 'sourceCategory'.
	 * @returns The source category.
	 */
	getSourceCategory(): Category

	/**
	 * Returns the object for the relation 'targetCategory'.
	 * @returns The target category.
	 */
	getTargetCategory(): Category

	/**
	 * Returns the type of this category link (see constants).
	 * @returns The type of the link.
	 */
	getTypeCode(): number
}
```
