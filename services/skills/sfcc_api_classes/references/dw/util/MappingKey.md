# dw.util.MappingKey

## Overview
Encapsulates single or compound keys for ImportKeyValueMapping job step.

## Description
Encapsulates the key for a mapping read in with the ImportKeyValueMapping job step. Can be either single keys (e.g., product id) or compound keys with multiple string components (e.g., product id and site).

```ts
declare class MappingKey  {
	/**
	 * Gets the (possibly compound) key
	 * If the key consists of only a single value, the array contains a single element
	 */
	readonly keyComponents: String[]

	/**
	 * Gets a key that contains only a single key component (not a compound key)
	 * Returns null if this is not a single component key
	 */
	readonly singleComponentKey: String

	/**
	 * Instantiates a new key using compound key components
	 * Accepts single string or multiple components for a compound key
	 */
	constructor(...keyComponents: String)

	/**
	 * Gets the (possibly compound) key
	 * If the key consists of only a single value, the array contains a single element
	 */
	getKeyComponents(): String[]

	/**
	 * Gets a key that contains only a single key component
	 * Returns null if this is not a single component key
	 */
	getSingleComponentKey(): String
}
```
