 # dw.crypto.WeakMessageDigest

 ## Overview
 Compatibility wrapper for legacy message digest algorithms (MD2, MD5, SHA, SHA-1).

 ## Description
 Provides hashing helpers intended for backward compatibility. Prefer `MessageDigest` for current usage. Methods include digesting strings and byte arrays and incremental updates.

 ```ts
 declare class WeakMessageDigest  {
     static DIGEST_MD2: 'MD2'
     static DIGEST_MD5: 'MD5'
     static DIGEST_SHA: 'SHA'
     static DIGEST_SHA_1: 'SHA-1'

     constructor(algorithm: string)

     /** Digest a string and return hex-encoded string (deprecated). */
     digest(input: string): string

     /** Digest bytes using optional algorithm; returns raw bytes. */
     digest(algorithm: string | null, input: Bytes): Bytes

     /** Complete digest and return bytes. */
     digest(): Bytes

     /** Digest the supplied Bytes and return bytes. */
     digestBytes(input: Bytes): Bytes

     /** Update internal digest state with bytes. */
     updateBytes(input: Bytes): void
 }
 ```
