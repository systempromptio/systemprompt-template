# dw.catalog.SearchRefinements

## Overview
Container of refinement definitions and values for a search result; used to enumerate available refinements and values.

## Description
Provides collections of refinement definitions appropriate for the current search result, and methods to retrieve refinement values (with optional sorting). Subclasses include product- and content-specific implementations.

All Known Subclasses
- ContentSearchRefinements
- ProductSearchRefinements

```ts
declare class SearchRefinements  {
    /** Sorted collection of all refinement definitions (including those without values). */
    allRefinementDefinitions: Collection

    /** Sorted collection of refinement definitions filtered to those that have values for the result. */
    refinementDefinitions: Collection

    /** Returns a sorted list of refinement definitions for the deepest common category. */
    getAllRefinementDefinitions(): Collection

    /** Returns refinement values for an attribute (sorted). */
    getAllRefinementValues(attributeName: string, sortMode?: number, sortDirection?: number): Collection

    /** Returns refinement definitions filtered to those with values for the result. */
    getRefinementDefinitions(): Collection

    /** Returns refinement values for an attribute with sort options. */
    getRefinementValues(attributeName: string, sortMode: number, sortDirection: number): Collection
}
```
