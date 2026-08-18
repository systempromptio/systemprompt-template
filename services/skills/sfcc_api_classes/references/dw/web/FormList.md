# dw.web.FormList

## Overview
Represents a list of forms with support for item selection.

## Description
Represents a list of forms.

```
Object
  dw.web.FormElement
    dw.web.FormGroup
      dw.web.FormList
```

```ts
declare class FormList extends FormGroup {
	/**
	 * The selected list items if the list is configured to support selection of items.
	 */
	readonly selectManyItems: List

	/**
	 * A list of all selected objects if the list is configured to support the selection of items. The objects are the objects that were bound to each row.
	 */
	readonly selectManyObjects: List

	/**
	 * The default list item if the list is configured to support the selection of a default item.
	 */
	readonly selectOneItem: FormListItem

	/**
	 * The selected object if the list is configured to support the selection of a default item. The object is the object bound to the item.
	 */
	readonly selectOneObject: Object

	/**
	 * Returns the selected list items if the list is configured to support selection of items.
	 * @returns A List of FormListItem elements or null if no selection was configured for the form
	 */
	getSelectManyItems(): List

	/**
	 * Returns a list of all selected objects if the list is configured to support the selection of items. The objects are the objects that were bound to each row.
	 * @returns A List of objects or null if no selection was configured for the form
	 */
	getSelectManyObjects(): List

	/**
	 * Returns the default list item if the list is configured to support the selection of a default item.
	 * @returns The default FormListItem elements or null if no selection was configured
	 */
	getSelectOneItem(): FormListItem

	/**
	 * Returns the selected object if the list is configured to support the selection of a default item. The object is the object bound to the item.
	 * @returns The selected object
	 */
	getSelectOneObject(): Object
}
```
