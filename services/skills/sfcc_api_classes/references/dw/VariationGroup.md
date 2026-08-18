# dw.catalog.VariationGroup

## Overview
Represents a product variation group (a variation of a master product) and provides accessors for variation-specific
attributes, product links, recommendations and metadata.

## Description
A VariationGroup exposes attributes that may be defined on the variation or inherited from its master product.
It provides methods to read product links, recommendations and descriptive metadata (name, descriptions, images, etc.).

```ts
declare class VariationGroup extends Product {
    /** Returns all product links of the variation group (or the master if none defined). */
    getAllProductLinks(): Collection

    /** Returns all product links of the specified type. @param type */
    getAllProductLinks(type: number): Collection

    /** Returns the brand (variation value or master fallback). */
    getBrand(): string

    /** Returns the classification category (always inherited from master). */
    getClassificationCategory(): Category

    /** Returns custom attributes for this variation group. */
    getCustom(): CustomAttributes

    /** Returns the EAN of the variation group (or master fallback). */
    getEAN(): string

    /** Returns the image for the variation group (or master fallback). */
    getImage(): MediaFile

    /** Returns the long description (MarkupText) for this variation group. */
    getLongDescription(): MarkupText

    /** Returns the manufacturer name (variation or master). */
    getManufacturerName(): string

    /** Returns the manufacturer SKU (variation or master). */
    getManufacturerSKU(): string

    /** Returns the master Product associated with this variation. */
    getMasterProduct(): Product

    /** Returns the name (variation or master). */
    getName(): string

    /** Returns the onlineFrom date (variation or master). */
    getOnlineFrom(): Date

    /** Returns the onlineTo date (variation or master). */
    getOnlineTo(): Date

    /** Returns the page description (variation or master). */
    getPageDescription(): string

    /** Returns the page keywords (variation or master). */
    getPageKeywords(): string

    /** Returns the page title (variation or master). */
    getPageTitle(): string

    /** Returns the page URL (variation or master). */
    getPageURL(): string

    /** Returns product links assigned to this variation that target the current site catalog. */
    getProductLinks(): Collection

    /** Returns product links of the specified type. @param type */
    getProductLinks(type: number): Collection

    /** Returns recommendations of the given type for this variation group. @param type */
    getRecommendations(type: number): Collection

    /** Returns short description (MarkupText) for this variation group. */
    getShortDescription(): MarkupText

    /** Returns tax class id (variation or master). */
    getTaxClassID(): string

    /** Returns rendering template name (variation or master). */
    getTemplate(): string

    /** Returns the thumbnail MediaFile (variation or master). */
    getThumbnail(): MediaFile

    /** Returns sales unit defined by master product. */
    getUnit(): string

    /** Returns unit quantity defined by master product. */
    getUnitQuantity(): Quantity

    /** Returns UPC code (variation or master). */
    getUPC(): string

    /** Returns true if the variation group has options (or master has options). */
    isOptionProduct(): boolean
}
```
