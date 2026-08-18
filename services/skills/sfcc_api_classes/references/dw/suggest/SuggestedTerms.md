# dw.suggest.SuggestedTerms

## Overview
Container for a list of suggested terms belonging to a single original term from the user's search phrase.

## Description
Each user input term is processed separately by the suggestion engine. For each original term, a list of terms is suggested: completed, corrected, or exact matching terms. This class refers to the original unmodified term and contains a list of SuggestedTerm objects.

```ts
declare class SuggestedTerms  {
	/**
	 * Returns true if this set of suggested terms is empty.
	 * @readonly
	 */
	empty: boolean

	/**
	 * The suggested term considered best matching with the original term.
	 * @readonly
	 */
	firstTerm: SuggestedTerm

	/**
	 * The original term of the user input for which this instance provides suggested terms.
	 * @readonly
	 */
	originalTerm: string

	/**
	 * The list of SuggestedTerms for the original term.
	 * @readonly
	 */
	terms: Iterator<SuggestedTerm>


	/**
	 * Returns the suggested term considered best matching with the original term.
	 */
	getFirstTerm(): SuggestedTerm

	/**
	 * Returns the original term of the user input for which this instance provides suggested terms.
	 */
	getOriginalTerm(): string

	/**
	 * Returns the list of SuggestedTerms for the original term.
	 */
	getTerms(): Iterator<SuggestedTerm>

	/**
	 * Returns true if this set of suggested terms is empty.
	 */
	isEmpty(): boolean
}
```
