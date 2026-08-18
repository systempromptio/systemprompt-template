# dw.catalog.SearchRefinementDefinition

## Overview
Base class describing a search refinement (attribute, price, category) used by search models.

## Description
Holds metadata for a refinement: attribute ID, display name, cutoff threshold and value type. Subclasses provide product- or content-specific behavior.

All Known Subclasses
- ContentSearchRefinementDefinition
- ProductSearchRefinementDefinition

```ts
declare class SearchRefinementDefinition extends ExtensibleObject {
    /** The attribute ID or empty string if not an attribute refinement. */
    attributeID: string

    /** True when this definition represents an attribute refinement. */
    attributeRefinement: boolean

    /** Cut-off threshold for the refinement. */
    cutoffThreshold: number

    /** Display name for the refinement. */
    displayName: string

    /** Value type code (see ObjectAttributeDefinition constants). */
    valueTypeCode: number

    /** Returns the attribute ID. */
    getAttributeID(): string

    /** Returns the cut-off threshold. */
    getCutoffThreshold(): number

    /** Returns the display name. */
    getDisplayName(): string

    /** Returns the value type code. */
    getValueTypeCode(): number

    /** True if this is an attribute refinement. */
    isAttributeRefinement(): boolean
}
```
