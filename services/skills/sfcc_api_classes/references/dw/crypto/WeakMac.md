 # dw.crypto.WeakMac

 ## Overview
 Deprecated HMAC wrapper exposing legacy HMAC algorithms (HmacMD5, HmacSHA1) for backward compatibility.

 ## Description
 Implements message authentication code (MAC) operations using deprecated digest algorithms. Use `Mac` for secure, modern HMAC usage. Handles sensitive data; avoid for new development.

 ```ts
 declare class WeakMac  {
     /** Constant for HmacMD5 algorithm */
     static HMAC_MD5: 'HmacMD5'

     /** Constant for HmacSHA1 algorithm */
     static HMAC_SHA_1: 'HmacSHA1'

     /**
      * Construct a WeakMac instance for the named algorithm.
      * @param algorithm algorithm name (HmacMD5|HmacSHA1)
      */
     constructor(algorithm: string)

     /**
      * Compute HMAC for a UTF-8 string input using a passphrase key.
      */
     digest(input: string, key: string): Bytes

     /**
      * Compute HMAC for a UTF-8 string input using binary key.
      */
     digest(input: string, key: Bytes): Bytes

     /**
      * Compute HMAC for binary input using binary key.
      */
     digest(input: Bytes, key: Bytes): Bytes
 }
 ```
