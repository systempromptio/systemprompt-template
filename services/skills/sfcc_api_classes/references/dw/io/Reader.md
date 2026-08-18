# dw.io.Reader

## Overview
Character-oriented reader that supports reading from strings or input streams and convenient line/substring accessors.

## Description
Provides methods to read characters, lines, or the whole content from an input source. Some convenience methods that read entire streams may be unsafe for very large inputs—prefer streaming (readLine/readN) for big data.

```ts
declare class Reader  {
    /** Deprecated: reads the whole stream and returns a list of lines (unsafe for large inputs). */
    lines: List

    /** Deprecated: reads the whole stream as one string (unsafe for large inputs). */
    string: String

    /** Create a Reader from a string. */
    Reader(source: String): void

    /** Create a Reader from an InputStream using UTF-8. */
    Reader(stream: InputStream): void

    /** Create a Reader from an InputStream using the specified encoding. */
    Reader(stream: InputStream, encoding: String): void

    /** Closes the reader. */
    close(): void

    /** Deprecated: read the whole stream into a list of lines. */
    getLines(): List

    /** Deprecated: read the whole stream into a string. */
    getString(): String

    /** Read a single character (returns single-char string or null at EOF). */
    read(): String

    /** Deprecated: read N characters, may throw on EOF. */
    read(length: Number): String

    /** Read the next line (without line termination). */
    readLine(): String

    /** Read all lines into a List (use with caution for large inputs). */
    readLines(): List

    /** Read up to n characters, returns null on EOF. */
    readN(n: Number): String

    /** Read entire stream as a string (unsafe for large inputs). */
    readString(): String

    /** Returns whether the stream is ready to be read without blocking. */
    ready(): boolean

    /** Skip n characters. */
    skip(n: Number): void
}
```

## All Known Subclasses
- FileReader
