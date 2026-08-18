# dw.crypto.CertificateUtils

## Overview
Utilities for managing certificates and keys in Salesforce B2C Commerce. Provides static methods to retrieve, encode, and parse X.509 certificates and public keys.

## Description
CertificateUtils offers static utility methods for handling X.509 certificates and public keys, including retrieval from references, encoding to base64 DER, and parsing from encoded formats or JWK. Supports both certificate and key references.

```ts
declare class CertificateUtils  {
    /**
     * Gets the certificate from the given certificate reference.
     * @param certificateRef The certificate reference.
     * @returns The X509Certificate.
     * @throws Exception if the reference is invalid or does not refer to an X.509 certificate.
     */
    static getCertificate(certificateRef: CertificateRef): X509Certificate;

    /**
     * Gets the public certificate from the given private key reference.
     * @param keyRef The key reference.
     * @returns The X509Certificate.
     * @throws Exception if the reference is invalid or there is no X.509 certificate.
     */
    static getCertificate(keyRef: KeyRef): X509Certificate;

    /**
     * Encodes the certificate to the base64-encoded DER format.
     * @param certificateRef The certificate to encode.
     * @returns Base64-encoded DER certificate.
     */
    static getEncodedCertificate(certificateRef: CertificateRef): string;

    /**
     * Gets the public key from the given certificate reference, exported in base64-encoded X.509 SubjectPublicKeyInfo format.
     * @param certificateRef The certificate reference with the public key to encode.
     * @returns The encoded public key.
     */
    static getEncodedPublicKey(certificateRef: CertificateRef): string;

    /**
     * Parses the certificate from the base64-encoded DER format.
     * @param certificate The encoded certificate.
     * @returns Reference to the parsed certificate.
     */
    static parseEncodedCertificate(certificate: string): CertificateRef;

    /**
     * Parses the public key from the given key in X.509 SubjectPublicKeyInfo format.
     * @param algorithm The public key algorithm, either 'EC' or 'RSA'.
     * @param encodedKey The encoded key.
     * @returns Reference to the public key.
     */
    static parseEncodedPublicKey(algorithm: string, encodedKey: string): CertificateRef;

    /**
     * Parses the public key from the given base64-encoded JWK string. Only RSA and EC keys are supported.
     * @param jwk Encoded JWK.
     * @returns Reference to the public key.
     */
    static parsePublicKeyFromJWK(jwk: string): CertificateRef;
}
```
