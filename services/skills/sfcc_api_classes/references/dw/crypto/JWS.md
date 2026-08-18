# dw.crypto.JWS

## Overview
Represents a JSON Web Signature (JWS) object for signing and verifying payloads using public/private keys.

## Description
Handles sensitive security-related data. Supports signing and verifying payloads using EC and RSA keys, with support for various algorithms. Provides access to JWS headers and payload.

```ts
declare class JWS  {
    /**
     * Algorithm (alg) from the header (read-only).
     */
    readonly algorithm: string;

    /**
     * JWS header (read-only).
     */
    readonly header: JWSHeader;

    /**
     * Copy of the JWS header as a Map (read-only).
     */
    readonly headerMap: Map;

    /**
     * Payload from this object (read-only).
     */
    readonly payload: string;

    /**
     * Constructs a new JWS for signing with a string payload.
     * @param header JWS header (must include valid alg).
     * @param payload Content to sign.
     */
    constructor(header: JWSHeader, payload: string);

    /**
     * Constructs a new JWS for signing with a byte array payload.
     * @param header JWS header (must include valid alg).
     * @param payload Content to sign.
     */
    constructor(header: JWSHeader, payload: Bytes);

    /**
     * Gets the algorithm (alg) from the header.
     * @returns Value of the algorithm or null if missing.
     */
    getAlgorithm(): string;

    /**
     * Gets a copy of the JWS header.
     * @returns Copy of the JWS header.
     */
    getHeader(): JWSHeader;

    /**
     * Gets a copy of the JWS header as a Map.
     * @returns Copy of the JWS header.
     */
    getHeaderMap(): Map;

    /**
     * Gets the payload from this object.
     * @returns UTF-8 encoded payload.
     */
    getPayload(): string;

    /**
     * Parses a JWS object from its compact serialization format.
     * @param jws JWS in compact serialization format.
     * @returns JWS object.
     */
    static parse(jws: string): JWS;

    /**
     * Parses a JWS object from its compact serialization format with a detached payload.
     * @param jws JWS without a payload in compact serialization format.
     * @param payload Detached payload.
     * @returns JWS object.
     */
    static parse(jws: string, payload: string): JWS;

    /**
     * Parses a JWS object from its compact serialization format with a detached payload as bytes.
     * @param jws JWS without a payload in compact serialization format.
     * @param payload Detached payload as bytes.
     * @returns JWS object.
     */
    static parse(jws: string, payload: Bytes): JWS;

    /**
     * Serializes this JWS in compact serialization form.
     * @param detachPayload true for a detached payload, false to serialize the payload too.
     * @returns Compact serialized object.
     */
    serialize(detachPayload: boolean): string;

    /**
     * Signs the payload using the given private key.
     * @param keyRef Reference to the private key.
     */
    sign(keyRef: KeyRef): void;

    /**
     * Verifies the signature of the payload.
     * @param certificateRef Reference to the certificate to use for verification.
     * @returns true if verification succeeds, false otherwise.
     */
    verify(certificateRef: CertificateRef): boolean;
}
```
