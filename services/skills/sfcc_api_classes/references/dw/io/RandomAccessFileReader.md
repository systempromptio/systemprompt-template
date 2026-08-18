# dw.io.RandomAccessFileReader

## Overview
Reader for random-access files. Behaves like a byte array with a movable file pointer.

## Description
Supports reading bytes at arbitrary positions in a file. Use `getPosition`/`setPosition` to manage the file pointer. Close when finished to release resources.

```ts
declare class RandomAccessFileReader  {
    /** Maximum bytes that can be read in a single `readBytes` call (10240). */
    static MAX_READ_BYTES: Number = 10240

    /** Current offset in the file. */
    position: Number

    /** Constructs a reader for the given File. */
    RandomAccessFileReader(file: File): void

    /** Closes the reader and releases system resources. */
    close(): void

    /** Returns the current offset in the file. */
    getPosition(): Number

    /** Returns the length of the file in bytes. */
    length(): Number

    /** Reads a signed 8-bit value from the current file pointer. */
    readByte(): Number

    /** Reads up to `numBytes` bytes starting at the current pointer. */
    readBytes(numBytes: Number): Bytes

    /** Sets the file-pointer offset (may be beyond EOF). */
    setPosition(position: Number): void
}
```
