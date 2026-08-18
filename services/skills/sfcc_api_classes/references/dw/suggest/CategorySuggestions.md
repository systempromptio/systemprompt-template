# dw.suggest.CategorySuggestions

## Overview
Container for category suggestions found using suggested search terms as criteria, executed in the current catalog and locale.

## Description
The category suggestion container provides access to categories found using the suggested terms as search criteria. The category lookup is executed in the current catalog and locale. Furthermore, the list of suggested terms (after processing the original user input search query) is accessible through the inherited methods.

```ts
declare class CategorySuggestions extends Suggestions {
  /**
   * List of categories found using suggested terms as search criteria. Category lookup executed in current catalog and locale.
   */
  readonly suggestedCategories: Iterator

  /**
   * Returns list of categories found using suggested terms as search criteria. Category lookup executed in current catalog and locale.
   * @returns Iterator containing SuggestedCategory instance for each found category, may be empty
   */
  getSuggestedCategories(): Iterator
}
```
