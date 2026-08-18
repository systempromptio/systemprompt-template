# dw.io.FileReader

## Overview
`FileReader` reads from a `dw.io.File` and extends `Reader` with file-specific constructors.

## Description
Construct `FileReader` with a `File` and optional encoding. Close when finished to release resources. Inherits reading helpers from `dw.io.Reader` (`read`, `readLine`, `readLines`, `getString`, etc.).

## All Known Subclasses
No known subclasses.

```ts
declare class FileReader extends Reader {
    /** Constructs a FileReader for the given File. */
    constructor(file: File)

    /** Constructs a FileReader for the given File with specified encoding. */
    constructor(file: File, encoding: string)

    /** Close the reader and release resources. */
    close(): void
}
```
