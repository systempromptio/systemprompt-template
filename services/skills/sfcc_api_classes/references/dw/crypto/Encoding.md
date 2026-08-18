# dw.crypto.Encoding

## Overview
Utility class for handling common character encodings, including Base64, hexadecimal, and URL encoding/decoding.

## Description
Provides static methods to encode and decode strings and byte arrays using Base64, Base64URL, hexadecimal, and URI-safe formats. No instances can be created.

```ts
declare class Encoding  {
    /**
     * Decodes a Base64 or Base64URL string to a byte array.
     * @param string A string in base-64 alphabet to decode.
     * @returns The decoded array of bytes.
     */
    static fromBase64(string: string): Bytes;

    /**
     * Converts a hexadecimal string to a byte array.
     * @param string A string containing only hex characters to decode.
     * @returns The decoded array of bytes.
     */
    static fromHex(string: string): Bytes;

    /**
     * Decodes a URL-safe string into its original form.
     * @param string The string to decode.
     * @returns The decoded string.
     */
    static fromURI(string: string): string;

    /**
     * Decodes a URL-safe string using the specified encoding.
     * @param string The string to decode.
     * @param encoding The name of a supported encoding.
     * @returns The decoded string.
     */
    static fromURI(string: string, encoding: string): string;

    /**
     * Encodes a byte array to a Base64 string.
     * @param bytes The array of bytes to encode.
     * @returns The encoded string containing only Base64 characters.
     */
    static toBase64(bytes: Bytes): string;

    /**
     * Encodes a byte array to a Base64URL string.
     * @param bytes The array of bytes to encode.
     * @returns The encoded string containing only Base64URL characters.
     */
    static toBase64URL(bytes: Bytes): string;

    /**
     * Converts a byte array to a hexadecimal string.
     * @param bytes The array of bytes to encode.
     * @returns The encoded string containing only hex characters.
     */
    static toHex(bytes: Bytes): string;

    /**
     * Encodes a string into its URL-safe form using the default encoding.
     * @param string The string to encode.
     * @returns The encoded string.
     */
    static toURI(string: string): string;

    /**
     * Encodes a string into its URL-safe form using the specified encoding.
     * @param string The string to encode.
     * @param encoding The name of a supported encoding.
     * @returns The encoded string.
     */
    static toURI(string: string, encoding: string): string;
}
```
