# dw.web.Form

## Overview
Top-level element in the form instance hierarchy, managing secure key generation and form validation.

## Description
The class is the top level element in the form instance hierarchy.

```ts
declare class Form extends FormGroup {
	/**
	 * The secure key html name to be used for the hidden input field that will contain the secure key value.
	 */
	readonly secureKeyHtmlName: string

	/**
	 * The secure key value that is generated for the form to use in a hidden input field for authentication.
	 */
	readonly secureKeyValue: string

	/**
	 * Returns the secure key html name to be used for the hidden input field that will contain the secure key value.
	 */
	getSecureKeyHtmlName(): string

	/**
	 * Returns the secure key value that is generated for the form to use in a hidden input field for authentication.
	 */
	getSecureKeyValue(): string
}
```
