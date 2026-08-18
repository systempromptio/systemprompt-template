# dw.suggest.ContentSuggestions

## Overview
Container for content page suggestions found using suggested search terms.

## Description
Provides access to content pages found using suggested terms as search criteria. Content lookup is executed in the current library and locale.

```ts
declare class ContentSuggestions extends Suggestions {
	/**
	 * List of suggested content pages (Read Only).
	 * @readonly
	 */
	readonly suggestedContent: Iterator

	/**
	 * Returns list of content pages found using suggested terms.
	 */
	getSuggestedContent(): Iterator
}
```
