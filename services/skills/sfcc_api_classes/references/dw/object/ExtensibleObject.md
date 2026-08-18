# dw.object.ExtensibleObject

## Overview
Base class for persistent business objects customizable through metadata.

## Description
All persistent objects that support custom attributes derive from ExtensibleObject. Use `describe()` to access metadata and `getCustom()` to access attributes.

## All Known Subclasses
ActiveData, Basket, BonusDiscountLineItem, Campaign, Catalog, Category, CategoryAssignment, Content, CustomerAddress, CustomerGroup, and many others.

```ts
declare class ExtensibleObject extends PersistentObject {
    /** Custom attributes container (read-only). */
    custom: CustomAttributes

    /** Returns the metadata for this object or null. */
    describe(): ObjectTypeDefinition

    /** Returns the custom attributes for this object. */
    getCustom(): CustomAttributes
}
```
