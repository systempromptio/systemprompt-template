# dw.content.Content

## Overview
Represents a content asset in Commerce Cloud Digital with metadata, localization and page-related helpers.

## Description
Content exposes read-only attributes for identification, metadata, folder assignments and page-specific
properties; it provides accessors to convert to a Page representation where applicable.

```ts
declare class Content extends ExtensibleObject {
    /** Folder used for classification. */
    classificationFolder: Folder

    /** Localized description or null. */
    description: string

    /** All folders this content is assigned to. */
    folders: Collection

    /** Content asset ID. */
    ID: string

    /** Content asset name. */
    name: string

    /** Online flag. */
    online: boolean

    /** Online flag alias. */
    onlineFlag: boolean

    /** True if this content is a Page. */
    page: boolean

    /** Page description in current locale or null. */
    pageDescription: string

    /** Page keywords in current locale or null. */
    pageKeywords: string

    /** All page meta tags applicable to this content. */
    pageMetaTags: Array

    /** Page title in current locale or null. */
    pageTitle: string

    /** Page URL in current locale or null. */
    pageURL: string

    /** Searchable flag. */
    searchable: boolean

    /** Searchable flag alias. */
    searchableFlag: boolean

    /** Sitemap change frequency. */
    siteMapChangeFrequency: string

    /** Sitemap inclusion flag. */
    siteMapIncluded: number

    /** Sitemap priority. */
    siteMapPriority: number

    /** Template attribute value. */
    template: string

    /** Returns the classification Folder. */
    getClassificationFolder(): Folder

    /** Returns the localized description or null. */
    getDescription(): string

    /** Returns folders assigned to this content. */
    getFolders(): Collection

    /** Returns the asset ID. */
    getID(): string

    /** Returns the asset name. */
    getName(): string

    /** Returns the online flag. */
    getOnlineFlag(): boolean

    /** Returns the page description for current locale or null. */
    getPageDescription(): string

    /** Returns page keywords for current locale or null. */
    getPageKeywords(): string

    /** Returns a page meta tag by id. @param id */
    getPageMetaTag(id: string): PageMetaTag

    /** Returns all page meta tags. */
    getPageMetaTags(): Array

    /** Returns the page title for current locale or null. */
    getPageTitle(): string

    /** Returns the page URL for current locale or null. */
    getPageURL(): string

    /** Returns searchable flag. */
    getSearchableFlag(): boolean

    /** Returns sitemap change frequency. */
    getSiteMapChangeFrequency(): string

    /** Returns whether content is included in sitemap. */
    getSiteMapIncluded(): number

    /** Returns sitemap priority. */
    getSiteMapPriority(): number

    /** Returns template value. */
    getTemplate(): string

    /** Returns whether content is online. */
    isOnline(): boolean

    /** Returns whether content is a page. */
    isPage(): boolean

    /** Returns whether content is searchable. */
    isSearchable(): boolean

    /** Converts this Content to a Page if applicable. */
    toPage(): Page
}
```
