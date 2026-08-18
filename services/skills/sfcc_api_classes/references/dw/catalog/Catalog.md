# dw.catalog.Catalog

## Overview
Represents a digital catalog, which is a container for products, categories, recommendations, and shared attributes. Each product belongs to one catalog, and each site has a single site catalog.

## Description
A Catalog organizes products and categories in a hierarchical structure. It supports product assignments, category trees, recommendations, and shared product options. Catalogs can be shared between sites, and each site has a designated site catalog that defines available products and category structure. Not directly instantiable.

## Inheritance
Object → PersistentObject → ExtensibleObject → Catalog

```ts
declare class Catalog extends ExtensibleObject {
	/**
	 * Localized short description for the current locale.
	 */
	readonly description: string | null
	/**
	 * Localized display name for the current locale.
	 */
	readonly displayName: string | null
	/**
	 * Catalog ID.
	 */
	readonly ID: string
	/**
	 * Root category of this catalog.
	 */
	readonly root: Category

	/**
	 * Returns the localized short description for the current locale.
	 */
	getDescription(): string | null
	/**
	 * Returns the localized display name for the current locale.
	 */
	getDisplayName(): string | null
	/**
	 * Returns the catalog ID.
	 */
	getID(): string
	/**
	 * Returns the root category of this catalog.
	 */
	getRoot(): Category
}
```
