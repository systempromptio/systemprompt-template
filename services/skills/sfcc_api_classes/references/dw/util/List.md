# dw.util.List

## Overview
Ordered collection with indexed access and precise element positioning control. Zero-based, allows duplicates, supports array-like operations.

## Description
Provides precise control over element insertion by index, access elements by integer position, and search capabilities. Lists are zero-based similar to arrays and allow duplicate elements unlike sets.

## All Known Subclasses
ArrayList

```ts
declare class List extends Collection {
	/**
	 * Convenience variable for an empty and immutable list
	 */
	static EMPTY_LIST: List

	/**
	 * Adds the specified object into the list at the specified index
	 */
	addAt(index: Number, value: Object): void

	/**
	 * Creates and returns a new List that is the result of concatenating this list with each of the specified values
	 * This list itself is unmodified
	 * If any specified value is an array or Collection, its elements are appended rather than the object itself
	 */
	concat(...values: Object): List

	/**
	 * Replaces all of the elements in the list with the given object
	 */
	fill(obj: Object): void

	/**
	 * Returns the object at the specified index
	 */
	get(index: Number): Object

	/**
	 * Returns the index of the first occurrence of the specified element, or -1 if not found
	 */
	indexOf(value: Object): Number

	/**
	 * Converts all elements to a string by calling toString() and concatenates them with a comma separator
	 */
	join(): String

	/**
	 * Converts all elements to a string by calling toString() and concatenates them with the specified separator
	 * If separator is null, comma is used
	 */
	join(separator: String): String

	/**
	 * Returns the index of the last occurrence of the specified element, or -1 if not found
	 */
	lastIndexOf(value: Object): Number

	/**
	 * Removes and returns the last element from the list
	 */
	pop(): Object

	/**
	 * Appends the specified values to the end of the list in order
	 */
	push(...values: Object): Number

	/**
	 * Removes the object at the specified index
	 */
	removeAt(index: Number): Object

	/**
	 * Replaces all occurrences of oldValue with newValue
	 */
	replaceAll(oldValue: Object, newValue: Object): boolean

	/**
	 * Reverses the order of the elements in the list
	 */
	reverse(): void

	/**
	 * Rotates the elements in the list by the specified distance
	 */
	rotate(distance: Number): void

	/**
	 * Replaces the object at the specified index with the specified object
	 */
	set(index: Number, value: Object): Object

	/**
	 * Removes and returns the first element of the list
	 * Returns null if list is empty
	 */
	shift(): Object

	/**
	 * Randomly permutes the elements in the list
	 */
	shuffle(): void

	/**
	 * Returns the size of this list
	 */
	size(): Number

	/**
	 * Returns a slice or sublist from the specified index to the end
	 * Negative index counts from the end (-1 is last element)
	 */
	slice(from: Number): List

	/**
	 * Returns a slice or sublist from 'from' up to but not including 'to'
	 * Negative indexes count from the end
	 */
	slice(from: Number, to: Number): List

	/**
	 * Sorts the elements based on their natural order
	 * Sort is stable - equal elements will not be reordered
	 */
	sort(): void

	/**
	 * Sorts the elements using a PropertyComparator or comparison function
	 * Function must take two parameters and return <0, 0, or >0
	 * Sort is stable - equal elements will not be reordered
	 */
	sort(comparator: Object): void

	/**
	 * Returns a list containing the elements from index 'from' to 'to'
	 */
	subList(from: Number, to: Number): List

	/**
	 * Swaps the elements at the specified positions in the list
	 */
	swap(i: Number, j: Number): void

	/**
	 * Inserts values at the beginning of the list
	 * First argument becomes element 0, second becomes element 1, etc.
	 */
	unshift(...values: Object): Number
}
```
