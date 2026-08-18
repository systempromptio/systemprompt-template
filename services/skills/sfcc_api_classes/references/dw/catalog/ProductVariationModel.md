# dw.catalog.ProductVariationModel

## Overview
Provides complete variation information for master products, including variation attributes, values, groups, and variants with selection state management.

## Description
Represents the complete variation information for a master product. Provides access to variation attributes, their values, variation groups, and variants. Maintains selected variation attribute values representing customer selections in the storefront. Only considers complete and online variants. For master products, no attributes are pre-selected; for variant products, all attribute values are pre-selected. For variation groups, only attributes not defined by the group can be modified.

```ts
declare class ProductVariationModel  {
	/**
	 * The default variant of this model's master product. If no default variant is defined, returns an arbitrary variant.
	 */
	readonly defaultVariant: Variant

	/**
	 * The master of the product variation.
	 */
	readonly master: Product

	/**
	 * A collection of product variation attributes of the variation.
	 */
	readonly productVariationAttributes: Collection

	/**
	 * The variant currently selected for this variation model. Returns null if no variant is selected.
	 */
	readonly selectedVariant: Variant

	/**
	 * The variants currently selected for this variation model. Returns an empty collection if no variant is selected.
	 */
	readonly selectedVariants: Collection

	/**
	 * The collection of product variants of this variation model. Only includes online variants. Use Product.getVariants() for all variants.
	 */
	readonly variants: Collection

	/**
	 * The collection of variation groups of this variation model. Only includes online variation groups. Use Product.getVariationGroups() for all.
	 */
	readonly variationGroups: Collection

	/**
	 * Returns the values for the specified attribute. Only values existing for currently online and complete variants are returned.
	 * @param attribute - the attribute whose values will be returned
	 * @returns collection of ProductVariationAttributeValue instances, sorted by explicit sort order
	 */
	getAllValues(attribute: ProductVariationAttribute): Collection

	/**
	 * Returns the default variant of this model's master product.
	 * @returns the default variant
	 */
	getDefaultVariant(): Variant

	/**
	 * Returns values for the specified attribute, filtered based on currently selected values.
	 * @param attribute - the attribute
	 * @returns collection of filtered ProductVariationAttributeValue instances
	 */
	getFilteredValues(attribute: ProductVariationAttribute): Collection

	/**
	 * Returns an HTML representation of the product variation attribute id.
	 * @param attribute - the product variation attribute
	 * @returns HTML representation
	 */
	getHtmlName(attribute: ProductVariationAttribute): string

	/**
	 * Returns an HTML representation of the product variation attribute id with custom prefix.
	 * @param prefix - custom prefix
	 * @param attribute - the product variation attribute
	 * @returns HTML representation with prefix
	 */
	getHtmlName(prefix: string, attribute: ProductVariationAttribute): string

	/**
	 * Returns the first image appropriate for the currently selected attribute values.
	 * @param viewtype - the view type
	 * @param attribute - the variation attribute
	 * @param value - the variation attribute value
	 * @returns the MediaFile or null
	 */
	getImage(viewtype: string, attribute: ProductVariationAttribute, value: ProductVariationAttributeValue): MediaFile

	/**
	 * Returns an image appropriate for the current selected variation values with the specific index.
	 * @param viewtype - the view type
	 * @param index - the image index
	 * @returns the MediaFile or null
	 */
	getImage(viewtype: string, index: number): MediaFile

	/**
	 * Returns the first image appropriate for the current selected variation values.
	 * @param viewtype - the view type
	 * @returns the MediaFile or null
	 */
	getImage(viewtype: string): MediaFile

	/**
	 * Returns the images appropriate for the currently selected attribute values.
	 * @param viewtype - the view type
	 * @returns list of MediaFile objects
	 */
	getImages(viewtype: string): List

	/**
	 * Returns the master of the product variation.
	 * @returns the master product
	 */
	getMaster(): Product

	/**
	 * Returns the product variation attribute for the specific id, or null if there is no product variation attribute for that id.
	 * @param id - the attribute ID
	 * @returns the ProductVariationAttribute or null
	 */
	getProductVariationAttribute(id: string): ProductVariationAttribute

	/**
	 * Returns a collection of product variation attributes of the variation.
	 * @returns collection of ProductVariationAttribute instances
	 */
	getProductVariationAttributes(): Collection

	/**
	 * Returns the selected value for the specified product variation attribute.
	 * @param attribute - the product variation attribute
	 * @returns the selected ProductVariationAttributeValue or null
	 */
	getSelectedValue(attribute: ProductVariationAttribute): ProductVariationAttributeValue

	/**
	 * Returns the variant currently selected for this variation model.
	 * @returns the selected Variant or null
	 */
	getSelectedVariant(): Variant

	/**
	 * Returns the variants currently selected for this variation model.
	 * @returns collection of selected variants
	 */
	getSelectedVariants(): Collection

	/**
	 * Returns the collection of product variants of this variation model.
	 * @returns collection of variants
	 */
	getVariants(): Collection

	/**
	 * Returns the variants that match the specified filter conditions.
	 * @param filter - HashMap with attribute IDs as keys and attribute values as values
	 * @returns collection of matching variants
	 */
	getVariants(filter: HashMap): Collection

	/**
	 * Returns the collection of variation groups of this variation model.
	 * @returns collection of variation groups
	 */
	getVariationGroups(): Collection

	/**
	 * Returns the value for the specified variant or variation group product and variation attribute.
	 * @param variantOrVariationGroup - the variant or variation group product
	 * @param attribute - the variation attribute
	 * @returns the ProductVariationAttributeValue or null
	 */
	getVariationValue(variantOrVariationGroup: Product, attribute: ProductVariationAttribute): ProductVariationAttributeValue

	/**
	 * Returns true if any variant is available with the specified value of the specified variation attribute.
	 * @param attribute - the variation attribute
	 * @param value - the variation attribute value
	 * @returns true if orderable variants exist
	 */
	hasOrderableVariants(attribute: ProductVariationAttribute, value: ProductVariationAttributeValue): boolean

	/**
	 * Identifies if the specified product variation attribute value is the one currently selected.
	 * @param attribute - the product variation attribute
	 * @param value - the product variation attribute value
	 * @returns true if selected
	 */
	isSelectedAttributeValue(attribute: ProductVariationAttribute, value: ProductVariationAttributeValue): boolean

	/**
	 * Applies a selected attribute value to this model instance.
	 * @param variationAttributeID - the ID of the variation attribute
	 * @param variationAttributeValueID - the ID of the variation attribute value
	 */
	setSelectedAttributeValue(variationAttributeID: string, variationAttributeValueID: string): void

	/**
	 * Constructs a URL to select a set of variation attribute values.
	 * @param action - the action
	 * @param varAttrAndValues - attribute/value pairs
	 * @returns the URL
	 */
	url(action: string, ...varAttrAndValues: Object): URL

	/**
	 * Generates a URL for selecting a value for a given variation attribute.
	 * @param action - the action
	 * @param attribute - the variation attribute
	 * @param value - the variation attribute value
	 * @returns the URL string
	 */
	urlSelectVariationValue(action: string, attribute: ProductVariationAttribute, value: ProductVariationAttributeValue): string

	/**
	 * Generates a URL for unselecting a value for a given variation attribute.
	 * @param action - the action
	 * @param attribute - the variation attribute
	 * @returns the URL string
	 */
	urlUnselectVariationValue(action: string, attribute: ProductVariationAttribute): string
}
```
