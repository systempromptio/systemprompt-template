# dw.web.LoopIterator

## Overview
Iterator used in ISLOOP implementation with properties for loop status tracking.

## Description
Iterator used in &lt;ISLOOP&gt; implementation. It defines properties used to determine loop status. LoopIterator object is assigned to variable declared in "status" attribute of the &lt;ISLOOP&gt; tag.

```
Object
  dw.util.Iterator
    dw.web.LoopIterator
```

```ts
declare class LoopIterator extends Iterator {
	/**
	 * Return begin iteration index. By default begin index is 0.
	 */
	readonly begin: number

	/**
	 * Return iteration count, starting with 1.
	 */
	readonly count: number

	/**
	 * Return end iteration index. By default end index equals 'length - 1', provided that length is determined. If length cannot be determined end index is -1.
	 */
	readonly end: number

	/**
	 * Identifies if count is an even value.
	 */
	readonly even: boolean

	/**
	 * Identifies if the iterator is positioned at first iteratable item.
	 */
	readonly first: boolean

	/**
	 * Return iteration index, which is the position of the iterator in the underlying iteratable object. Index is 0-based and is calculated according the following formula: Index = (Count - 1) * Step.
	 */
	readonly index: number

	/**
	 * Identifies if the iterator is positioned at last iteratable item.
	 */
	readonly last: boolean

	/**
	 * Return the length of the object. If length cannot be determined, -1 is returned.
	 */
	readonly length: number

	/**
	 * Identifies if count is an odd value.
	 */
	readonly odd: boolean

	/**
	 * Return iterator step.
	 */
	readonly step: number

	/**
	 * Returns the begin iteration index. By default begin index is 0.
	 * @returns The begin iteration index
	 */
	getBegin(): number

	/**
	 * Returns iteration count, starting with 1.
	 * @returns The iteration count
	 */
	getCount(): number

	/**
	 * Returns end iteration index. By default end index equals 'length - 1', provided that length is determined. If length cannot be determined end index is -1.
	 * @returns The end iteration index
	 */
	getEnd(): number

	/**
	 * Returns iteration index, which is the position of the iterator in the underlying iteratable object. Index is 0-based and is calculated according the following formula: Index = (Count - 1) * Step.
	 * @returns The iteration index
	 */
	getIndex(): number

	/**
	 * Returns the length of the object. If length cannot be determined, -1 is returned.
	 * @returns The length of the object
	 */
	getLength(): number

	/**
	 * Returns iterator step.
	 * @returns The iterator step
	 */
	getStep(): number

	/**
	 * Identifies if count is an even value.
	 * @returns True if even, false otherwise
	 */
	isEven(): boolean

	/**
	 * Identifies if the iterator is positioned at first iteratable item.
	 * @returns True if first, false otherwise
	 */
	isFirst(): boolean

	/**
	 * Identifies if the iterator is positioned at last iteratable item.
	 * @returns True if last, false otherwise
	 */
	isLast(): boolean

	/**
	 * Identifies if count is an odd value.
	 * @returns True if odd, false otherwise
	 */
	isOdd(): boolean
}
```
