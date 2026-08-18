# dw.web.PagingModel

## Overview
Helper class for applying pagination to collections and iterators with URL generation support.

## Description
A page model is a helper class to apply a pages to a collection of elements or an iterator of elements and supports creating URLs for continued paging through the elements.

The page model is intended to be initialized with the collection or iterator, than the paging position is applied and than the elements are extracted with getPageElements().

In case the page model is initialized with a collection the page model can be reused multiple times.

```
Object
  dw.web.PagingModel
```

```ts
declare class PagingModel  {
	/**
	 * The default page size.
	 */
	static DEFAULT_PAGE_SIZE: 10

	/**
	 * The maximum supported page size.
	 */
	static MAX_PAGE_SIZE: 2000

	/**
	 * The URL Parameter used for the page size.
	 */
	static PAGING_SIZE_PARAMETER: 'sz'

	/**
	 * The URL parameter used for the start position.
	 */
	static PAGING_START_PARAMETER: 'start'

	/**
	 * The count of the number of items in the model.
	 */
	readonly count: number

	/**
	 * The index number of the current page. The page counting starts with 0. The method also works with a miss-aligned start.
	 */
	readonly currentPage: number

	/**
	 * Identifies if the model is empty.
	 */
	readonly empty: boolean

	/**
	 * The index of the last element on the current page.
	 */
	readonly end: number

	/**
	 * The maximum possible page number. Counting for pages starts with 0.
	 */
	readonly maxPage: number

	/**
	 * The total page count. The method also works with a miss-aligned start.
	 */
	readonly pageCount: number

	/**
	 * An iterator that can be used to iterate through the elements of the current page. In case of a collection as source, can be called multiple times. In case of an iterator as source, must be called only once.
	 */
	readonly pageElements: Iterator

	/**
	 * The size of the page.
	 */
	pageSize: number

	/**
	 * The current start position from which iteration will start.
	 */
	start: number

	/**
	 * Constructs the PagingModel using the specified iterator and count value.
	 * @param elements - The iterator of elements
	 * @param count - The total count
	 */
	constructor(elements: Iterator, count: number)

	/**
	 * Constructs the PagingModel using the specified collection.
	 * @param elements - The collection of elements
	 */
	constructor(elements: Collection)

	/**
	 * Returns an URL containing the page size parameter appended to the specified url.
	 * @param url - The URL to append to
	 * @param pageSize - The page size
	 * @returns URL with page size parameter
	 */
	static appendPageSize(url: URL, pageSize: number): URL

	/**
	 * Returns an URL by appending the current page start position and the current page size to the URL.
	 * @param url - The URL to append to
	 * @returns URL with paging parameters
	 */
	appendPaging(url: URL): URL

	/**
	 * Returns an URL by appending the paging parameters for a desired page start position and the current page size to the specified url.
	 * @param url - The URL to append to
	 * @param position - The desired page start position
	 * @returns URL with paging parameters
	 */
	appendPaging(url: URL, position: number): URL

	/**
	 * Returns the count of the number of items in the model.
	 * @returns The count
	 */
	getCount(): number

	/**
	 * Returns the index number of the current page. Page counting starts with 0.
	 * @returns The current page index
	 */
	getCurrentPage(): number

	/**
	 * Returns the index of the last element on the current page.
	 * @returns The end index
	 */
	getEnd(): number

	/**
	 * Returns the maximum possible page number. Counting for pages starts with 0.
	 * @returns The maximum page number
	 */
	getMaxPage(): number

	/**
	 * Returns the total page count.
	 * @returns The page count
	 */
	getPageCount(): number

	/**
	 * Returns an iterator that can be used to iterate through the elements of the current page.
	 * @returns Iterator of page elements
	 */
	getPageElements(): Iterator

	/**
	 * Returns the size of the page.
	 * @returns The page size
	 */
	getPageSize(): number

	/**
	 * Returns the current start position from which iteration will start.
	 * @returns The start position
	 */
	getStart(): number

	/**
	 * Identifies if the model is empty.
	 * @returns True if empty, false otherwise
	 */
	isEmpty(): boolean

	/**
	 * Sets the size of the page.
	 * @param pageSize - The page size to set
	 */
	setPageSize(pageSize: number): void

	/**
	 * Sets the current start position from which iteration will start.
	 * @param start - The start position to set
	 */
	setStart(start: number): void
}
```
