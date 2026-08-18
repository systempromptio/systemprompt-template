# dw.web.URLParameter

## Overview
Represents a key-value-pair for URL parameters.

## Description
Encapsulates URL parameter data as name-value pairs with optional encoding control for URL construction.

```ts
declare class URLParameter {
	/**
	 * Constructs the parameter using the specified name and value and encoded in the form "name=value".
	 * @param aName - The name
	 * @param aValue - The value
	 */
	constructor(aName: string, aValue: string)

	/**
	 * Constructs the parameter using the specified name and value. If the "encodeName" is set to true, the parameter is encoded in the form "name=value". Otherwise, it only contains the "value" (needed for URL patterns).
	 * @param aName - The name
	 * @param aValue - The value
	 * @param encodeName - If true, the name will be part of the string form
	 */
	constructor(aName: string, aValue: string, encodeName: boolean)
}
```
