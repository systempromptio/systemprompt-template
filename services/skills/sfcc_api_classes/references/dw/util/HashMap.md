````markdown
# dw.util.HashMap

## Overview
Represents a hash map of objects that extends the Map class, providing key-value storage with hash-based lookup.

## Description
Represents a hash map of objects.

```
Object
  dw.util.Map
    dw.util.HashMap
```

```ts
declare class HashMap extends Map {
	/**
	 * Identifies if this map is empty.
	 */
	readonly empty: boolean

	/**
	 * Convenience variable, for an empty and immutable list.
	 */
	static EMPTY_MAP: Map

	/**
	 * The size of the map. This is a bean attribute method and supports the access to the collections length similar to a ECMA array, such as 'products.length'.
	 */
	readonly length: Number

	/**
	 * Constructs a new HashMap.
	 */
	constructor()

	/**
	 * Clears the map of all objects.
	 */
	clear(): void

	/**
	 * Returns a shallow copy of this map.
	 * @returns a shallow copy of this map
	 */
	clone(): HashMap

	/**
	 * Identifies if this map contains an element identfied by the specified key.
	 * @param key - the key to use
	 * @returns true if this map contains an element whose key is equal to the specified key
	 */
	containsKey(key: Object): boolean

	/**
	 * Identifies if this map contains an element identfied by the specified value.
	 * @param value - the value to use
	 * @returns true if this map contains an element whose value is equal to the specified value
	 */
	containsValue(value: Object): boolean

	/**
	 * Returns a set of the map's entries. The returned set is actually a view to the entries of this map.
	 * @returns a set of the map's entries
	 */
	entrySet(): Set

	/**
	 * Returns the object associated with the key or null.
	 * @param key - the key to use
	 * @returns the object associated with the key or null
	 */
	get(key: Object): Object

	/**
	 * REturns the size of the map. This is a bean attribute method and supports the access to the collections length similar to a ECMA array, such as 'products.length'.
	 * @returns the number of objects in the map
	 */
	getLength(): Number

	/**
	 * Identifies if this map is empty.
	 * @returns true if the map is empty, false otherwise
	 */
	isEmpty(): boolean

	/**
	 * Returns a set of the map's keys. The returned set is actually a view to the keys of this map.
	 * @returns a set of the map's keys
	 */
	keySet(): Set

	/**
	 * Puts the specified value into the map using the specified key to identify it.
	 * @param key - the key to use to identify the value
	 * @param value - the object to put into the map
	 * @returns previous value associated with specified key, or null if there was no mapping for key
	 */
	put(key: Object, value: Object): Object

	/**
	 * Copies all of the objects inside the specified map into this map.
	 * @param other - the map whose contents are copied into this map
	 */
	putAll(other: Map): void

	/**
	 * Removes the object from the map that is identified by the key.
	 * @param key - the key that identifies the object to remove
	 * @returns the removed object or null
	 */
	remove(key: Object): Object

	/**
	 * Returns the size of the map.
	 * @returns the number of objects in the map
	 */
	size(): Number

	/**
	 * Returns a collection of the values contained in this map.
	 * @returns a collection of the values contained in this map
	 */
	values(): Collection
}
```

````