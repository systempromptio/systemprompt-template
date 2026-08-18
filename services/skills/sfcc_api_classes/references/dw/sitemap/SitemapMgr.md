# dw.sitemap.SitemapMgr

## Overview
Exposes helpers to list, add, and delete the custom sitemap files that live on the appserver shared filesystem.

## Description
All operations work against the folder surfaced under Merchant Tools -> SEO -> Sitemaps -> Custom Sitemaps; changes are immediate on disk but must be published by running the same scheduled job that writes to the public sitemap.

```ts
declare class SitemapMgr  {
    /**
     * @readonly
     * Reads every custom sitemap file and groups them by hostname so callers can inspect or iterate the hosted lists.
     */
    readonly customSitemapFiles: Map

    /**
     * Copies the provided WebDAV file into the host's custom sitemap directory so the job that builds sitemaps can pick it up.
     */
    static addCustomSitemapFile(hostName: string, file: File): void

    /**
     * Removes the supplied SitemapFile from the appserver directory so it no longer contributes to the generated sitemap.
     */
    static deleteCustomSitemapFile(sitemapFile: SitemapFile): void

    /**
     * Deletes every custom sitemap file that was uploaded for the given hostname.
     */
    static deleteCustomSitemapFiles(hostName: string): void

    /**
     * Deletes all uploaded custom sitemap files for every hostname.
     */
    static deleteCustomSitemapFiles(): void

    /**
     * Loads the host -> sitemap list map that backs the `customSitemapFiles` property.
     */
    static getCustomSitemapFiles(): Map
}
```