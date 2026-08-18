# dw.content.ContentSearchModel

## Overview
Search model for building and executing content searches with refinements and pagination.

## Description
Encapsulates the search state for Content queries: filters, refinements, sort order, pagination and
result retrieval helpers.

```ts
declare class ContentSearchModel {
    /** Creates a new search model instance. */
    constructor()

    /** Returns the currently selected Content result. */
    getContent(): Content

    /** Returns the content ID used for search when set. */
    getContentID(): string

    /** Returns the deepest common folder across results. */
    getDeepestCommonFolder(): Folder

    /** Returns the collection of available refinements. */
    getRefinements(): Collection

    /** Returns whether the search is filtered by folder. */
    isFilteredByFolder(): boolean

    /** Executes the search using current model parameters. */
    search(): SearchResult

    /** Sets the content ID to filter the search. */
    setContentID(contentID: string): void

    /** Sets whether the model should be filtered by folder. */
    setFilteredByFolder(filtered: boolean): void
}
```
