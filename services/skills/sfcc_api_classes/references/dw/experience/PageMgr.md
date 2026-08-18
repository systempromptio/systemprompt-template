# dw.experience.PageMgr

## Overview
Utility class providing methods to retrieve, render and serialize Page Designer pages and to initialize custom editors.

## Description
Provides static helpers to obtain pages (by id, category, product), render pages or regions, serialize pages to JSON, and initialize `CustomEditor` instances.

```ts
declare class PageMgr  {
	/**
	 * Initialize a custom editor of given type id using the provided configuration.
	 * @param customEditorTypeID
	 * @param configuration
	 */
	static getCustomEditor(customEditorTypeID: string, configuration: Map): CustomEditor

	/** Returns the page matching the given id. */
	static getPage(pageID: string): Page

	/** Get dynamic page for a category (deprecated — use getPageByCategory). */
	static getPage(category: Category, pageMustBeVisible: boolean, aspectTypeID: string): Page

	/** Get dynamic page for a category (bottom-up traversal). */
	static getPageByCategory(category: Category, pageMustBeVisible: boolean, aspectTypeID: string): Page

	/** Get dynamic page for a product. */
	static getPageByProduct(product: Product, pageMustBeVisible: boolean, aspectTypeID: string): Page

	/** Render a page to markup. */
	static renderPage(pageID: string, parameters: string): string
	static renderPage(pageID: string, aspectAttributes: Map, parameters: string): string

	/** Render a region to markup. */
	static renderRegion(region: Region): string
	static renderRegion(region: Region, regionRenderSettings: RegionRenderSettings): string

	/** Serialize a page to JSON. */
	static serializePage(pageID: string, parameters: string): string
	static serializePage(pageID: string, aspectAttributes: Map, parameters: string): string
}
```
