# dw.util.Map

## Overview
Key-value object storage with direct access via keys. Supports checking containment by key or value.

## Description
Represents a Map of objects with key-value associations. Provides operations to add, retrieve, remove entries, and query map contents.

## All Known Subclasses
HashMap, LinkedHashMap, SortedMap

```ts
declare class Map  {
	/**
	 * Identifies if this map is empty
	 */
	readonly empty: boolean

	/**
	 * Convenience variable for an empty and immutable map
	 */
	static EMPTY_MAP: Map

	/**
	 * The size of the map
	 * Bean attribute method supporting array-like access (e.g., 'products.length')
	 */
	readonly length: Number

	/**
	 * Clears the map of all objects
	 */
	clear(): void

	/**
	 * Identifies if this map contains an element identified by the specified key
	 */
	containsKey(key: Object): boolean

	/**
	 * Identifies if this map contains an element identified by the specified value
	 */
	containsValue(value: Object): boolean

	/**
	 * Returns a set of the map's entries
	 * The returned set is a view to the entries of this map
	 */
	entrySet(): Set

	/**
	 * Returns the object associated with the key or null
	 */
	get(key: Object): Object

	/**
	 * Returns the size of the map
	 * Bean attribute method supporting array-like access
	 */
	getLength(): Number

	/**
	 * Identifies if this map is empty
	 */
	isEmpty(): boolean

	/**
	 * Returns a set of the map's keys
	 * The returned set is a view to the keys of this map
	 */
	keySet(): Set

	/**
	 * Puts the specified value into the map using the specified key to identify it
	 */
	put(key: Object, value: Object): Object

	/**
	 * Copies all of the objects inside the specified map into this map
	 */
	putAll(other: Map): void

	/**
	 * Removes the object from the map that is identified by the key
	 */
	remove(key: Object): Object

	/**
	 * Returns the size of the map
	 */
	size(): Number

	/**
	 * Returns a collection of the values contained in this map
	 * API Versioned: From version 16.1, returns a view on values like keySet() and entrySet()
	 * Before 16.1, returned an independent modifiable collection
	 */
	values(): Collection
}
```
