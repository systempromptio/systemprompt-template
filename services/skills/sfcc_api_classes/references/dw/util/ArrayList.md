# dw.util.ArrayList

## Overview
Container for a list of objects with array-like operations, supporting construction from collections, iterators, or variadic arguments.

## Description
The ArrayList class is a container for a list of objects.

```ts
declare class ArrayList extends List {
	/**
	 * Constructor for a new ArrayList.
	 */
	constructor()

	/**
	 * Constructor for a new ArrayList. Initializes the ArrayList with the elements of the given collection.
	 * @param collection - the elements to insert into the list
	 */
	constructor(collection: Collection)

	/**
	 * Constructor for a new ArrayList. Initializes the ArrayList with the elements of the given iterator.
	 * @param iterator - the iterator that provides access to the elements to insert into the list
	 */
	constructor(iterator: Iterator)

	/**
	 * Constructor for a new ArrayList. Initializes the ArrayList with the arguments given as constructor parameters. Can also be called with an ECMA array as argument. If called with a single ECMA array as argument, the individual elements of that array are used to initialize the ArrayList. To create an ArrayList with a single array as element, create an empty ArrayList and then call add1() on it.
	 * @param values - the set of objects to insert into the list
	 */
	constructor(...values: Object[])

	/**
	 * Returns a shallow copy of this array list.
	 * @returns a shallow copy of this array list
	 */
	clone(): ArrayList
}
```
