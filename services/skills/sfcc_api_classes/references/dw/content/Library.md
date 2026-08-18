 # dw.content.Library

 ## Overview
 Represents the content library for a site: collection of content assets and the folder hierarchy.

 ## Description
 Provides access to library metadata and root folder. Only one library exists per site; obtain via ContentMgr.getSiteLibrary().

 ```ts
 declare class Library extends ExtensibleObject {
     /** The CMS channel of the library. */
     CMSChannelID: string

     /** The display name for the library in the current locale. */
     displayName: string

     /** The ID of this library. */
     ID: string

     /** The root folder for this library. */
     root: Folder

     getCMSChannelID(): string
     getDisplayName(): string
     getID(): string
     getRoot(): Folder
 }
 ```
