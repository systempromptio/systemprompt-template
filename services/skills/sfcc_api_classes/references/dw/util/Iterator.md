# dw.util.Iterator

## Overview
Allows sequential access to elements in a collection with forward-only iteration.

## Description
The Iterator class allows you to access items in a collection.

## All Known Subclasses
LoopIterator, SeekableIterator

```
Object
  dw.util.Iterator
```

```ts
declare class Iterator  {
	/**
	 * Convert the iterator into a list. After this conversion the iterator is empty and hasNext() will always return false. Use with care - large database results can cause OutOfMemory.
	 * @returns the iterator as a list
	 */
	asList(): List

	/**
	 * Converts a sub-sequence within the iterator into a list. Use with care - large database results can cause OutOfMemory.
	 * @param start - the number of elements to iterate before adding elements to the sublist (negative values treated as 0)
	 * @param size - the maximum number of elements to add to the sublist (nonpositive values always result in empty list)
	 * @returns a sub-sequence within the iterator into a list
	 */
	asList(start: Number, size: Number): List

	/**
	 * Indicates if there are more elements.
	 * @returns true if there are more elements, false otherwise
	 */
	hasNext(): boolean

	/**
	 * Returns the next element from the Iterator.
	 * @returns the next element from the Iterator
	 */
	next(): Object
}
```
