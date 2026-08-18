# dw.catalog.SortingOption

## Overview
Represents an option for how to sort products in storefront search results, with description and display name properties.

## Description
Represents an option for how to sort products in storefront search results. Each sorting option has an ID, display name, description, and optional associated sorting rule.

```ts
declare class SortingOption extends PersistentObject {
	/**
	 * Description of the sorting option for the current locale
	 */
	readonly description: string

	/**
	 * Display name of the sorting option for the current locale
	 */
	readonly displayName: string

	/**
	 * ID of the sorting option
	 */
	readonly ID: string

	/**
	 * Sorting rule for this sorting option, or null if there is no associated rule
	 */
	readonly sortingRule: SortingRule | null

	/**
	 * Returns the description of the sorting option for the current locale.
	 */
	getDescription(): string

	/**
	 * Returns the display name of the sorting option for the current locale.
	 */
	getDisplayName(): string

	/**
	 * Returns the ID of the sorting option.
	 */
	getID(): string

	/**
	 * Returns the sorting rule for this sorting option, or null if there is no associated rule.
	 */
	getSortingRule(): SortingRule | null
}
```
