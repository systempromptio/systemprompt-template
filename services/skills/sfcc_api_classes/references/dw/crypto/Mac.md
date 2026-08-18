# dw.crypto.Mac

## Overview
Message Authentication Code (MAC) algorithms and helpers (HMAC variants). Computes keyed hash values to verify integrity and authenticity.

## Description
Provides HMAC-style message authentication using underlying hash algorithms (SHA-256, SHA-384, SHA-512). Deprecated SHA-1 and MD5 constants remain but should not be used. Methods compute MACs from strings or bytes using a secret key; keys are not validated for format or parameters.

```ts
declare class Mac  {
    /** HmacMD5 (deprecated) */
    static HMAC_MD5: 'HmacMD5'
    /** HmacSHA1 (deprecated) */
    static HMAC_SHA_1: 'HmacSHA1'
    /** HmacSHA256 */
    static HMAC_SHA_256: 'HmacSHA256'
    /** HmacSHA384 */
    static HMAC_SHA_384: 'HmacSHA384'
    /** HmacSHA512 */
    static HMAC_SHA_512: 'HmacSHA512'

    /**
     * Construct a Mac instance for the given algorithm name (e.g. HmacSHA256).
     * @param algorithm Standard algorithm name, must not be null.
     */
    constructor(algorithm: string)

    /**
     * Computes the MAC for the given string input using a string key.
     * Input is converted to UTF-8 bytes before hashing.
     */
    digest(input: string, key: string): import('../../dw/util/Bytes').Bytes

    /**
     * Computes the MAC for the given string input using a bytes key.
     */
    digest(input: string, key: import('../../dw/util/Bytes').Bytes): import('../../dw/util/Bytes').Bytes

    /**
     * Computes the MAC for the given bytes input using a bytes key.
     */
    digest(input: import('../../dw/util/Bytes').Bytes, key: import('../../dw/util/Bytes').Bytes): import('../../dw/util/Bytes').Bytes
}
```
