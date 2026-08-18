# dw.util.HashSet

## Overview
Represents a HashSet of objects, providing unordered collection with unique elements and hash-based lookup.

## Description
Represents a HashSet.

```
Object
  dw.util.Collection
    dw.util.Set
      dw.util.HashSet
```

```ts
declare class HashSet extends Set {
	/**
	 * Constructs a new HashSet.
	 */
	constructor()

	/**
	 * Construct a new HashSet by initializing the HashSet with the elements of the given collection.
	 * @param collection - the collection to add to the set
	 */
	constructor(collection: Collection)

	/**
	 * Returns a shallow copy of this set.
	 * @returns a shallow copy of this set
	 */
	clone(): HashSet
}
```
