# dw.suggest.SuggestedPhrase

## Overview
Represents a suggested search phrase based on user input.

## Description
Provides access to a suggested phrase and indicates whether it is an exact match. Use getPhrase() to retrieve the phrase as a string.

```ts
declare class SuggestedPhrase  {
	/**
	 * Flag signaling whether this phrase is an exact match.
	 * @readonly
	 */
	readonly exactMatch: boolean

	/**
	 * The actual phrase as a string value.
	 * @readonly
	 */
	readonly phrase: string

	/**
	 * Returns the actual phrase as a string value.
	 * @returns The phrase
	 */
	getPhrase(): string

	/**
	 * Returns a flag signaling whether this phrase is an exact match.
	 * @returns True if this phrase is an exact match, false otherwise
	 */
	isExactMatch(): boolean
}
```
