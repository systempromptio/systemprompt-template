# dw.web.FormFieldOption

## Overview
Represents an option for a form field.

## Description
Represents an option for a form field. Each option has a value, label, HTML representation, and selection state.

```ts
declare class FormFieldOption  {
	/**
	 * Indicates if this option is checked.
	 */
	readonly checked: boolean

	/**
	 * Value for HTML value attribute of HTML option element.
	 */
	readonly htmlValue: string

	/**
	 * Value for HTML label attribute of HTML option element. If not specified in form option definition, label is identical with string representation of option value.
	 */
	label: string

	/**
	 * Object bound to this option value.
	 */
	readonly object: Object

	/**
	 * ID of the option. Internal ID used to uniquely reference this option. If not specified in form option definition, ID is identical with string representation of option value.
	 */
	readonly optionId: string

	/**
	 * Parent field element.
	 */
	readonly parent: FormField

	/**
	 * Indicates if this option is selected.
	 */
	readonly selected: boolean

	/**
	 * Actual value associated with this option. This value is formatted and returned as HTML value with getHtmlValue().
	 */
	readonly value: Object

	/**
	 * Returns value for HTML value attribute of HTML option element.
	 */
	getHtmlValue(): string

	/**
	 * Returns value for HTML label attribute of HTML option element. If not specified, returns string representation of option value.
	 */
	getLabel(): string

	/**
	 * Returns object bound to this option value.
	 */
	getObject(): Object

	/**
	 * Returns ID of the option. Internal ID used to uniquely reference this option.
	 */
	getOptionId(): string

	/**
	 * Returns parent field element.
	 */
	getParent(): FormField

	/**
	 * Returns actual value associated with this option.
	 */
	getValue(): Object

	/**
	 * Indicates if this option is checked.
	 */
	isChecked(): boolean

	/**
	 * Indicates if this option is selected.
	 */
	isSelected(): boolean

	/**
	 * Sets label attribute for this option.
	 * @param label - Label to set
	 */
	setLabel(label: string): void
}
```
