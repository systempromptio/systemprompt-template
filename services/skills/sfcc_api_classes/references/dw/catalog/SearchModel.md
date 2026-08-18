# dw.catalog.SearchModel

## Overview
Utility for building and executing storefront search queries, managing refinements and generating URLs for search actions.

## Description
Provides methods to add/remove refinements and sorting, run the search, and build URLs that reproduce or modify the current query. Commonly used by storefront controllers and templates to produce refinement and sorting links.

```ts
declare class SearchModel  {
    /** Execute the search and return a SearchStatus. */
    search(): SearchStatus

    /** Set the search phrase used in this query. */
    setSearchPhrase(phrase: string): void

    /** Set refinement values for an attribute (values may be pipe-delimited). Existing values replaced. */
    setRefinementValues(attributeID: string, values: string): void

    /** Remove refinement values for an attribute; pass null to remove all. */
    removeRefinementValues(attributeID: string, values: string): void

    /** Set a numeric/string range for an attribute refinement (existing values removed). */
    setRefinementValueRange(attributeID: string, minValue: string, maxValue: string): void

    /** Set or remove a sorting condition for the specified attribute (use direction constants). */
    setSortingCondition(attributeID: string, direction: number): void

    /** Builds a URL for the current query for an action name. */
    url(action: string): URL

    /** Builds a URL for the current query using an existing URL object. */
    url(url: URL): URL

    /** Builds a URL that re-executes the query with default sorting. */
    urlDefaultSort(urlOrAction: string|URL): URL

    /** Build a URL that adds a refinement for an attribute/value. */
    urlRefineAttribute(actionOrUrl: string|URL, attributeID: string, value: string): URL

    /** Build a URL that adds a refinement value for an attribute (broadens results). */
    urlRefineAttributeValue(actionOrUrl: string|URL, attributeID: string, value: string): URL

    /** Build a URL that adds/replaces a refinement value range for an attribute. */
    urlRefineAttributeValueRange(actionOrUrl: string|URL, attributeID: string, minValue: string, maxValue: string): URL

    /** Build a URL that removes a refinement for an attribute. */
    urlRelaxAttribute(actionOrUrl: string|URL, attributeID: string): URL

    /** Build a URL that removes a refinement value for an attribute. */
    urlRelaxAttributeValue(actionOrUrl: string|URL, attributeID: string, value: string): URL

    /** Build a URL that applies a specific sort by name and direction. */
    urlSort(actionOrUrl: string|URL, sortBy: string, sortDir: number): URL
}
```
