# dw.catalog.SortingRule

## Overview
Represents a product sorting rule for use with ProductSearchModel.

## Description
Represents a product sorting rule for use with the ProductSearchModel. A sorting rule defines how products should be ordered in search results.

```ts
declare class SortingRule extends PersistentObject {
	/**
	 * ID of the sorting rule
	 */
	readonly ID: string

	/**
	 * Returns the ID of the sorting rule.
	 */
	getID(): string
}
```
