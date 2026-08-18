# dw.catalog.SortingRule

## Overview
Represents a product sorting rule usable by search models.

## Description
Encapsulates an identifier for a sorting rule. Used by `ProductSearchModel` to apply ordering.

## All Known Subclasses
None

```ts
declare class SortingRule extends dw.object.PersistentObject {
    /** The ID of the sorting rule. */
    ID: string

    /** Returns the sorting rule ID. */
    getID(): string
}
```
