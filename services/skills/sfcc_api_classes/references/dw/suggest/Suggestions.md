# dw.suggest.Suggestions

## Overview
Base class for suggestions containers with methods to access suggested items and search phrases.

## Description
For each type of items, a subclass provides methods to access the actual items.

## All Known Subclasses
BrandSuggestions, CategorySuggestions, ContentSuggestions, CustomSuggestions, ProductSuggestions

```ts
declare class Suggestions  {
	/**
	 * The suggested search phrases associated with these suggestions, containing corrected and completed versions of the original search phrase.
	 * @readonly
	 */
	searchPhraseSuggestions: SearchPhraseSuggestions

	/**
	 * A list of SuggestedPhrase objects related to the user input search phrase.
	 * @deprecated Use getSearchPhraseSuggestions() instead
	 * @readonly
	 */
	suggestedPhrases: Iterator<SuggestedPhrase>

	/**
	 * A list of SuggestedTerms objects, each representing suggested terms for a particular term in the user input search phrase.
	 * @deprecated Use getSearchPhraseSuggestions() instead
	 * @readonly
	 */
	suggestedTerms: Iterator<SuggestedTerms>


	/**
	 * Returns the suggested search phrases associated with these suggestions.
	 */
	getSearchPhraseSuggestions(): SearchPhraseSuggestions

	/**
	 * Returns a list of SuggestedPhrase objects related to the user input search phrase.
	 * @deprecated Use getSearchPhraseSuggestions() instead
	 */
	getSuggestedPhrases(): Iterator<SuggestedPhrase>

	/**
	 * Returns a list of SuggestedTerms objects, each representing suggested terms for a particular term in the user input search phrase.
	 * @deprecated Use getSearchPhraseSuggestions() instead
	 */
	getSuggestedTerms(): Iterator<SuggestedTerms>

	/**
	 * Returns whether this suggestions container has any suggested phrases.
	 * @deprecated Use getSearchPhraseSuggestions() instead
	 */
	hasSuggestedPhrases(): boolean

	/**
	 * Returns whether this suggestions container has any suggested terms.
	 * @deprecated Use getSearchPhraseSuggestions() instead
	 */
	hasSuggestedTerms(): boolean

	/**
	 * Returns whether this suggestions container has any suggested items (e.g., products or categories).
	 */
	hasSuggestions(): boolean
}
```
