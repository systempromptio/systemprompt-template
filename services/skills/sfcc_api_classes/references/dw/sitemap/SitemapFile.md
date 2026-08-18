# dw.sitemap.SitemapFile

## Overview
Represents sitemap files in the appserver's shared file system with methods to access file details and validation status.

## Description
Instances of this class represent sitemap files located in the appservers shared file system. Methods are used to get details of a sitemap file, such as the hostname it is associated with.

```
Object
  dw.sitemap.SitemapFile
```

```ts
declare class SitemapFile  {
	/**
	 * The name of the file e.g. sitemap_index.xml
	 * @readonly
	 */
	readonly fileName: string

	/**
	 * The size of the file in bytes.
	 * @readonly
	 */
	readonly fileSize: number

	/**
	 * The URL used to access this file in a storefront request.
	 * @readonly
	 */
	readonly fileURL: string

	/**
	 * The host name this file is associated with.
	 * @readonly
	 */
	readonly hostName: string

	/**
	 * Checks if this instance of sitemap file is valid. Examples for invalid files are: file size > 10mb. Additional violations might be added later.
	 * @readonly
	 */
	readonly valid: boolean

	/**
	 * Returns the name of the file e.g. sitemap_index.xml
	 * @returns The file's name, never null.
	 */
	getFileName(): string

	/**
	 * Returns the size of the file in bytes.
	 * @returns The fileSize in bytes.
	 */
	getFileSize(): number

	/**
	 * Returns the URL used to access this file in a storefront request.
	 * @returns The fileURL, never null.
	 */
	getFileURL(): string

	/**
	 * Returns the host name this file is associated with.
	 * @returns The hostname, never null.
	 */
	getHostName(): string

	/**
	 * Checks if this instance of sitemap file is valid. Examples for invalid files are: file size > 10mb
	 * @returns True if the SitemapFile is valid, false otherwise.
	 */
	isValid(): boolean
}
```
