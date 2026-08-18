# dw.object.SimpleExtensible

## Overview
Alternative base class for objects with dynamic custom attributes not validated against metadata.

## Description
Allows arbitrary custom attributes to be set and retrieved at runtime via `getCustom()`. Use when objects do not rely on the metadata system.

```ts
declare class SimpleExtensible  {
    /** The custom attributes for this object. */
    custom: dw.object.CustomAttributes // (Read Only)

    /** Returns the custom attributes container. */
    getCustom(): dw.object.CustomAttributes
}
```
