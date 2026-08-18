# dw.catalog.Variant

## Overview
Represents a variant of a product variation. If a value is not defined on the variant, it is inherited from variation groups (by position) or the variation master.

## Description
A Variant is a specific version of a product, inheriting most data from its master product or assigned variation groups. It cannot be instantiated directly. Values not set on the variant are resolved by fallback to variation groups or the master product.

```ts
declare class Variant extends Product {
	/**
	 * All product links of the variant. Falls back to variation groups or master if not defined.
	 */
	readonly allProductLinks: Collection;

	/**
	 * The brand of the variant. Falls back to variation groups or master if not defined.
	 */
	readonly brand: string;

	/**
	 * The classification category, always inherited from the master product.
	 */
	readonly classificationCategory: Category;

	/**
	 * Custom attributes, inherited from master and can be overridden by the variant.
	 */
	readonly custom: CustomAttributes;

	/**
	 * The EAN of the variant. Falls back to variation groups or master if not defined.
	 */
	readonly EAN: string;

	/**
	 * The image of the variant. Falls back to variation groups or master if not defined.
	 */
	readonly image: MediaFile;

	/**
	 * The long description of the variant. Falls back to variation groups or master if not defined.
	 */
	readonly longDescription: MarkupText;

	/**
	 * The manufacturer name of the variant. Falls back to variation groups or master if not defined.
	 */
	readonly manufacturerName: string;

	/**
	 * The manufacturer SKU of the variant. Falls back to variation groups or master if not defined.
	 */
	readonly manufacturerSKU: string;

	/**
	 * The master product for this variant.
	 */
	readonly masterProduct: Product;

	/**
	 * The name of the variant. Falls back to variation groups or master if not defined.
	 */
	readonly name: string;

	/**
	 * The onlineFrom date of the variant. Falls back to variation groups or master if not defined.
	 */
	readonly onlineFrom: Date;

	/**
	 * The onlineTo date of the variant. Falls back to variation groups or master if not defined.
	 */
	readonly onlineTo: Date;

	/**
	 * True if the variant or its groups/master has options.
	 */
	readonly optionProduct: boolean;

	/**
	 * The page description of the variant. Falls back to variation groups or master if not defined.
	 */
	readonly pageDescription: string;

	/**
	 * The page keywords of the variant. Falls back to variation groups or master if not defined.
	 */
	readonly pageKeywords: string;

	/**
	 * The page title of the variant. Falls back to variation groups or master if not defined.
	 */
	readonly pageTitle: string;

	/**
	 * The page URL of the variant. Falls back to variation groups or master if not defined.
	 */
	readonly pageURL: string;

	/**
	 * All product links for the current site catalog. Falls back to groups or master if not defined.
	 */
	readonly productLinks: Collection;

	/**
	 * The short description of the variant. Falls back to variation groups or master if not defined.
	 */
	readonly shortDescription: MarkupText;

	/**
	 * The tax class ID of the variant. Falls back to variation groups or master if not defined.
	 */
	readonly taxClassID: string;

	/**
	 * The rendering template name of the variant. Falls back to variation groups or master if not defined.
	 */
	readonly template: string;

	/**
	 * The thumbnail image of the variant. Falls back to variation groups or master if not defined.
	 */
	readonly thumbnail: MediaFile;

	/**
	 * The sales unit of the variant. Falls back to variation groups or master if not defined.
	 */
	readonly unit: string;

	/**
	 * The unit quantity of the variant. Falls back to variation groups or master if not defined.
	 */
	readonly unitQuantity: Quantity;

	/**
	 * The UPC of the variant. Falls back to variation groups or master if not defined.
	 */
	readonly UPC: string;

	/**
	 * Returns all product links of the variant. Falls back to groups or master if not defined.
	 */
	getAllProductLinks(): Collection;

	/**
	 * Returns all product links of the specified type for the variant.
	 * @param type Type of the product link
	 */
	getAllProductLinks(type: number): Collection;

	/**
	 * Returns the brand of the variant.
	 */
	getBrand(): string;

	/**
	 * Returns the classification category (from master).
	 */
	getClassificationCategory(): Category;

	/**
	 * Returns the custom attributes of the variant.
	 */
	getCustom(): CustomAttributes;

	/**
	 * Returns the EAN of the variant.
	 */
	getEAN(): string;

	/**
	 * Returns the image of the variant.
	 */
	getImage(): MediaFile;

	/**
	 * Returns the long description of the variant.
	 */
	getLongDescription(): MarkupText;

	/**
	 * Returns the manufacturer name of the variant.
	 */
	getManufacturerName(): string;

	/**
	 * Returns the manufacturer SKU of the variant.
	 */
	getManufacturerSKU(): string;

	/**
	 * Returns the master product for this variant.
	 */
	getMasterProduct(): Product;

	/**
	 * Returns the name of the variant.
	 */
	getName(): string;

	/**
	 * Returns the onlineFrom date of the variant.
	 */
	getOnlineFrom(): Date;

	/**
	 * Returns the onlineTo date of the variant.
	 */
	getOnlineTo(): Date;

	/**
	 * Returns the page description of the variant.
	 */
	getPageDescription(): string;

	/**
	 * Returns the page keywords of the variant.
	 */
	getPageKeywords(): string;

	/**
	 * Returns the page title of the variant.
	 */
	getPageTitle(): string;

	/**
	 * Returns the page URL of the variant.
	 */
	getPageURL(): string;

	/**
	 * Returns all product links for the current site catalog. Falls back to groups or master if not defined.
	 */
	getProductLinks(): Collection;

	/**
	 * Returns all product links of the specified type for the current site catalog.
	 * @param type Type of the product link
	 */
	getProductLinks(type: number): Collection;

	/**
	 * Returns recommendations of the specified type for this variant.
	 * @param type Recommendation type
	 */
	getRecommendations(type: number): Collection;

	/**
	 * Returns the short description of the variant.
	 */
	getShortDescription(): MarkupText;

	/**
	 * Returns the tax class ID of the variant.
	 */
	getTaxClassID(): string;

	/**
	 * Returns the rendering template name of the variant.
	 */
	getTemplate(): string;

	/**
	 * Returns the thumbnail image of the variant.
	 */
	getThumbnail(): MediaFile;

	/**
	 * Returns the sales unit of the variant.
	 */
	getUnit(): string;

	/**
	 * Returns the unit quantity of the variant.
	 */
	getUnitQuantity(): Quantity;

	/**
	 * Returns the UPC of the variant.
	 */
	getUPC(): string;

	/**
	 * Returns true if the variant or its groups/master has options.
	 */
	isOptionProduct(): boolean;
}
```
