 # dw.content.MediaFile

 ## Overview
 Represents references to media content (images, etc.) hosted in Commerce Cloud Digital or externally. Supports optional DIS image transformations.

 ## Description
 MediaFile exposes metadata and URLs for media objects. Many methods accept a `transform` parameter (object) to request image transformations when DIS is available.

 ```ts
 declare class MediaFile  {
     /** Returns the URL to the media file; supports optional transform parameter. */
     getURL(transform?: any): string

     /** Returns the file name or path. */
     getFileName(): string

     /** Returns the content type (mime). */
     getContentType(): string

     /** Returns the size in bytes. */
     getSize(): number
 }
 ```
