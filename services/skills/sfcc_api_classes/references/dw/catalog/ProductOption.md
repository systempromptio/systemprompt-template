# dw.catalog.ProductOption

## Overview
Represents a product option (e.g., color, size) with values, display name, and metadata.

## Description
Encapsulates a product option's ID, localized display name and description, default value, image, and a collection of `ProductOptionValue` entries. Values are read-only via accessors.


```ts
declare class ProductOption extends dw.object.ExtensibleObject {
	/** The default product option value (read-only). */
	static defaultValue: dw.catalog.ProductOptionValue

	/** Short description in the current locale (read-only). */
	static description: string

	/** Localized display name (read-only). */
	static displayName: string

	/** HTML-safe name for forms (read-only). */
	static htmlName: string

	/** The product option ID (read-only). */
	static ID: string

	/** Option image (read-only). */
	static image: dw.content.MediaFile

	/** Collection of `ProductOptionValue` (read-only). */
	static optionValues: dw.util.Collection

	getDefaultValue(): dw.catalog.ProductOptionValue

	getDescription(): string

	getDisplayName(): string

	getHtmlName(): string

	getHtmlName(prefix: string): string

	getID(): string

	getImage(): dw.content.MediaFile

	getOptionValues(): dw.util.Collection
}
```
