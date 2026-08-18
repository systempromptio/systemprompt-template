# dw.web.FormListItem

## Overview
Represents an item in a form list.

## Description
Represents an item in a form list.

```
Object
  dw.web.FormElement
    dw.web.FormGroup
      dw.web.FormListItem
```

```ts
declare class FormListItem extends FormGroup {
	/**
	 * The index of this item within the list.
	 */
	readonly itemIndex: number

	/**
	 * Returns the index of this item within the list.
	 * @returns The index of this item within the list
	 */
	getItemIndex(): number
}
```
