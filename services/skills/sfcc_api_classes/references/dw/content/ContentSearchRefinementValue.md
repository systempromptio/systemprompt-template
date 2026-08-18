 # dw.content.ContentSearchRefinementValue

 ## Overview
 Represents the value of a content search refinement (attribute or folder bucket).

 ## Description
 Holds value information for a refinement option returned by content search. Most behavior is inherited from catalog SearchRefinementValue.

 ```ts
 declare class ContentSearchRefinementValue  {
     /** Inherited: description of the refinement value. */
     getDescription(): string

     /** Inherited: display value for UI. */
     getDisplayValue(): string

     /** Inherited: number of hits for this value. */
     getHitCount(): number

     /** Inherited: unique identifier for the refinement value. */
     getID(): string

     /** Inherited: presentation ID for sorting/labeling. */
     getPresentationID(): string

     /** Inherited: underlying value object. */
     getValue(): any
 }
 ```
