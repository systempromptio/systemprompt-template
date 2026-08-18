# dw.util.SortedSet

## Overview
Represents a set maintaining elements in ascending order, sorted by natural ordering or custom comparator.

## Description
A set that further guarantees that its iterator will traverse the set in ascending element order, sorted according to the natural ordering of its elements (only supported for Number, String, Date, Money and Quantity), or by a comparator provided at sorted set creation time.

```
Object
  dw.util.Collection
    dw.util.Set
      dw.util.SortedSet
```

```ts
declare class SortedSet extends Set {
	/**
	 * Constructor to create a new SortedSet.
	 */
	constructor()

	/**
	 * Constructor to create a new SortedSet with a comparator.
	 * The comparator determines identity and the order of the elements for this set.
	 * @param comparator - an instance of a PropertyComparator or a comparison function
	 */
	constructor(comparator: Object)

	/**
	 * Constructor for a new SortedSet initialized with the elements of the given collection.
	 * @param collection - the collection of objects that are inserted into the set
	 */
	constructor(collection: Collection)

	/**
	 * Returns a shallow copy of this set.
	 * @returns a shallow copy of this set
	 */
	clone(): SortedSet

	/**
	 * Returns the first (lowest) element currently in this sorted set.
	 * @returns the first (lowest) element currently in this sorted set
	 */
	first(): Object

	/**
	 * Returns a view of the portion of this sorted set whose elements are strictly less than toElement.
	 * @param key - high endpoint (exclusive) of the headSet
	 * @returns a view of the specified initial range of this sorted set
	 */
	headSet(key: Object): SortedSet

	/**
	 * Returns the last (highest) element currently in this sorted set.
	 * @returns the last (highest) element currently in this sorted set
	 */
	last(): Object

	/**
	 * Returns a view of the portion of this sorted set whose elements range from fromElement (inclusive) to toElement (exclusive).
	 * @param from - low endpoint (inclusive) of the subSet
	 * @param to - high endpoint (exclusive) of the subSet
	 * @returns a view of the specified range within this sorted set
	 */
	subSet(from: Object, to: Object): SortedSet

	/**
	 * Returns a view of the portion of this sorted set whose elements are greater than or equal to fromElement.
	 * @param key - low endpoint (inclusive) of the tailSet
	 * @returns a view of the specified final range of this sorted set
	 */
	tailSet(key: Object): SortedSet
}
```
