# dw.value.MimeEncodedText

## Overview
Wraps an immutable text payload together with its mime type and encoding metadata.

## Description
The constructor freezes the encoded text, mime type, and encoding so reads always return the same values; the single string constructor defaults to text/plain;charset=UTF-8 with UTF-8 encoding while the full constructor lets callers specify every field.

```ts
declare class MimeEncodedText  {
    /**
     * @readonly
     * The encoding that was supplied during construction.
     */
    readonly encoding: string

    /**
     * @readonly
     * The mime type that was supplied during construction.
     */
    readonly mimeType: string

    /**
     * @readonly
     * The text payload that was supplied during construction.
     */
    readonly text: string

    /**
     * Returns the encoding assigned at construction.
     */
    getEncoding(): string

    /**
     * Returns the mime type assigned at construction.
     */
    getMimeType(): string

    /**
     * Returns the stored text.
     */
    getText(): string
}
```