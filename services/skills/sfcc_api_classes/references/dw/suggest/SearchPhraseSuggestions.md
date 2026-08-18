# dw.suggest.SearchPhraseSuggestions

## Overview
Contains suggested search phrases and alternative terms for each search phrase term based on user input.

## Description
Provides a list of suggested search phrases and, for each term in the user input search phrase, corrected and completed alternative terms.

```ts
declare class SearchPhraseSuggestions  {
	/**
	 * List of SuggestedPhrase objects that relates to the user input search phrase.
	 * @readonly
	 */
	readonly suggestedPhrases: Iterator

	/**
	 * List of SuggestedTerms objects. Each instance represents a set of terms suggested for a particular single term of the user input search phrase.
	 * @readonly
	 */
	readonly suggestedTerms: Iterator

	/**
	 * Returns a list of SuggestedPhrase objects that relates to the user input search phrase.
	 * @returns List of SuggestedPhrases
	 */
	getSuggestedPhrases(): Iterator

	/**
	 * Returns a list of SuggestedTerms objects. Each instance represents a set of terms suggested for a particular single term of the user input search phrase.
	 * @returns List of SuggestedTerms for each term of the user input search phrase
	 */
	getSuggestedTerms(): Iterator

	/**
	 * Returns whether this suggestions container has any suggested phrases. Does not account for suggested terms.
	 * @returns True if there are phrases available
	 */
	hasSuggestedPhrases(): boolean

	/**
	 * Returns whether this suggestions container has any suggested terms. Does not account for suggested phrases.
	 * @returns True if there are terms available
	 */
	hasSuggestedTerms(): boolean
}
```
