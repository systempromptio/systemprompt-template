# dw.io.CSVStreamWriter

## Overview
Writer for creating CSV files with configurable separator and quote characters.

## Description
Writes CSV lines to a `Writer`. Supports specifying separator and quote characters. Use with care for sensitive data persisted to disk.

## All Known Subclasses
No known subclasses.

```ts
declare class CSVStreamWriter  {
    /** Create writer with default separator ',' and quote '"' */
    constructor(writer: Writer)

    /** Create writer with specified separator (quote defaults to '"') */
    constructor(writer: Writer, separator: string)

    /** Create writer with specified separator and quote character */
    constructor(writer: Writer, separator: string, quote: string)

    /** Closes the underlying writer. */
    close(): void

    /** Writes a single CSV line composed of the provided strings. */
    writeNext(...line: string[]): void
}
```
