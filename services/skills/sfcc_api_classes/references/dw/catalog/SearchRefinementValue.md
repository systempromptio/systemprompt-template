# dw.catalog.SearchRefinementValue

## Overview
Represents a single refinement value for product or content search (value, display string, hit count, etc.).

## Description
Encapsulates the refinement value metadata used when rendering refinement lists: display text, underlying value, hit counts and optional presentation ID. Subclasses exist for product and content searches.

All Known Subclasses
- ContentSearchRefinementValue
- ProductSearchRefinementValue

```ts
declare class SearchRefinementValue  {
    /** Optional localized description for the value. */
    description: string

    /** Displayable value for UI. */
    displayValue: string

    /** Hit count for this refinement value. */
    hitCount: number

    /** Value ID (may be empty for price/category refinements). */
    ID: string

    /** Optional presentation ID for associating UI widgets. */
    presentationID: string

    /** Refinement raw value. */
    value: string

    /** Returns the optional localized description. */
    getDescription(): string

    /** Returns the display value in the current locale. */
    getDisplayValue(): string

    /** Returns the hit count for this value. */
    getHitCount(): number

    /** Returns the refinement value ID. */
    getID(): string

    /** Returns the optional presentation ID. */
    getPresentationID(): string

    /** Returns the refinement value. */
    getValue(): string
}
```
