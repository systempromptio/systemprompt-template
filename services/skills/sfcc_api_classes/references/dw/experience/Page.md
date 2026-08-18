# dw.experience.Page

## Overview
Represents a Page Designer managed page composed of regions and components; supports rendering and serialization.

## Description
A page contains regions that hold components and can be rendered via `PageMgr.renderPage(...)` or serialized via `PageMgr.serializePage(...)`. Exposes metadata like SEO title, description, folders, and helper methods to access regions and attributes.

```ts
declare class Page  {
	/** Aspect type ID if this page is dynamic. */
	aspectTypeID: string

	/** Classification folder assigned to the page. */
	classificationFolder: Folder

	/** Page description. */
	description: string

	/** Collection of folders this page belongs to. */
	folders: Collection

	/** Page ID. */
	ID: string

	/** Page name. */
	name: string

	/** SEO description. */
	pageDescription: string

	/** SEO keywords. */
	pageKeywords: string

	/** SEO title. */
	pageTitle: string

	/** Search words for indexing. */
	searchWords: string

	/** Page type ID. */
	typeID: string

	/** Visibility flag computed by rules (time, groups, etc.). */
	visible: boolean

	getAspectTypeID(): string
	getAttribute(attributeID: string): Object
	getClassificationFolder(): Folder
	getDescription(): string
	getFolders(): Collection
	getID(): string
	getName(): string
	getPageDescription(): string
	getPageKeywords(): string
	getPageTitle(): string
	getRegion(id: string): Region
	getSearchWords(): string
	getTypeID(): string
	hasVisibilityRules(): boolean
	isVisible(): boolean
}
```
