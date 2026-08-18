# dw.catalog.ProductVariationAttribute

## Overview
Represents a product variation attribute that defines how products vary within a variation group.

## Description
Represents a product variation attribute. The ID matches the product attribute definition and can be used to access variation-specific display information.

```ts
declare class ProductVariationAttribute  {
	/**
	 * The ID of the product attribute definition related to this variation attribute.
	 * Matches the value returned by ObjectAttributeDefinition.getID() for the appropriate product attribute definition.
	 * This ID is generally different than the ID returned by getID().
	 */
	readonly attributeID: string

	/**
	 * The display name for the product variation attribute, which can be used in the user interface.
	 */
	readonly displayName: string

	/**
	 * The ID of the product variation attribute.
	 */
	readonly ID: string

	/**
	 * Returns the ID of the product attribute definition related to this variation attribute.
	 * @returns the ID of the product attribute definition of this variation attribute
	 */
	getAttributeID(): string

	/**
	 * Returns the display name for the product variation attribute, which can be used in the user interface.
	 * @returns the display name for the product variation attribute
	 */
	getDisplayName(): string

	/**
	 * Returns the ID of the product variation attribute.
	 * @returns the ID of the product variation attribute
	 */
	getID(): string
}
```
