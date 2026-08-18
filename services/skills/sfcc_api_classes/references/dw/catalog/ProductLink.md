# dw.catalog.ProductLink

## Overview
Represents a link between two products (cross-sell, up-sell, accessory, replacement, etc.).

## Description
Encapsulates the relationship between a source product and a target product with a numeric type code. Several `LINKTYPE_*` constants define the link semantics (e.g., `LINKTYPE_CROSS_SELL`, `LINKTYPE_UP_SELL`).


```ts
declare class ProductLink {
	/** Accessory link type. */
	static LINKTYPE_ACCESSORY: 4

	/** Alternative order unit link type. */
	static LINKTYPE_ALT_ORDERUNIT: 6

	/** Cross-sell link type. */
	static LINKTYPE_CROSS_SELL: 1

	/** Newer version link type. */
	static LINKTYPE_NEWER_VERSION: 5

	/** Other/miscellaneous link type. */
	static LINKTYPE_OTHER: 8

	/** Replacement link type. */
	static LINKTYPE_REPLACEMENT: 2

	/** Spare part link type. */
	static LINKTYPE_SPARE_PART: 7

	/** Up-sell link type. */
	static LINKTYPE_UP_SELL: 3

	/** The source product (read-only). */
	sourceProduct: dw.catalog.Product

	/** The target product (read-only). */
	targetProduct: dw.catalog.Product

	/** Numeric type code for this link (read-only). */
	typeCode: number

	getSourceProduct(): dw.catalog.Product

	getTargetProduct(): dw.catalog.Product

	getTypeCode(): number
}
```
