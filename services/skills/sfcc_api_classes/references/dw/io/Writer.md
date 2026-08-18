# dw.io.Writer

## Overview
Base class for character-oriented writers that write to an underlying `OutputStream`.

## Description
Creates a writer from an `OutputStream` and supports writing strings, flushing and closing the stream.

```ts
declare class Writer  {
    /** Create a Writer from an OutputStream using UTF-8. */
    Writer(stream: OutputStream): void

    /** Create a Writer from an OutputStream using the specified encoding. */
    Writer(stream: OutputStream, encoding: String): void

    /** Closes the writer. */
    close(): void

    /** Flushes the buffer. */
    flush(): void

    /** Writes the given string to the stream. */
    write(str: String): void

    /** Writes a substring of `str` starting at `off` of length `len`. */
    write(str: String, off: Number, len: Number): void
}
```

## All Known Subclasses
- FileWriter
- PrintWriter
- StringWriter
