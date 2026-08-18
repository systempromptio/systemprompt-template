# dw.catalog.StoreGroup

## Overview
Represents a group of stores for marketing or organizational purposes.

## Description
Holds the group's ID, name and collection of assigned stores.

## All Known Subclasses
None

```ts
declare class StoreGroup extends dw.object.ExtensibleObject {
    /** The store group ID. */
    ID: string

    /** The store group name. */
    name: string

    /** Collection of `Store` objects in this group. */
    stores: dw.util.Collection

    /** Returns the store group ID. */
    getID(): string

    /** Returns the store group name. */
    getName(): string

    /** Returns a collection of stores assigned to this group. */
    getStores(): dw.util.Collection
}
```
