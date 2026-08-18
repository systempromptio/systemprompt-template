# dw.suggest.BrandSuggestions

## Overview
Container for brand suggestions found using suggested search terms, executed in the current catalog and locale.

## Description
The brands suggestion container provides access to brands found using the suggested terms. The brand lookup is executed in the current catalog and locale. Furthermore, the list of suggested terms (after processing the original user input search query) is accessible through the inherited methods.

```ts
declare class BrandSuggestions extends Suggestions {
  // No additional properties or methods beyond those inherited from Suggestions
}
```
