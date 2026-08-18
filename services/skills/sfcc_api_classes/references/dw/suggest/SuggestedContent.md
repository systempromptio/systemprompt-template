# dw.suggest.SuggestedContent

## Overview
Represents a suggested content page based on search input.

## Description
Provides access to a suggested content page. Use getContent() to retrieve the actual Content object.

```ts
declare class SuggestedContent  {
	/**
	 * The actual Content object corresponding to this suggested content.
	 * @readonly
	 */
	readonly content: Content

	/**
	 * Returns the actual Content object corresponding to this suggested content.
	 * @returns The content object
	 */
	getContent(): Content
}
```
