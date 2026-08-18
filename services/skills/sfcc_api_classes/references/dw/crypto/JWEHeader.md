# dw.crypto.JWEHeader

## Overview
Represents an immutable header of a JWE (JSON Web Encryption) object.

## Description
Provides access to the algorithm and encryption method parameters of a JWE header. Supports parsing from objects, Base64URL, and JSON strings. No instances can be created directly.

```ts
declare class JWEHeader  {
    /**
     * Value of the algorithm parameter (alg) (read-only).
     */
    readonly algorithm: string;

    /**
     * Value of the encryption algorithm parameter (enc) (read-only).
     */
    readonly encryptionAlgorithm: string;

    /**
     * Gets the value of the algorithm parameter (alg).
     * @returns Algorithm parameter from this header.
     */
    getAlgorithm(): string;

    /**
     * Gets the value of the encryption algorithm parameter (enc).
     * @returns Encryption algorithm parameter from this header.
     */
    getEncryptionAlgorithm(): string;

    /**
     * Converts a Map or object into a JWE header.
     * @param map Map or object data to convert.
     * @returns JWE Header.
     */
    static parse(map: Object): JWEHeader;

    /**
     * Parses a Base64URL-encoded JWE header.
     * @param base64encoded Base64URL string to parse.
     * @returns JWE Header.
     */
    static parseEncoded(base64encoded: string): JWEHeader;

    /**
     * Parses a JWE header from a JSON string.
     * @param json JSON string to parse.
     * @returns JWE Header.
     */
    static parseJSON(json: string): JWEHeader;

    /**
     * Gets a copy of these headers as a Map.
     * @returns Copy of the JWE headers.
     */
    toMap(): Map;

    /**
     * Gets the content of the headers as a JSON String.
     * @returns JSON String.
     */
    toString(): string;
}
```
