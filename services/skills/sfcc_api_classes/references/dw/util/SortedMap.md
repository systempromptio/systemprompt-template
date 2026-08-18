# dw.util.SortedMap

## Overview
Represents a map maintaining key-value pairs in ascending order, sorted by natural ordering or custom comparator.

## Description
A map that further guarantees ordering in ascending key order, sorted according to the natural ordering of its keys, or by a comparator provided at sorted map creation time. This order is reflected when iterating over the sorted map's collection views (returned by the entrySet, keySet and values methods). Note that sorting by natural order is only supported for Number, String, Date, Money and Quantity as key.

```
Object
  dw.util.Map
    dw.util.SortedMap
```

```ts
declare class SortedMap extends Map {
	/**
	 * Constructor to create a new SortedMap.
	 */
	constructor()

	/**
	 * Constructor to create a new SortedMap with a comparator.
	 * The comparator determines identity and the order of the element keys for this map.
	 * @param comparator - an instance of a PropertyComparator or a comparison function
	 */
	constructor(comparator: Object)

	/**
	 * Returns a shallow copy of this map.
	 * @returns a shallow copy of this map
	 */
	clone(): SortedMap

	/**
	 * Returns the first (lowest) key currently in this sorted map.
	 * @returns the first (lowest) key currently in this sorted map
	 */
	firstKey(): Object

	/**
	 * Returns a view of the portion of this map whose keys are strictly less than toKey.
	 * @param key - high endpoint (exclusive) of the headMap
	 * @returns a view of the portion of this map whose keys are strictly less than toKey
	 */
	headMap(key: Object): SortedMap

	/**
	 * Returns the last (highest) key currently in this sorted map.
	 * @returns the last (highest) key currently in this sorted map
	 */
	lastKey(): Object

	/**
	 * Returns a view of the portion of this map whose keys range from fromKey (inclusive) to toKey (exclusive).
	 * @param from - low endpoint (inclusive) of the subMap
	 * @param to - high endpoint (exclusive) of the subMap
	 * @returns a view of the portion of this map whose keys range from fromKey (inclusive) to toKey (exclusive)
	 */
	subMap(from: Object, to: Object): SortedMap

	/**
	 * Returns a view of the portion of this map whose keys are greater than or equal to fromKey.
	 * @param key - low endpoint (inclusive) of the tailMap
	 * @returns a view of the portion of this map whose keys are greater than or equal to fromKey
	 */
	tailMap(key: Object): SortedMap
}
```
