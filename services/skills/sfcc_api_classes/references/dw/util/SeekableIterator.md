# dw.util.SeekableIterator

## Overview
Iterator supporting forward seeking and paging in large result sets, primarily returned from search operations.

## Description
A special Iterator, which is returned by the system to iterate through large sets of data. The iterator supports seeking forward to a random position. This is a typical action when paging forward in a result set. The Iterator is primarily returned from search operations. Starting with API version 10.6, these iterators can only be iterated once to avoid possible memory problems for really large result sets. Putting them into the pipeline dictionary and trying to loop them multiple times is no longer possible because this would require buffering the iterated elements internally. Prior to 10.6, and for all customers still running API version 10.4 (compatibility mode), SeekableIterator instances stored in the pipeline dictionary could be iterated multiple times (for example, by several loop nodes).

```ts
declare class SeekableIterator extends Iterator {
  /**
   * The total element count for this iterator. The method returns -1, if the total count is not known.
   */
  readonly count: number

  /**
   * Returns a list representing a subsequence within the iterator. The underlying system resources of the iterator will be closed at the end. The start position must be 0 or a positive number.
   * @param start - the position from which to start the subsequence.
   * @param size - the number of items to collect.
   * @returns the list containing the subsequence.
   */
  asList(start: number, size: number): List

  /**
   * Closes all system resources associated with this iterator. Calling this method is strongly recommended if not all elements of this iterator are retrieved. This will allow the system to release system resources immediately. The SeekableIterator is closed automatically if all elements are retrieved. Then calling method close() is optional.
   */
  close(): void

  /**
   * Returns the first element of this iterator and closes it. If the iterator does not contain another element null is returned. If any of the methods next(), forward(int) or forward(int,int) have been called before null is returned. This method is useful if only the first element of an iterator is needed.
   * @returns the first element of an iterator and closes the iterator or returns null if the iterator doesn't have another element or the methods next(), forward(int) or forward(int,int) have already been called.
   */
  first(): Object

  /**
   * Seeks forward by the given number of elements. The number of seek steps must be 0 or a positive number.
   * @param n - the number of elements to seek forward.
   */
  forward(n: number): void

  /**
   * Seeks forward by the given number of elements and limits the iteration to the given number of elements. The method is typically used to position and trim an iterator for paging. The getCount() method will still return the total count of the underlying data collection.
   * @param n - the number of elements to seek forward.
   * @param size - the maximum number of elements return from the iterator
   */
  forward(n: number, size: number): void

  /**
   * Returns the total element count for this iterator. The method returns -1, if the total count is not known.
   * @returns the total element count for this iterator or -1.
   */
  getCount(): number

  /**
   * Indicates if there are more elements.
   * @returns true if there are more elements, false otherwise.
   */
  hasNext(): boolean

  /**
   * Returns the next element from the Iterator.
   * @returns the next element from the Iterator.
   */
  next(): Object
}
```
