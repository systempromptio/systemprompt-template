# dw.catalog.Recommendation

## Overview
Represents a recommendation entry returned by Commerce Cloud Digital (product or category recommendation metadata).

## Description
Contains localized title, descriptions, image and references to the recommended and source items. Recommendation type is expressed as an integer and recommendation targets are currently products.

```ts
declare class Recommendation extends ExtensibleObject {
    /** Represents a cross-sell recommendation. */
    static RECOMMENDATION_TYPE_CROSS_SELL: 1

    /** Represents an up-sell recommendation. */
    static RECOMMENDATION_TYPE_UP_SELL: 2

    /** Represents a recommendation that is neither cross-sell nor up-sell. */
    static RECOMMENDATION_TYPE_OTHER: 3

    /** The recommendation's callout message in the current locale. */
    calloutMsg: MarkupText

    /** The catalog containing the recommendation. */
    catalog: Catalog

    /** The recommendation's image. */
    image: MediaFile

    /** The recommendation's long description in the current locale. */
    longDescription: MarkupText

    /** The name of the recommended item in the current locale. */
    name: string

    /** The type of the recommendation (integer). */
    recommendationType: number

    /** Reference to the recommended item (typically a Product). */
    recommendedItem: Object

    /** The ID of the recommended item (product ID). */
    recommendedItemID: string

    /** The recommendation's short description in the current locale. */
    shortDescription: MarkupText

    /** Reference to the source item (Product or Category). */
    sourceItem: Object

    /** The ID of the recommendation source item (product ID or category name). */
    sourceItemID: string

    /** Returns the recommendation's callout message in the current locale. */
    getCalloutMsg(): MarkupText

    /** Return the catalog containing the recommendation. */
    getCatalog(): Catalog

    /** Returns the recommendation's image. */
    getImage(): MediaFile

    /** Returns the recommendation's long description in the current locale. */
    getLongDescription(): MarkupText

    /** Returns the name of the recommended item in the current locale. */
    getName(): string

    /** Returns the type of the recommendation (integer). */
    getRecommendationType(): number

    /** Return a reference to the recommended item (may be null). */
    getRecommendedItem(): Object

    /** Return the ID of the recommended item. */
    getRecommendedItemID(): string

    /** Returns the recommendation's short description in the current locale. */
    getShortDescription(): MarkupText

    /** Return a reference to the source item (Product or Category). */
    getSourceItem(): Object

    /** Return the ID of the recommendation source item. */
    getSourceItemID(): string
}
```
