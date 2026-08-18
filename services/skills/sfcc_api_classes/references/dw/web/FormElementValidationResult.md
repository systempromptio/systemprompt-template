# dw.web.FormElementValidationResult

## Overview
Encapsulates form validation results with validity state, optional error message, and custom data from validation scripts.

## Description
Represents a form element validation result. Validation scripts for form groups and fields create this object with validity state, message, and optional data. Server-side validation evaluates these settings to calculate element validity and message. Custom data persists and can be accessed from the form element after validation.

```ts
declare class FormElementValidationResult  {
	/**
	 * Optional data acquired during validation.
	 */
	readonly data: Map

	/**
	 * Optional message for validation failure.
	 */
	message: string

	/**
	 * Indicates if validation succeeded or failed.
	 */
	valid: boolean

	/**
	 * Creates validation result with validity state but no message.
	 * @param valid - Desired validity state
	 */
	constructor(valid: boolean)

	/**
	 * Creates validation result with validity state and message. Useful for failed validation with error message.
	 * @param valid - Desired validity state
	 * @param message - Desired message
	 */
	constructor(valid: boolean, message: string)

	/**
	 * Creates validation result with validity state, message, and custom data.
	 * @param valid - Desired validity state
	 * @param message - Desired message
	 * @param data - Desired custom data
	 */
	constructor(valid: boolean, message: string, data: Map)

	/**
	 * Adds custom data acquired during validation.
	 * @param key - Key for data value
	 * @param value - Data value for given key
	 */
	addData(key: Object, value: Object): void

	/**
	 * Returns custom data acquired during validation.
	 */
	getData(): Map

	/**
	 * Returns optional message for validation failure.
	 */
	getMessage(): string

	/**
	 * Indicates if validation succeeded.
	 */
	isValid(): boolean

	/**
	 * Sets optional message for validation failure.
	 * @param message - Message for validation failure
	 */
	setMessage(message: string): void

	/**
	 * Sets validation success or failure state.
	 * @param valid - True if validation succeeded
	 */
	setValid(valid: boolean): void
}
```
