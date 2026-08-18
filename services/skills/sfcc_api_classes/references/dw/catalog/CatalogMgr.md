# dw.catalog.CatalogMgr

## Overview
Provides static helper methods for accessing catalogs, categories, sorting options, and sorting rules for the current site.

## Description
CatalogMgr is a utility class for retrieving catalogs and categories by ID, as well as accessing sorting options and rules configured for the site. It is not instantiable and all methods are static.

```ts
declare class CatalogMgr {
	/**
	 * The catalog of the current site, or null if not assigned.
	 */
	static readonly siteCatalog: Catalog | null
	/**
	 * List of sorting options configured for this site.
	 */
	static readonly sortingOptions: List
	/**
	 * Collection of all sorting rules for this site, including global rules.
	 */
	static readonly sortingRules: Collection

	/**
	 * Returns the catalog with the specified ID, or null if not found.
	 * @param id Catalog ID
	 */
	static getCatalog(id: string): Catalog | null
	/**
	 * Returns the category of the site catalog with the specified ID, or null if not found.
	 * @param id Category ID
	 */
	static getCategory(id: string): Category | null
	/**
	 * Returns the catalog of the current site, or null if not assigned.
	 */
	static getSiteCatalog(): Catalog | null
	/**
	 * Returns the sorting option with the given ID, or null if not found.
	 * @param id Sorting option ID
	 */
	static getSortingOption(id: string): SortingOption | null
	/**
	 * Returns a list of sorting options configured for this site.
	 */
	static getSortingOptions(): List
	/**
	 * Returns the sorting rule with the given ID, or null if not found.
	 * @param id Sorting rule ID
	 */
	static getSortingRule(id: string): SortingRule | null
}
```
