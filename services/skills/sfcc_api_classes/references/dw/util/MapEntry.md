# dw.util.MapEntry

## Overview
Represents a single key-value entry within a Map.

## Description
The class represents an entry within a Map, providing access to the entry's key and value.

```ts
declare class MapEntry  {
	/**
	 * The entry's key
	 */
	readonly key: Object

	/**
	 * The entry's value
	 */
	readonly value: Object

	/**
	 * Returns the entry's key
	 */
	getKey(): Object

	/**
	 * Returns the entry's value
	 */
	getValue(): Object
}
```
