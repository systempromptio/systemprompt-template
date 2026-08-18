# dw.campaign.SlotContent

## Overview
Represents content entries returned for a slot configuration (products, content, categories, or markup text).

## Description
Holds slot-specific information: callout message, content collection (one of Product, Content, Category, or MarkupText), custom attributes, recommender name (for recommendation slots), and the slot identifier.

```ts
declare class SlotContent  {
    /** The callout message for the slot. */
    readonly calloutMsg: MarkupText

    /** Collection of content for the slot (Product | Content | Category | MarkupText). */
    readonly content: Collection

    /** Custom attributes map for the slot. */
    readonly custom: Map

    /** Recommender name for slots of type 'Recommendation'. */
    readonly recommenderName: string

    /** Unique slot identifier. */
    readonly slotID: string

    /** Returns the callout message for the slot. */
    getCalloutMsg(): MarkupText

    /** Returns the content collection for the slot. */
    getContent(): Collection

    /** Returns the custom attributes for the slot. */
    getCustom(): Map

    /** Returns the recommender name for recommendation slots. */
    getRecommenderName(): string

    /** Returns the unique slot ID. */
    getSlotID(): string
}
```

## All Known Subclasses
None
