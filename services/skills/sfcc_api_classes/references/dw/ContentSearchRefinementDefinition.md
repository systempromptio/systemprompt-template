# dw.content.ContentSearchRefinementDefinition

## Overview
Defines a refinement used by the ContentSearchModel, including folder-specific refinements.

## Description
Extends catalog search refinement with a folderRefinement property to indicate refinements that are
based on content folder structure.

```ts
declare class ContentSearchRefinementDefinition extends SearchRefinementDefinition {
    /** True when this refinement targets folders. */
    folderRefinement: boolean

    /** Returns whether this refinement is a folder refinement. */
    isFolderRefinement(): boolean
}
```
