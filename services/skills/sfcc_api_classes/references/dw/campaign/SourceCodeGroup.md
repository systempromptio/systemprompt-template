# dw.campaign.SourceCodeGroup

## Overview
Defines a named group of source codes; groups are often pattern-based and assigned to pricebooks.

## Description
A SourceCodeGroup represents a collection of source codes used to qualify customers for pricing or site experiences. It is persistent and extensible.

```ts
declare class SourceCodeGroup extends dw.object.ExtensibleObject {
    /** The identifier of the source code group. */
    readonly ID: string

    /** Collection of PriceBooks the group is assigned to. */
    readonly priceBooks: Collection

    /** Returns the ID of the SourceCodeGroup. */
    getID(): string

    /** Returns a Collection of PriceBooks assigned to the group. */
    getPriceBooks(): Collection
}
```

## All Known Subclasses
None
