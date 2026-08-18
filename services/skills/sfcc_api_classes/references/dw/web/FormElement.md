# dw.web.FormElement

## Overview
Base class for form elements in SFCC forms framework. Provides validation, HTML name generation, and hierarchy management for form components.

## Description
Represents a form element. Form elements are hierarchical and can be validated, cleared, and bound to business objects. Supports dynamic HTML name generation for browser autocompletion suppression.

## All Known Subclasses
Form, FormAction, FormField, FormGroup, FormList, FormListItem

```ts
declare class FormElement  {
	/**
	 * Dynamic HTML name for the field. Suppresses browser autocompletion for sensitive fields like credit cards. Also useful for unique form names when one form appears multiple times on a page.
	 */
	readonly dynamicHtmlName: string

	/**
	 * ID of the form element. Unique within the parent element.
	 */
	readonly formId: string

	/**
	 * Global unique name of the field, usable as name in HTML form. Not unique for radio buttons.
	 */
	readonly htmlName: string

	/**
	 * Parent element within the form.
	 */
	readonly parent: FormElement

	/**
	 * Indicates if this element and all its children are valid. Unsubmitted form elements are always valid.
	 */
	readonly valid: boolean

	/**
	 * Combined view of validation status (isValid() and getError()). Includes data returned by validation script if used.
	 */
	readonly validationResult: FormElementValidationResult

	/**
	 * Clears the form. After clearing, form contains no value or default value, is not bound to business object, and has valid status.
	 */
	clearFormElement(): void

	/**
	 * Returns dynamic HTML name for the field. Suppresses browser autocompletion for sensitive fields or creates unique form names.
	 */
	getDynamicHtmlName(): string

	/**
	 * Returns ID of the form element. Unique within the parent element.
	 */
	getFormId(): string

	/**
	 * Returns global unique name of the field, usable as name in HTML form. Not unique for radio buttons.
	 */
	getHtmlName(): string

	/**
	 * Returns parent element within the form.
	 */
	getParent(): FormElement

	/**
	 * Returns combined view of validation status (isValid() and getError()). Includes data from validation script if used.
	 */
	getValidationResult(): FormElementValidationResult

	/**
	 * Explicitly invalidates form element. Error text set to preconfigured custom error: "value-error" for FormField, "form-error" for FormGroup.
	 */
	invalidateFormElement(): void

	/**
	 * Explicitly invalidates field with custom error message.
	 * @param error - Error text to use
	 */
	invalidateFormElement(error: string): void

	/**
	 * Indicates if this element and all its children are valid. Unsubmitted form elements are always valid.
	 */
	isValid(): boolean
}
```
