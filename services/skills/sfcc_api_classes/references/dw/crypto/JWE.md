# dw.crypto.JWE

## Overview
Represents a JSON Web Encryption (JWE) object for encrypting and decrypting payloads using public/private keys.

## Description
Handles sensitive security-related data. Supports encryption and decryption of payloads using EC and RSA keys, with support for various algorithms and encryption methods. Provides access to JWE headers and payload.

```ts
declare class JWE  {
    /**
     * Algorithm (alg) from the header (read-only).
     */
    readonly algorithm: string;

    /**
     * Encryption method (enc) from the header (read-only).
     */
    readonly encryptionMethod: string;

    /**
     * Copy of the JWE headers as a Map (read-only).
     */
    readonly headerMap: Map;

    /**
     * Key ID (kid) from the header (read-only).
     */
    readonly keyID: string;

    /**
     * Decrypted payload (read-only).
     */
    readonly payload: string;

    /**
     * Constructs a new JWE for encryption with a string payload.
     * @param header JWE header (must include valid alg and enc).
     * @param payload Content to encrypt.
     */
    constructor(header: JWEHeader, payload: string);

    /**
     * Constructs a new JWE for encryption with a byte array payload.
     * @param header JWE header (must include valid alg and enc).
     * @param payload Content to encrypt.
     */
    constructor(header: JWEHeader, payload: Bytes);

    /**
     * Decrypts the payload using the given private key.
     * @param privateKey Reference to private RSA or EC key.
     */
    decrypt(privateKey: KeyRef): void;

    /**
     * Encrypts the payload using the given public key.
     * @param publicKey Reference to public RSA or EC key.
     */
    encrypt(publicKey: CertificateRef): void;

    /**
     * Gets the algorithm (alg) from the header.
     * @returns Value of the algorithm or null if missing.
     */
    getAlgorithm(): string;

    /**
     * Gets the encryption method (enc) from the header.
     * @returns Value of the encryption method or null if missing.
     */
    getEncryptionMethod(): string;

    /**
     * Gets a copy of the JWE headers as a Map.
     * @returns Copy of the JWE headers.
     */
    getHeaderMap(): Map;

    /**
     * Gets the key id (kid) from the header.
     * @returns Value of the key id or null if missing.
     */
    getKeyID(): string;

    /**
     * Gets the decrypted payload.
     * @returns Payload or null if encrypted.
     */
    getPayload(): string;

    /**
     * Parses a JWE object from its compact serialization format.
     * @param jwe JWE in compact serialization format.
     * @returns JWE object.
     */
    static parse(jwe: string): JWE;

    /**
     * Gets this JWE in compact serialization form.
     * @returns Compact serialized object.
     */
    serialize(): string;
}
```
