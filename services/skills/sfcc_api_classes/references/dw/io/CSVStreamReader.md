# dw.io.CSVStreamReader

## Overview
Reader for CSV input that supports quoted separators and quoted entries containing newlines.

## Description
Parses CSV streams from a `Reader`. Supports configurable separator and quote characters and optional header-line skipping. Use `readNext()` to iterate safely over large files; `readAll()` returns all lines and may OOM on large inputs.

## All Known Subclasses
No known subclasses.

```ts
declare class CSVStreamReader  {
    /** Create reader with default separator ',' and quote '"' */
    constructor(ioreader: Reader)

    /** Create reader with specified separator (quote defaults to '"') */
    constructor(ioreader: Reader, separator: string)

    /** Create reader with specified separator and quote char */
    constructor(ioreader: Reader, separator: string, quote: string)

    /** Create reader with separator, quote and number of header lines to skip */
    constructor(ioreader: Reader, separator: string, quote: string, skip: number)

    /** Closes the underlying reader. */
    close(): void

    /** Returns a List of all lines; each line is an array of strings. May OOM for large files. */
    readAll(): List

    /** Reads and returns the next line as an array of strings; returns null at EOF. */
    readNext(): string[]
}
```
