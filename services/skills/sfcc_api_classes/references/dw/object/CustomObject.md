# dw.object.CustomObject

## Overview
Represents a custom object and its attributes.

## Description
Container for a custom object instance. Exposes `custom` attributes and a read-only `type` identifying the custom object type.

```ts
declare class CustomObject extends ExtensibleObject {
    /** The custom attributes for this object (read-only). */
    custom: CustomAttributes

    /** The custom object type (read-only). */
    type: string

    /** Returns the custom attributes for this object. */
    getCustom(): CustomAttributes

    /** Returns the type of the CustomObject. */
    getType(): string
}
```
