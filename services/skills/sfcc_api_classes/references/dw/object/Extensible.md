# dw.object.Extensible

## Overview
Base class alternative to ExtensibleObject for objects customizable through the metadata system.

## Description
Provides `describe()` to access object-type metadata and `getCustom()` to retrieve/store attribute values. Used where ExtensibleObject is not appropriate.

## All Known Subclasses
AbstractItem, AbstractItemCtnr, Appeasement, AppeasementItem, Invoice, InvoiceItem, Return, ReturnCase, ReturnCaseItem, ReturnItem, ShippingOrder, ShippingOrderItem, TrackingInfo

```ts
declare class Extensible  {
    /** Custom attributes container (read-only). */
    custom: CustomAttributes

    /** Returns the metadata for this object or null. */
    describe(): ObjectTypeDefinition

    /** Returns the custom attributes for this object. */
    getCustom(): CustomAttributes
}
```
