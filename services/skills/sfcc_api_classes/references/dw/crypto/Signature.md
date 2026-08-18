# dw.crypto.Signature

## Overview
Adapter to Java's signature services (JCA). Supports RSA and ECDSA signature and verification using several digest algorithms.

## Description
Provides signing and verification helpers for algorithms such as SHA256withRSA, SHA384withRSA, SHA512withRSA, RSA-PSS variants, and ECDSA variants. Methods accept base64-encoded strings or `Bytes` and support keys supplied as base64 strings or `KeyRef`/`CertificateRef` references. Key size constraints apply and signatures are returned as base64 or bytes.

```ts
declare class Signature  {
    /** Supported digest algorithm names */
    static SUPPORTED_DIGEST_ALGORITHMS_AS_ARRAY: string[]

    constructor()

    /** Checks whether the digest algorithm is supported. */
    isDigestAlgorithmSupported(digestAlgorithm: string): boolean

    /** Signs base64 content using a base64 private key and returns base64 signature. */
    sign(contentToSign: string, privateKey: string, digestAlgorithm: string): string

    /** Signs base64 content using a KeyRef to a private key and returns base64 signature. */
    sign(contentToSign: string, privateKey: import('./KeyRef').KeyRef, digestAlgorithm: string): string

    /** Signs bytes and returns signature bytes (private key as base64). */
    signBytes(contentToSign: import('../../dw/util/Bytes').Bytes, privateKey: string, digestAlgorithm: string): import('../../dw/util/Bytes').Bytes

    /** Signs bytes and returns signature bytes (private key as KeyRef). */
    signBytes(contentToSign: import('../../dw/util/Bytes').Bytes, privateKey: import('./KeyRef').KeyRef, digestAlgorithm: string): import('../../dw/util/Bytes').Bytes

    /** Verifies a signature (bytes) against bytes content using a base64 public key. */
    verifyBytesSignature(signature: import('../../dw/util/Bytes').Bytes, contentToVerify: import('../../dw/util/Bytes').Bytes, publicKey: string, digestAlgorithm: string): boolean

    /** Verifies a signature (bytes) against bytes content using a CertificateRef. */
    verifyBytesSignature(signature: import('../../dw/util/Bytes').Bytes, contentToVerify: import('../../dw/util/Bytes').Bytes, certificate: import('./CertificateRef').CertificateRef, digestAlgorithm: string): boolean

    /** Verifies a base64 signature against base64 content using a base64 public key. */
    verifySignature(signature: string, contentToVerify: string, publicKey: string, digestAlgorithm: string): boolean

    /** Verifies a base64 signature against base64 content using a CertificateRef. */
    verifySignature(signature: string, contentToVerify: string, certificate: import('./CertificateRef').CertificateRef, digestAlgorithm: string): boolean
}
```
