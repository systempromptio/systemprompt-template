# dw.content.ContentMgr

## Overview
Manager for retrieving content assets, folders and libraries in the content system.

## Description
Provides static helpers to fetch Content and Folder objects from libraries and the site-specific
library helper for convenience.

```ts
declare class ContentMgr {
    /** ID for the private library constant. */
    static PRIVATE_LIBRARY: string

    /** Returns the site library identifier. */
    siteLibrary: string

    /** Returns a Content by ID from the default library. */
    static getContent(id: string): Content

    /** Returns a Content by library and ID. */
    static getContent(library: string, id: string): Content

    /** Returns a Folder by ID from default library. */
    static getFolder(id: string): Folder

    /** Returns a Folder by library and ID. */
    static getFolder(library: string, id: string): Folder

    /** Returns a Library object for a library ID. */
    static getLibrary(libraryId: string): Library

    /** Returns the current site library. */
    static getSiteLibrary(): string
}
```
