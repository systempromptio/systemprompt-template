# dw.catalog.ProductAttributeModel

## Overview
Represents the attribute model for products: groups and definitions used to render product attributes and display values.

## Description
Provides access to product attribute groups and their definitions. Models can represent global attributes or be scoped to a category or product (including attribute values for a specific product).

```ts
declare class ProductAttributeModel  {
    /** Sorted collection of attribute groups in this model (read-only). */
    readonly attributeGroups: dw.util.Collection

    /** Unsorted collection of attribute definitions marked as order-required (read-only). */
    readonly orderRequiredAttributeDefinitions: dw.util.Collection

    /** Sorted collection of visible attribute groups (read-only). */
    readonly visibleAttributeGroups: dw.util.Collection

    /** Constructs a ProductAttributeModel representing only global product attribute groups. */
    constructor()

    /** Returns the ObjectAttributeDefinition with the given id, or null. */
    getAttributeDefinition(id: string): dw.object.ObjectAttributeDefinition | null

    /** Returns a sorted collection of attribute definitions for the given group. */
    getAttributeDefinitions(group: dw.object.ObjectAttributeGroup): dw.util.Collection

    /** Returns the attribute group with the given id, or null. */
    getAttributeGroup(id: string): dw.object.ObjectAttributeGroup | null

    /** Returns a sorted collection of attribute groups represented by this model. */
    getAttributeGroups(): dw.util.Collection

    /** Returns the localized display value for the given attribute definition (MediaFile/MarkupText for image/HTML attributes). */
    getDisplayValue(definition: dw.object.ObjectAttributeDefinition): any

    /** Returns an unsorted collection of attribute definitions marked order-required. */
    getOrderRequiredAttributeDefinitions(): dw.util.Collection

    /** Returns the attribute value for the specified definition (requires model created for a specific product). */
    getValue(definition: dw.object.ObjectAttributeDefinition): any

    /** Returns a sorted collection of visible attribute definitions for the given group. */
    getVisibleAttributeDefinitions(group: dw.object.ObjectAttributeGroup): dw.util.Collection

    /** Returns a sorted collection of visible attribute groups. */
    getVisibleAttributeGroups(): dw.util.Collection
}
```

## All Known Subclasses
None

