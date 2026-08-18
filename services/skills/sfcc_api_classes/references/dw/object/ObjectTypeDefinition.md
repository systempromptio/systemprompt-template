# dw.object.ObjectTypeDefinition

## Overview
Provides access to metadata for a system or custom object type, including its attributes and attribute groups.

## Description
Use this to inspect declared attributes and attribute groups for business objects (system or custom). It supports lookup of custom/system attribute definitions and retrieving collections used in UI rendering.

```ts
declare class ObjectTypeDefinition  {
    /** Collection of all declared attributes (system and custom). */
    attributeDefinitions: dw.util.Collection // (Read Only)

    /** Collection of all declared attribute groups. */
    attributeGroups: dw.util.Collection // (Read Only)

    /** Display name for this type. */
    displayName: string // (Read Only)

    /** Type id of the business objects. */
    ID: string // (Read Only)

    /** True if this is a system type definition. */
    system: boolean // (Read Only)

    /** Returns a collection of all declared attributes. */
    getAttributeDefinitions(): dw.util.Collection

    /** Returns the named attribute group or null. */
    getAttributeGroup(name: string): dw.object.ObjectAttributeGroup | null

    /** Returns all attribute groups. */
    getAttributeGroups(): dw.util.Collection

    /** Returns custom attribute definition by name or null. */
    getCustomAttributeDefinition(name: string): dw.object.ObjectAttributeDefinition | null

    /** Returns localized display name. */
    getDisplayName(): string

    /** Returns type id. */
    getID(): string

    /** Returns system attribute definition by name or null. */
    getSystemAttributeDefinition(name: string): dw.object.ObjectAttributeDefinition | null

    /** True if this is a system object type. */
    isSystem(): boolean
}
```
