 # dw.content.MarkupText

 ## Overview
 Represents HTML content snippets with special Commerce Cloud link functions; rewrites links for storefront use.

 ## Description
 Supports special link functions (`$url`, `$httpUrl`, `$httpsUrl`, `$include`, `$staticlink$`) that are rewritten to storefront-ready URLs. Commonly used for product descriptions and content snippets.

 ```ts
 declare class MarkupText  {
     /** The content with all links rewritten for storefront use. */
     markup: string

     /** Returns rewritten markup. */
     getMarkup(): string
 }
 ```
