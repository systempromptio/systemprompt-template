# dw.suggest.SuggestModel

## Overview
Model for accessing search suggestions with spell correction, auto-completion, and search-as-you-type functionality.

## Description
Provides methods to access search suggestions covering two functional areas: suggesting words based on user input using spell correction or prediction (auto-completion), and search-as-you-type where items are looked up before the user completes typing. Supports various suggestion types: products, categories, brands, content pages, and custom search phrases. The API always creates suggestions with autocorrections regardless of the "Search Autocorrections" preference.

```ts
declare class SuggestModel  {
	/**
	 * The maximum number of suggestions obtainable from this model.
	 */
	static MAX_SUGGESTIONS: 10

	/**
	 * BrandSuggestions container for the current search phrase.
	 * @readonly
	 */
	brandSuggestions: BrandSuggestions

	/**
	 * CategorySuggestions container for the current search phrase.
	 * @readonly
	 */
	categorySuggestions: CategorySuggestions

	/**
	 * ContentSuggestions container for the current search phrase.
	 * @readonly
	 */
	contentSuggestions: ContentSuggestions

	/**
	 * CustomSuggestions container for the current search phrase.
	 * @readonly
	 */
	customSuggestions: CustomSuggestions

	/**
	 * Returns true if search suggestions are filtered by folder (Page Designer content excluded as it belongs to no folder).
	 */
	filteredByFolder: boolean

	/**
	 * List of search phrases currently popular among all users (region, locale, and browser-specific).
	 * @readonly
	 */
	popularSearchPhrases: Iterator<string>

	/**
	 * ProductSuggestions container for the current search phrase.
	 * @readonly
	 */
	productSuggestions: ProductSuggestions

	/**
	 * List of personalized search phrases the current user entered recently (identified by CQuotient cookie).
	 * @readonly
	 */
	recentSearchPhrases: Iterator<string>


	constructor()

	/**
	 * Adds a refinement for product suggestions with name-value pair (multiple values delimited by '|').
	 */
	addRefinementValues(attributeID: string, values: string): void

	/**
	 * Returns BrandSuggestions container for the current search phrase.
	 */
	getBrandSuggestions(): BrandSuggestions

	/**
	 * Returns CategorySuggestions container for the current search phrase.
	 */
	getCategorySuggestions(): CategorySuggestions

	/**
	 * Returns ContentSuggestions container for the current search phrase.
	 */
	getContentSuggestions(): ContentSuggestions

	/**
	 * Returns CustomSuggestions container for the current search phrase.
	 */
	getCustomSuggestions(): CustomSuggestions

	/**
	 * Returns list of search phrases currently popular among all users.
	 */
	getPopularSearchPhrases(): Iterator<string>

	/**
	 * Returns ProductSuggestions container for the current search phrase.
	 */
	getProductSuggestions(): ProductSuggestions

	/**
	 * Returns list of personalized search phrases the current user entered recently.
	 */
	getRecentSearchPhrases(): Iterator<string>

	/**
	 * Returns true if search suggestions are filtered by folder.
	 */
	isFilteredByFolder(): boolean

	/**
	 * Removes previously added refinement values (multiple values delimited by '|', null removes all).
	 */
	removeRefinementValues(attributeID: string, values: string): void

	/**
	 * Apply category ID to filter product, brand, and category suggestions to specified category or subcategories.
	 */
	setCategoryID(categoryID: string): void

	/**
	 * Set flag to filter search suggestions by folder (false to include content assets not belonging to any folder).
	 */
	setFilteredByFolder(filteredByFolder: boolean): void

	/**
	 * Set maximum number of returned suggested items (max: MAX_SUGGESTIONS).
	 */
	setMaxSuggestions(maxSuggestions: number): void

	/**
	 * Set product suggestion refinement values for an attribute (multiple values delimited by '|', replaces existing).
	 */
	setRefinementValues(attributeID: string, values: string): void

	/**
	 * Set user input search phrase to be processed by auto-completion, spell correction, and enhancement.
	 */
	setSearchPhrase(searchPhrase: string): void
}
```
