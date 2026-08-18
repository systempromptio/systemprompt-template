# dw.io.FileWriter

## Overview
Convenience `Writer` for character files with helpers for line-based writing and configurable line separator.

## Description
Construct `FileWriter` for a target `File` with optional encoding and append mode. Provides `writeLine()` which appends the configured line separator. Be careful when writing sensitive data to disk.

## All Known Subclasses
No known subclasses.

```ts
declare class FileWriter extends Writer {
    /** Current line separator string (defaults to '\n'). */
    lineSeparator: string

    /** Constructs a FileWriter for the given File (UTF-8 encoding by default). */
    constructor(file: File)

    /** Constructs a FileWriter with optional append mode. */
    constructor(file: File, append: boolean)

    /** Constructs a FileWriter with specified encoding. */
    constructor(file: File, encoding: string)

    /** Constructs a FileWriter with encoding and append mode. */
    constructor(file: File, encoding: string, append: boolean)

    /** Closes the writer and releases resources. */
    close(): void

    /** Returns the current line separator. */
    getLineSeparator(): string

    /** Sets the line separator (e.g. '\n' or '\r\n'). */
    setLineSeparator(lineSeparator: string): void

    /** Writes a line and appends the line separator. */
    writeLine(str: string): void
}
```
