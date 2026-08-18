# dw.util.LinkedHashSet

## Overview
HashSet implementation with guaranteed iteration order, iterating elements in the order they were added.

## Description
The class LinkedHashSet implements a hash set with a guaranteed iteration order. The elements are iterated in the order they have been added to the HashSet.

```
Object
  dw.util.Collection
    dw.util.Set
      dw.util.LinkedHashSet
```

```ts
declare class LinkedHashSet extends Set {
	/**
	 * Constructs a new LinkedHashSet.
	 */
	constructor()

	/**
	 * Constructor for a new LinkedHashSet by initializing the LinkedHashSet with the elements of the given collection.
	 * @param collection - the collection of items to insert into this set
	 */
	constructor(collection: Collection)

	/**
	 * Returns a shallow copy of this set.
	 * @returns a shallow copy of this set
	 */
	clone(): LinkedHashSet
}
```
