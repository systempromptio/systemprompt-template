 # dw.crypto.WeakCipher

 ## Overview
 Lightweight compatibility wrapper exposing legacy encryption/decryption helpers for deprecated algorithms and key handling.

 ## Description
 Provides convenience methods to encrypt/decrypt strings and byte arrays using simple keys or keystore references. Intended for backward compatibility; use `Cipher` for modern encryption needs. Handles Base64-encoded input/outputs for string variants.

 ```ts
 declare class WeakCipher  {
     /**
      * Decrypts a Base64-encoded message using a passphrase and transformation.
      * @param base64Msg Base64-encoded encrypted message
      * @param key Passphrase or key string
      * @param transformation Transformation in "algorithm/mode/padding" format
      * @param saltOrIV Salt or initialization vector
      * @param iterations Number of iterations when deriving a key from passphrase
      */
     decrypt(base64Msg: string, key: string, transformation: string, saltOrIV: string, iterations: number): string

     /**
      * Decrypts bytes using passphrase-derived key.
      */
     decryptBytes(encryptedBytes: Bytes, key: string, transformation: string, saltOrIV: string, iterations: number): Bytes

     /**
      * Decrypts bytes using a private key reference from the keystore.
      */
     decryptBytes(encryptedBytes: Bytes, privateKey: KeyRef, transformation: string, saltOrIV: string, iterations: number): Bytes

     /**
      * Encrypts a message string and returns Base64-encoded cipher text.
      */
     encrypt(message: string, key: string, transformation: string, saltOrIV: string, iterations: number): string

     /**
      * Encrypts bytes and returns encrypted bytes.
      */
     encryptBytes(messageBytes: Bytes, key: string, transformation: string, saltOrIV: string, iterations: number): Bytes

     /**
      * Alternate encrypt/decrypt overloads accepting CertificateRef or KeyRef for keystore keys.
      */
     // preserves all overloads shown above
 }
 ```
