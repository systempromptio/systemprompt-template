# dw.suggest.SuggestedTerm

## Overview
Represents a single suggested search term, including completions, corrections, or exact matches.

## Description
Each user input term is processed separately by the suggestion engine. For each original term, a list of terms is suggested: completed terms, corrected terms, or exact matches. Each suggested term is represented by this class, while the list of suggested terms for a single original term is represented by SuggestedTerms.

```ts
declare class SuggestedTerm  {
	/**
	 * Returns whether this suggested term is an additional term with no corresponding term in the original search phrase.
	 * @readonly
	 */
	additional: boolean

	/**
	 * Returns whether this suggested term is an auto-completed version of the original term (original term is a prefix).
	 * @readonly
	 */
	completed: boolean

	/**
	 * Returns whether this suggested term is a corrected version of the original term.
	 * @readonly
	 */
	corrected: boolean

	/**
	 * Returns whether this suggested term exactly matches the original term.
	 * @readonly
	 */
	exactMatch: boolean

	/**
	 * The string value of this suggested term.
	 * @readonly
	 */
	value: string


	/**
	 * Returns this suggested term as a string value.
	 */
	getValue(): string

	/**
	 * Returns whether this suggested term is an additional term with no corresponding term in the original search phrase.
	 */
	isAdditional(): boolean

	/**
	 * Returns whether this suggested term is an auto-completed version of the original term.
	 */
	isCompleted(): boolean

	/**
	 * Returns whether this suggested term is a corrected version of the original term.
	 */
	isCorrected(): boolean

	/**
	 * Returns whether this suggested term exactly matches the original term.
	 */
	isExactMatch(): boolean
}
```
