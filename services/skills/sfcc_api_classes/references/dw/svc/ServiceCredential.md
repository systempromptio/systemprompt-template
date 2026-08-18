# dw.svc.ServiceCredential

## Overview
Configuration object for Service Credentials.

## Description
Stores service credentials including URL, user, and password. Extends EncryptedObject for secure password handling. Provides access to plain text password and deprecated encryption method.

```ts
declare class ServiceCredential extends EncryptedObject {
	/**
	 * Constant for specification of the public key encryption algorithm RSA.
	 * @deprecated Use Cipher to encrypt data as needed.
	 */
	static ENCRYPTION_ALGORITHM_RSA: 'RSA'

	/**
	 * The unique Credential ID.
	 * @readonly
	 */
	ID: string

	/**
	 * The Password in plain text.
	 * @readonly
	 */
	password: string

	/**
	 * The URL.
	 * @readonly
	 */
	URL: string

	/**
	 * The User ID.
	 * @readonly
	 */
	user: string

	/**
	 * Encrypts the password with the given algorithm and the public key from a certificate in the keystore. Returns base64-encoded representation of the result.
	 * @param algorithm - The algorithm for encryption. Currently only "RSA" is supported.
	 * @param publicKey - A reference to a trusted certificate entry containing the public key in the keystore.
	 * @deprecated Use Cipher to encrypt data as needed.
	 */
	getEncryptedPassword(algorithm: string, publicKey: CertificateRef): string

	/**
	 * Returns the unique Credential ID.
	 */
	getID(): string

	/**
	 * Returns the Password in plain text.
	 */
	getPassword(): string

	/**
	 * Returns the URL.
	 */
	getURL(): string

	/**
	 * Returns the User ID.
	 */
	getUser(): string
}
```
