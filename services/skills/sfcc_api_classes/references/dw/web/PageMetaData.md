# dw.web.PageMetaData

## Overview
Contains meta data about the page including title, description, keywords, and custom meta tags.

## Description
Contains meta data about the page.

For each request an instance of this class will be placed in the pipeline dictionary under the key "CurrentPageMetaData". The information stored in CurrentPageMetaData can be referenced in templates and rendered in an HTML head section.

To update the CurrentPageMetaData there is the pipelet UpdatePageMetaData provided.

```
Object
  dw.web.PageMetaData
```

```ts
declare class PageMetaData  {
	/**
	 * The page's description.
	 */
	description: string

	/**
	 * The page's key words.
	 */
	keywords: string

	/**
	 * All page meta tags added to this container.
	 */
	readonly pageMetaTags: Array

	/**
	 * The page's title.
	 */
	title: string

	/**
	 * Adds a page meta tag to this container.
	 * @param pageMetaTag - The page meta tag to be added
	 */
	addPageMetaTag(pageMetaTag: PageMetaTag): void

	/**
	 * Adds a page meta tags list to this container.
	 * @param pageMetaTags - The page meta tags list to be added
	 */
	addPageMetaTags(pageMetaTags: Array): void

	/**
	 * Returns the page's description.
	 * @returns The page's description
	 */
	getDescription(): string

	/**
	 * Returns the page's key words.
	 * @returns The page's key words
	 */
	getKeywords(): string

	/**
	 * Returns all page meta tags added to this container.
	 * @returns Array of page meta tags
	 */
	getPageMetaTags(): Array

	/**
	 * Returns the page's title.
	 * @returns The page's title
	 */
	getTitle(): string

	/**
	 * Returns true if a page meta tag with the given ID is set, false otherwise.
	 * @param id - The ID to check
	 * @returns True if set, false otherwise
	 */
	isPageMetaTagSet(id: string): boolean

	/**
	 * Sets the page's description.
	 * @param description - The description to set
	 */
	setDescription(description: string): void

	/**
	 * Sets the page's key words.
	 * @param keywords - The keywords to set
	 */
	setKeywords(keywords: string): void

	/**
	 * Sets the page's title.
	 * @param title - The title to set
	 */
	setTitle(title: string): void
}
```
