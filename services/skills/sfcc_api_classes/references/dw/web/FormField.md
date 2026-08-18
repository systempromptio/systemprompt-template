# dw.web.FormField

## Overview
Represents a field in a form with validation, type constraints, and option management.

## Description
Represents a field in a form. Supports multiple data types (boolean, string, int, number, date), validation rules, option lists for selection fields, and error messaging.

```ts
declare class FormField extends FormElement {
	/**
	 * Indicates if field is checked. For boolean fields, represents the boolean value. For string/int fields, returns true if current value matches "selected-value".
	 */
	readonly checked: boolean

	/**
	 * Optional description for the field.
	 */
	readonly description: string

	/**
	 * Error text shown when field is invalid. Returns null if valid or no error message defined. Error types: missing-error, parse-error, range-error, value-error.
	 */
	readonly error: string

	/**
	 * Indicates boolean/checkbox field in form definition.
	 */
	static FIELD_TYPE_BOOLEAN: number

	/**
	 * Indicates date field in form definition.
	 */
	static FIELD_TYPE_DATE: number

	/**
	 * Indicates integer field in form definition.
	 */
	static FIELD_TYPE_INTEGER: number

	/**
	 * Indicates number field in form definition.
	 */
	static FIELD_TYPE_NUMBER: number

	/**
	 * Indicates string field in form definition.
	 */
	static FIELD_TYPE_STRING: number

	/**
	 * Current external string representation of field value.
	 */
	htmlValue: string

	/**
	 * Optional label text for the field.
	 */
	readonly label: string

	/**
	 * Indicates if field is mandatory.
	 */
	readonly mandatory: boolean

	/**
	 * Maximum length for field. Applicable to all types but only validates string fields. Default is Integer.MAX_VALUE.
	 */
	readonly maxLength: number

	/**
	 * Maximum value for field. Only applicable for int, number, and date types. Null if not specified.
	 */
	readonly maxValue: Object

	/**
	 * Minimum length for field. Applicable to all types but only validates string fields. Default is 0.
	 */
	readonly minLength: number

	/**
	 * Minimum value for field. Only applicable for int, number, and date types. Null if not specified.
	 */
	readonly minValue: Object

	/**
	 * List of possible values for field. Used to render selection lists or radio buttons.
	 */
	options: FormFieldOptions

	/**
	 * Optional regex pattern from form definition. Only validates string fields. Null if not set.
	 */
	readonly regEx: string

	/**
	 * Indicates if field is selected. For boolean fields, represents the boolean value. For string/int fields, returns true if current value matches "selected-value".
	 */
	readonly selected: boolean

	/**
	 * Selected option or null if field has no options or none selected.
	 */
	readonly selectedOption: FormFieldOption

	/**
	 * Object optionally associated with currently selected option.
	 */
	readonly selectedOptionObject: Object

	/**
	 * Type of the field. One of FIELD_TYPE constants.
	 */
	readonly type: number

	/**
	 * Internal value representation: string, number, boolean, or date.
	 */
	value: Object

	/**
	 * Returns optional description for the field.
	 */
	getDescription(): string

	/**
	 * Returns error text shown when field is invalid. Error types: missing-error, parse-error, range-error, value-error. Returns null if valid or no error message defined.
	 */
	getError(): string

	/**
	 * Returns current external string representation of field value.
	 */
	getHtmlValue(): string

	/**
	 * Returns optional label text for the field.
	 */
	getLabel(): string

	/**
	 * Returns maximum length for field. Validates only string fields.
	 */
	getMaxLength(): number

	/**
	 * Returns maximum value for field. Only applicable for int, number, and date types.
	 */
	getMaxValue(): Object

	/**
	 * Returns minimum length for field. Validates only string fields.
	 */
	getMinLength(): number

	/**
	 * Returns minimum value for field. Only applicable for int, number, and date types.
	 */
	getMinValue(): Object

	/**
	 * Returns list of possible values for field. Used for selection lists or radio buttons.
	 */
	getOptions(): FormFieldOptions

	/**
	 * Returns optional regex pattern. Only validates string fields.
	 */
	getRegEx(): string

	/**
	 * Returns selected option or null if field has no options or none selected.
	 */
	getSelectedOption(): FormFieldOption

	/**
	 * Returns object optionally associated with currently selected option.
	 */
	getSelectedOptionObject(): Object

	/**
	 * Returns type of field. One of FIELD_TYPE constants.
	 */
	getType(): number

	/**
	 * Returns internal value representation: string, number, boolean, or date.
	 */
	getValue(): Object

	/**
	 * Indicates if field is checked. For boolean fields, represents the boolean value. For string/int fields, returns true if current value matches "selected-value".
	 */
	isChecked(): boolean

	/**
	 * Indicates if field is mandatory.
	 */
	isMandatory(): boolean

	/**
	 * Indicates if field is selected. For boolean fields, represents the boolean value. For string/int fields, returns true if current value matches "selected-value".
	 */
	isSelected(): boolean

	/**
	 * Sets HTML value. Form field has two representations: HTML value (external string) and plain value (typed).
	 * @param htmlValue - HTML value to set
	 */
	setHtmlValue(htmlValue: string): void

	/**
	 * Updates option list based on map keys and values.
	 * @param optionValues - Map of option keys and values
	 */
	setOptions(optionValues: Map): void

	/**
	 * Updates option list based on map keys and values with range limits.
	 * @param optionValues - Map of option keys and values
	 * @param begin - Start index
	 * @param end - End index
	 */
	setOptions(optionValues: Map, begin: number, end: number): void

	/**
	 * Updates option list based on iterator of objects with range limits.
	 * @param optionValues - Iterator of objects
	 * @param begin - Start index
	 * @param end - End index
	 */
	setOptions(optionValues: Iterator, begin: number, end: number): void

	/**
	 * Updates option list based on iterator of objects.
	 * @param optionValues - Iterator of objects
	 */
	setOptions(optionValues: Iterator): void

	/**
	 * Sets typed value of the field.
	 * @param value - Value to set
	 */
	setValue(value: Object): void
}
```
