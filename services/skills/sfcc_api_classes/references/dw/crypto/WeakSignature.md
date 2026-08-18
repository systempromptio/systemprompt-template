 # dw.crypto.WeakSignature

 ## Overview
 Adapter exposing legacy signature algorithms (SHA1withRSA) for compatibility with older systems.

 ## Description
 Provides signing and verification helpers using deprecated digest algorithms. Use `Signature` for modern, secure operations. Methods accept Base64-encoded strings or raw bytes and may reference keys by `KeyRef` or `CertificateRef`.

 ```ts
 declare class WeakSignature  {
     constructor()

     /** Check support for a digest algorithm. */
     isDigestAlgorithmSupported(digestAlgorithm: string): boolean

     /** Sign Base64 content with Base64 private key; returns Base64 signature. */
     sign(contentToSign: string, privateKey: string, digestAlgorithm: string): string

     /** Sign using a keystore KeyRef; returns Base64 signature. */
     sign(contentToSign: string, privateKey: KeyRef, digestAlgorithm: string): string

     /** Sign raw bytes and return signature bytes. */
     signBytes(contentToSign: Bytes, privateKey: string, digestAlgorithm: string): Bytes

     /** Sign raw bytes with KeyRef; returns signature bytes. */
     signBytes(contentToSign: Bytes, privateKey: KeyRef, digestAlgorithm: string): Bytes

     /** Verify signature provided as bytes against bytes content. */
     verifyBytesSignature(signature: Bytes, contentToVerify: Bytes, publicKey: string, digestAlgorithm: string): boolean

     /** Verify bytes signature using CertificateRef. */
     verifyBytesSignature(signature: Bytes, contentToVerify: Bytes, certificate: CertificateRef, digestAlgorithm: string): boolean

     /** Verify string signature against string content using public key. */
     verifySignature(signature: string, contentToVerify: string, publicKey: string, digestAlgorithm: string): boolean

     /** Verify string signature using CertificateRef. */
     verifySignature(signature: string, contentToVerify: string, certificate: CertificateRef, digestAlgorithm: string): boolean
 }
 ```
