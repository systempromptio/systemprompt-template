# dw.extensions.pinterest.PinterestProduct

## Overview
Represents a product formatted for Pinterest export, exposing product attributes (title, price, images, availability, etc.) and helpers to set or get Pinterest-specific fields.

## Description
Provides getters and setters for product metadata used by Pinterest feeds: IDs, links, brand, category, GTIN, prices, images, availability, condition, size, color and return policy.

## All Known Subclasses
No known subclasses.

```ts
declare class PinterestProduct  {
    /** Availability value: in stock */
    static AVAILABILITY_IN_STOCK: 'AVAILABILITY_IN_STOCK'

    /** Availability value: out of stock */
    static AVAILABILITY_OUT_OF_STOCK: 'AVAILABILITY_OUT_OF_STOCK'

    /** Condition value: new */
    static CONDITION_NEW: 'CONDITION_NEW'

    /** Condition value: refurbished */
    static CONDITION_REFURBISHED: 'CONDITION_REFURBISHED'

    /** Condition value: used */
    static CONDITION_USED: 'CONDITION_USED'

    /** Returns the availability string for the Pinterest product. */
    getAvailability(): string

    /** Returns the Pinterest brand. */
    getBrand(): string

    /** Returns the Pinterest color label. */
    getColor(): string

    /** Returns the Pinterest color hex value. */
    getColorHex(): string

    /** Returns the URL of the color swatch image. */
    getColorImage(): URL

    /** Returns the condition string for the Pinterest product. */
    getCondition(): string

    /** Returns the Pinterest description. */
    getDescription(): string

    /** Returns the Google product category for this product. */
    getGoogleProductCategory(): string

    /** Returns the GTIN (Global Trade Item Number). */
    getGtin(): string

    /** Returns the product ID (same as Demandware product ID). */
    getID(): string

    /** Returns list of image URLs for Pinterest. */
    getImageLinks(): List

    /** Returns the ID of the item group (master product). */
    getItemGroupID(): string

    /** Returns the URL of the item group's page. */
    getItemGroupLink(): URL

    /** Returns the storefront link for this product. */
    getLink(): URL

    /** Returns the maximum price to show in Pinterest. */
    getMaxPrice(): Money

    /** Returns the minimum price to show in Pinterest. */
    getMinPrice(): Money

    /** Returns the price to show in Pinterest. */
    getPrice(): Money

    /** Returns the Pinterest product category path. */
    getProductCategory(): string

    /** Returns the Pinterest return policy. */
    getReturnPolicy(): string

    /** Returns the Pinterest size label. */
    getSize(): string

    /** Returns the Pinterest title. */
    getTitle(): string

    /** Sets the availability for the Pinterest product. */
    setAvailability(availability: string): void

    /** Sets the Pinterest brand. */
    setBrand(brand: string): void

    /** Sets the Pinterest color label. */
    setColor(color: string): void

    /** Sets the Pinterest color hex value. */
    setColorHex(colorHex: string): void

    /** Sets the URL for the color swatch image. */
    setColorImage(colorImage: URL): void

    /** Sets the condition for the Pinterest product. */
    setCondition(condition: string): void

    /** Sets the Pinterest description. */
    setDescription(description: string): void

    /** Sets the Google product category for this product. */
    setGoogleProductCategory(googleProductCategory: string): void

    /** Sets the GTIN for the product. */
    setGtin(gtin: string): void

    /** Sets the list of image URLs for Pinterest. */
    setImageLinks(imageLinks: List): void

    /** Sets the item group ID (master product ID). */
    setItemGroupID(itemGroupID: string): void

    /** Sets the item group's URL. */
    setItemGroupLink(itemGroupLink: URL): void

    /** Sets the storefront link for the product. */
    setLink(link: URL): void

    /** Sets the maximum price to show in Pinterest. */
    setMaxPrice(maxPrice: Money): void

    /** Sets the minimum price to show in Pinterest. */
    setMinPrice(minPrice: Money): void

    /** Sets the price to show in Pinterest. */
    setPrice(price: Money): void

    /** Sets the Pinterest category path. */
    setProductCategory(productCategory: string): void

    /** Sets the Pinterest return policy. */
    setReturnPolicy(returnPolicy: string): void

    /** Sets the Pinterest size label. */
    setSize(size: string): void

    /** Sets the Pinterest title. */
    setTitle(title: string): void
}
```
