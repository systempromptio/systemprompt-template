# dw.object.ActiveData

## Overview
Represents the active data for an object in Commerce Cloud Digital.

## Description
Represents the active data for an object. Provides access to custom attributes and a flag indicating
whether the active data exists for the object.

## All Known Subclasses
CustomerActiveData, ProductActiveData

```ts
declare class ActiveData extends ExtensibleObject {
    /** The custom attributes for this object (read-only). */
    custom: CustomAttributes

    /** True when no active data exists for the object. */
    empty: boolean

    /** Returns the custom attributes for this object. */
    getCustom(): CustomAttributes

    /** Returns true if the ActiveData doesn't exist for the object. */
    isEmpty(): boolean
}
```
