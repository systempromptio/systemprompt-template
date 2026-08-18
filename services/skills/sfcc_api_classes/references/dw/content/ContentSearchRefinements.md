 # dw.content.ContentSearchRefinements

 ## Overview
 Interface to refinement options for the content asset search. Provides folder- and attribute-based refinements used to limit or broaden search results.

 ## Description
 Exposes refinement definitions and values for content search results. Use it to obtain folder refinement definitions, matching folders, and sorted refinement values for attributes.

 ```ts
 declare class ContentSearchRefinements  {
     /** The appropriate folder refinement definition based on the search result. */
     folderRefinementDefinition: ContentSearchRefinementDefinition

     /** A collection of matching folders. */
     matchingFolders: Collection

     /** Returns a sorted collection of refinement values for the given definition. */
     getAllRefinementValues(definition: ContentSearchRefinementDefinition): Collection

     /** Returns the number of search hits for the passed folder object. */
     getFolderHits(folder: Folder): number

     /** Returns the appropriate folder refinement definition based on the search result. */
     getFolderRefinementDefinition(): ContentSearchRefinementDefinition

     /** Returns a collection of matching folders. */
     getMatchingFolders(): Collection

     /** Returns the next-level folder refinement values for the provided folder. */
     getNextLevelFolderRefinementValues(folder: Folder): Collection
 }
 ```
