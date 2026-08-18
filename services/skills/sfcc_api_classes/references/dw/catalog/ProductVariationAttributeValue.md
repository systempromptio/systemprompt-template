# dw.catalog.ProductVariationAttributeValue

## Overview
Represents a product variation attribute value with display and image information.

## Description
Represents a product variation attribute value. Provides access to value metadata, localized display information, and associated images for variant attributes (typically color).

```ts
declare class ProductVariationAttributeValue  {
	/**
	 * The description of the product variation attribute value in the current locale.
	 */
	readonly description: string

	/**
	 * The display value for the product variation attribute value, which can be used in the user interface.
	 */
	readonly displayValue: string

	/**
	 * The ID of the product variation attribute value.
	 */
	readonly ID: string

	/**
	 * The value for the product variation attribute value.
	 */
	readonly value: Object

	/**
	 * Returns true if the specified object is equal to this object.
	 * @param obj - the object to test
	 * @returns true if the specified object is equal to this object
	 */
	equals(obj: Object): boolean

	/**
	 * Returns the description of the product variation attribute value in the current locale.
	 * @returns the description or null if not found
	 */
	getDescription(): string

	/**
	 * Returns the display value for the product variation attribute value, which can be used in the user interface.
	 * @returns the display value
	 */
	getDisplayValue(): string

	/**
	 * Returns the ID of the product variation attribute value.
	 * @returns the ID
	 */
	getID(): string

	/**
	 * Calls getImages(String) and returns the image at the specific index.
	 * Returns null if images exist for this view type and variant but not at the specified index.
	 * Falls back to master product image at the specified index if no variant images exist.
	 * @param viewtype - the view type annotated to image
	 * @param index - the index number of the image within image list
	 * @returns the MediaFile or null
	 */
	getImage(viewtype: string, index: number): MediaFile

	/**
	 * Calls getImages(String) and returns the first image of the list.
	 * Built specifically for handling color swatches in an apparel site.
	 * Falls back to the first master product image if no variant images exist.
	 * @param viewtype - the view type annotated to image
	 * @returns the MediaFile or null
	 */
	getImage(viewtype: string): MediaFile

	/**
	 * Returns all images that match the given view type and have the variant value of this value (typically 'color').
	 * Images are returned in order of their index number ascending.
	 * Falls back to master product images if no variant images are defined.
	 * @param viewtype - the view type annotated to images
	 * @returns a list of MediaFile objects, possibly empty
	 */
	getImages(viewtype: string): List

	/**
	 * Returns the value for the product variation attribute value.
	 * @returns the value
	 */
	getValue(): Object

	/**
	 * Calculates the hash code for a product variation attribute value.
	 * @returns the hash code
	 */
	hashCode(): number
}
```
