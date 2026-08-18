# dw.web.FormFieldOptions

## Overview
List of options for a form field with index and property-based access.

## Description
Represents the list of options for a field. Supports index-style access to options (e.g., myfield.options[2]) and property-based access (e.g., myfield.options.red).

```ts
declare class FormFieldOptions  {
	/**
	 * Number of option values.
	 */
	readonly optionsCount: number

	/**
	 * Returns number of option values.
	 */
	getOptionsCount(): number
}
```
