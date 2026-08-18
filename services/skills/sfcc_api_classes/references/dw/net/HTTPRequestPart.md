# dw.net.HTTPRequestPart

## Overview
Represents a part for multipart HTTP requests — string, bytes, or file parts with optional content type, encoding and filename.

## Description
Used to construct multipart form data; parts have a name and a value (String, Bytes, or File). Encoding, contentType and fileName can be specified for Bytes/File parts.

```ts
declare class HTTPRequestPart  {
    /** Read-only bytes value of the part, if applicable. */
    bytesValue: Bytes

    /** Content type of this part. */
    contentType: string

    /** Charset encoding for string parts. */
    encoding: string

    /** File name used in the multipart header. */
    fileName: string

    /** Read-only File value of the part, if applicable. */
    fileValue: File

    /** Name of the part. */
    name: string

    /** Read-only string value of the part, if applicable. */
    stringValue: string

    constructor(name: string, value: string)
    constructor(name: string, value: string, encoding: string)
    constructor(name: string, file: File)
    constructor(name: string, data: Bytes)
    constructor(name: string, data: Bytes, contentType: string, encoding: string, fileName: string)
    constructor(name: string, file: File, contentType: string, encoding: string)
    constructor(name: string, file: File, contentType: string, encoding: string, fileName: string)

    /** Get bytes value or null if not a bytes part. */
    getBytesValue(): Bytes

    /** Returns the content type or null if not specified. */
    getContentType(): string

    /** Returns the encoding or null if not specified. */
    getEncoding(): string

    /** Returns the file name used in multipart headers. */
    getFileName(): string

    /** Returns the File value or null if not a file part. */
    getFileValue(): File

    /** Returns the part name (never null). */
    getName(): string

    /** Returns the string value or null if not a string part. */
    getStringValue(): string
}
```
