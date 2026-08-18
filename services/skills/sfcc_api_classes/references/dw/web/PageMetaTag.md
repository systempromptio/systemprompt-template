# dw.web.PageMetaTag

## Overview
Represents a page meta tag used in HTML documents to provide structured metadata.

## Description
Page meta tags are used in HTML documents to provide structured data about a web page. They are usually part of the head section. Common tags are for example robots, description or social tags like open graph (e.g. 'og:title').

Page meta tags can be obtained from various contexts (home page, detail page, listing page) and can be set at PageMetaData container object.

```
Object
  dw.web.PageMetaTag
```

```ts
declare class PageMetaTag  {
	/**
	 * The page meta tag content.
	 */
	readonly content: string

	/**
	 * The page meta tag ID.
	 */
	readonly ID: string

	/**
	 * Returns true if the page meta tag type is name, false otherwise.
	 */
	readonly name: boolean

	/**
	 * Returns true if the page meta tag type is property, false otherwise.
	 */
	readonly property: boolean

	/**
	 * Returns true if the page meta tag type is title, false otherwise.
	 */
	readonly title: boolean

	/**
	 * Returns the page meta tag content.
	 * @returns Page meta tag content
	 */
	getContent(): string

	/**
	 * Returns the page meta tag ID.
	 * @returns Page meta tag ID
	 */
	getID(): string

	/**
	 * Returns true if the page meta tag type is name, false otherwise.
	 * @returns True if type is name, false otherwise
	 */
	isName(): boolean

	/**
	 * Returns true if the page meta tag type is property, false otherwise.
	 * @returns True if type is property, false otherwise
	 */
	isProperty(): boolean

	/**
	 * Returns true if the page meta tag type is title, false otherwise.
	 * @returns True if type is title, false otherwise
	 */
	isTitle(): boolean
}
```

Hello there in future! =)
