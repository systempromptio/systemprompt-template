# dw.catalog.CategoryAssignment

## Overview
Represents a category assignment binding a product to a category with localized content (descriptions, images, and messages).

## Description
Represents a category assignment in Commerce Cloud Digital.

```
Object
  dw.object.PersistentObject
    dw.object.ExtensibleObject
      dw.catalog.CategoryAssignment
```

```ts
declare class CategoryAssignment extends ExtensibleObject {
	/**
	 * The category assignment's callout message in the current locale.

	 */
	readonly calloutMsg: MarkupText

	/**
	 * The category to which this category assignment is bound.

	 */
	readonly category: Category

	/**
	 * The category assignment's image.

	 */
	readonly image: MediaFile

	/**
	 * The category assignment's long description in the current locale.

	 */
	readonly longDescription: MarkupText

	/**
	 * The name of the category assignment in the current locale.

	 */
	readonly name: string

	/**
	 * The product to which this category assignment is bound.

	 */
	readonly product: Product

	/**
	 * The category assignment's short description in the current locale.

	 */
	readonly shortDescription: MarkupText

	/**
	 * Returns the category assignment's callout message in the current locale.
	 * @returns The callout message, or null if it wasn't found.
	 */
	getCalloutMsg(): MarkupText

	/**
	 * Returns the category to which this category assignment is bound.
	 * @returns The category.
	 */
	getCategory(): Category

	/**
	 * Returns the category assignment's image.
	 * @returns The image.
	 */
	getImage(): MediaFile

	/**
	 * Returns the category assignment's long description in the current locale.
	 * @returns The long description, or null if it wasn't found.
	 */
	getLongDescription(): MarkupText

	/**
	 * Returns the name of the category assignment in the current locale.
	 * @returns The name, or null if it wasn't found.
	 */
	getName(): string

	/**
	 * Returns the product to which this category assignment is bound.
	 * @returns The product.
	 */
	getProduct(): Product

	/**
	 * Returns the category assignment's short description in the current locale.
	 * @returns The short description, or null if it wasn't found.
	 */
	getShortDescription(): MarkupText
}
```
