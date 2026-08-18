# dw.crypto.MessageDigest

## Overview
Stateless message digest (hash) algorithms such as SHA-256 and SHA-512. Provides one-shot and incremental digest operations.

## Description
Computes one-way hash values for strings or bytes. Deprecated constants for older algorithms (MD2/MD5/SHA-1) remain but should be avoided in favor of SHA-256 or SHA-512. Methods return either Bytes or hex/base64-encoded strings depending on the overload.

```ts
declare class MessageDigest  {
    static DIGEST_MD2: 'MD2'
    static DIGEST_MD5: 'MD5'
    static DIGEST_SHA: 'SHA'
    static DIGEST_SHA_1: 'SHA-1'
    static DIGEST_SHA_256: 'SHA-256'
    static DIGEST_SHA_512: 'SHA-512'

    /** Construct with algorithm name (SHA-256 or SHA-512) */
    constructor(algorithm: string)

    /** Deprecated: digests a string and returns a hex-encoded string (platform default encoding used). */
    digest(input: string): string

    /** Computes the digest for provided algorithm (or constructor algorithm if null) and byte input. */
    digest(algorithm: string | null, input: import('../../dw/util/Bytes').Bytes): import('../../dw/util/Bytes').Bytes

    /** Completes the digest and returns resulting bytes. */
    digest(): import('../../dw/util/Bytes').Bytes

    /** Computes digest for given Bytes input. */
    digestBytes(input: import('../../dw/util/Bytes').Bytes): import('../../dw/util/Bytes').Bytes

    /** Updates the digest with additional bytes for incremental computation. */
    updateBytes(input: import('../../dw/util/Bytes').Bytes): void
}
```
