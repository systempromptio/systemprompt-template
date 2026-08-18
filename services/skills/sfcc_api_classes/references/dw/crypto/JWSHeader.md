# dw.crypto.JWSHeader

## Overview
Represents an immutable header of a JWS (JSON Web Signature) object.

## Description
Provides access to the algorithm parameter of a JWS header. Supports parsing from objects, Base64URL, and JSON strings. No instances can be created directly.

```ts
declare class JWSHeader  {
    /**
     * Value of the algorithm parameter (alg) (read-only).
     */
    readonly algorithm: string;

    /**
     * Gets the value of the algorithm parameter (alg).
     * @returns Algorithm parameter from this header.
     */
    getAlgorithm(): string;

    /**
     * Converts a Map or object into a JWS header.
     * @param map Map or object data to convert.
     * @returns JWS Header.
     */
    static parse(map: Object): JWSHeader;

    /**
     * Parses a Base64URL-encoded JWS header.
     * @param base64encoded Base64URL string to parse.
     * @returns JWS Header.
     */
    static parseEncoded(base64encoded: string): JWSHeader;

    /**
     * Parses a JWS header from a JSON string.
     * @param json JSON string to parse.
     * @returns JWS Header.
     */
    static parseJSON(json: string): JWSHeader;

    /**
     * Gets a copy of these headers as a Map.
     * @returns Copy of the JWS headers.
     */
    toMap(): Map;

    /**
     * Gets the content of the headers as a JSON String.
     * @returns JSON String.
     */
    toString(): string;
}
```
