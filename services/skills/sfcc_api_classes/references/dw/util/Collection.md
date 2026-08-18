# dw.util.Collection

## Overview
Represents a collection of objects.

## Description
Represents a collection of objects.

## All Known Subclasses
ArrayList, FilteringCollection, HashSet, LinkedHashSet, List, Set, SortedSet

```ts
declare class Collection  {
	/**
	 * Returns true if the collection is empty.
	 */
	readonly empty: boolean

	/**
	 * The length of the collection. This is similar to a ECMA array of 'products.length'.
	 */
	readonly length: number

	/**
	 * Adds the specified objects to the collection. Can be called with an ECMA array as argument. If called with a single ECMA array as argument the individual elements of that array are added to the collection. If the array object itself should be added use add1().
	 * @param values - the values to add
	 * @returns true if the values were added, false otherwise
	 */
	add(...values: Object): boolean

	/**
	 * Adds a single object to the collection.
	 * @param object - the object to add
	 * @returns true if the object was added, false otherwise
	 */
	add1(object: Object): boolean

	/**
	 * Adds the collection of objects to the collection.
	 * @param objs - the objects to add
	 * @returns true if the objects were added, false otherwise
	 */
	addAll(objs: Collection): boolean

	/**
	 * Clears the collection.
	 */
	clear(): void

	/**
	 * Returns true if the collection contains the specified object.
	 * @param obj - the object to locate in this collection
	 * @returns true if the collection contains the specified object, false otherwise
	 */
	contains(obj: Object): boolean

	/**
	 * Returns true if the collection contains all of the objects in the specified collection.
	 * @param objs - the collection of objects to locate in this collection
	 * @returns true if the collection contains all of the specified objects, false otherwise
	 */
	containsAll(objs: Collection): boolean

	/**
	 * Returns the length of the collection. This is similar to a ECMA array of 'products.length'.
	 * @returns the length of the collection
	 */
	getLength(): number

	/**
	 * Returns true if the collection is empty.
	 * @returns true if the collection is empty, false otherwise
	 */
	isEmpty(): boolean

	/**
	 * Returns an iterator that can be used to access the members of the collection.
	 * @returns an iterator that can be used to access the members of the collection
	 */
	iterator(): Iterator

	/**
	 * Removes the specified object from the collection.
	 * @param obj - the object to remove
	 * @returns true if the specified object was removed, false otherwise
	 */
	remove(obj: Object): boolean

	/**
	 * Removes all of object in the specified object from the collection.
	 * @param objs - the collection of objects to remove
	 * @returns true if all of the specified objects were removed, false otherwise
	 */
	removeAll(objs: Collection): boolean

	/**
	 * Removes all of object in the collection that are not in the specified collection.
	 * @param objs - the collection of objects to retain in the collection
	 * @returns true if the collection retains all of the specified objects, false otherwise
	 */
	retainAll(objs: Collection): boolean

	/**
	 * Returns the size of the collection.
	 * @returns the size of the collection
	 */
	size(): number

	/**
	 * Returns all elements of this collection in a newly created array. The returned array is independent of this collection and can be modified without changing the collection. The elements in the array are in the same order as they are returned when iterating over this collection.
	 * @returns a newly created array
	 */
	toArray(): Array

	/**
	 * Returns a subset of the elements of this collection in a newly created array. The returned array is independent of this collection and can be modified without changing the collection. The elements in the array are in the same order as they are returned when iterating over this collection.
	 * @param start - the number of elements to iterate before adding elements to the array. Negative values are treated as 0
	 * @param size - the maximum number of elements to add to the array. Nonpositive values always result in empty array
	 * @returns a newly created array
	 */
	toArray(start: number, size: number): Array
}
```
