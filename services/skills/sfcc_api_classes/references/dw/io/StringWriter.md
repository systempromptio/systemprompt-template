# dw.io.StringWriter

## Overview
A `Writer` implementation that accumulates written characters into a String.

## Description
Use to build strings in memory. For large outputs prefer writing directly to files to reduce memory usage.

```ts
declare class StringWriter extends Writer {
    /** Constructs a new StringWriter. */
    StringWriter(): void

    /** Returns a string representation of this writer's content. */
    toString(): String

    /** Writes the given string to the stream. */
    write(str: String): void

    /** Writes a substring of `str` starting at `off` of length `len`. */
    write(str: String, off: Number, len: Number): void
}
```
