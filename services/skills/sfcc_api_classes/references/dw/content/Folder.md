 # dw.content.Folder

 ## Overview
 Represents a folder that organizes content assets in Commerce Cloud Digital.

 ## Description
 Provides access to folder metadata, children, and contained content. Commonly returned by library and content APIs.

 ```ts
 declare class Folder extends ExtensibleObject {
     /** The content objects for this folder, sorted by position. */
     content: Collection

     /** The description for the folder in the current locale or null. */
     description: string

     /** The display name for the folder in the current locale or null. */
     displayName: string

     /** The ID of the folder (unique within a library). */
     ID: string

     /** Indicates if the folder is set online. */
     online: boolean

     /** The online content objects for this folder, sorted by position. */
     onlineContent: Collection

     /** The online subfolders of this folder, sorted by position. */
     onlineSubFolders: Collection

     /** Page description for the folder in the current locale, or null. */
     pageDescription: string

     /** Page keywords for the folder in the current locale, or null. */
     pageKeywords: string

     /** Returns the site's root folder or parent folder as appropriate. */
     getParent(): Folder

     /** Returns the root folder for this folder's library. */
     getRoot(): Folder

     /** Returns child folders collection. */
     getSubFolders(): Collection

     /** Returns the collection of content assets in this folder. */
     getContent(): Collection
 }
 ```
