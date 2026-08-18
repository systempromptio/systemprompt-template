# dw.catalog.StoreGroup

## Overview
Represents a store group for organizing stores for marketing purposes.

## Description
Represents a store group. Store groups can be used to group the stores for different marketing purposes.

```ts
declare class StoreGroup extends ExtensibleObject {
	/**
	 * ID of the store group
	 */
	readonly ID: string

	/**
	 * Name of the store group
	 */
	readonly name: string

	/**
	 * All stores assigned to the store group
	 */
	readonly stores: Collection

	/**
	 * Returns the ID of the store group.
	 */
	getID(): string

	/**
	 * Returns the name of the store group.
	 */
	getName(): string

	/**
	 * Returns all the stores that are assigned to the store group.
	 */
	getStores(): Collection
}
```
