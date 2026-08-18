# dw.object.ObjectAttributeGroup

## Overview
Represents a group of object attributes.

## Description
Holds a collection of attribute definitions belonging to a named attribute group for an object type. Provides metadata such as ID, display name, description, and whether the group is system-defined.

```ts
declare class ObjectAttributeGroup  {
    /** All attribute definitions for this group. */
    attributeDefinitions: dw.util.Collection // (Read Only)

    /** Description for the current locale. */
    description: string // (Read Only)

    /** Display name of this group. */
    displayName: string // (Read Only)

    /** The ID of this group. */
    ID: string // (Read Only)

    /** The owning object type definition. */
    objectTypeDefinition: dw.object.ObjectTypeDefinition // (Read Only)

    /** True if this is a system (pre-defined) group. */
    system: boolean // (Read Only)

    /** Returns all attribute definitions for this group. */
    getAttributeDefinitions(): dw.util.Collection

    /** Returns the description in the current locale. */
    getDescription(): string

    /** Returns the display name. */
    getDisplayName(): string

    /** Returns the ID of this group. */
    getID(): string

    /** Returns the owning object type definition. */
    getObjectTypeDefinition(): dw.object.ObjectTypeDefinition

    /** Identifies if this is a system attribute group. */
    isSystem(): boolean
}
```
