# dw.crypto.Cipher

## Overview
Provides access to encryption and decryption services using Java Cryptography Architecture (JCA). Supports symmetric and asymmetric algorithms, modes, and paddings.

## Description
This class allows access to encryption services offered through the Java Cryptography Architecture (JCA). At this time the implementation of the encryption/decryption methods is based on the default JCE provider of the JDK.

dw.crypto.Cipher is intentionally an Adapter to the full cryptography power supplied in the security provider implementation.

Note: this class handles sensitive security-related data. Pay special attention to PCI DSS v3 requirements 2, 4, and 12.

```
Object
  dw.crypto.Cipher
```

```ts
declare class Cipher  {
	/**
	 * Strings containing keys, plain texts, cipher texts etc. are internally converted into byte arrays using this encoding (currently UTF8).
	 */
	static CHAR_ENCODING: 'UTF8'

	constructor()

	/**
	 * Decrypts the passed Base-64 encoded message using the passed key and applying the transformations described by the passed parameters.
	 * @param base64Msg - The base64 encoded cipher bytes
	 * @param key - When using a symmetric cryptographic algorithm, use the same key to encrypt and decrypt
	 * @param transformation - The transformation in "algorithm/mode/padding" format
	 * @param saltOrIV - Initialization value appropriate for the algorithm
	 * @param iterations - The number of passes to make when turning a passphrase into a key
	 * @returns The original plaintext message
	 */
	decrypt(base64Msg: string, key: string, transformation: string, saltOrIV: string, iterations: number): string

	/**
	 * Alternative method to decrypt(), which allows to use a key in the keystore for the decryption. Only asymmetric algorithms supported.
	 * @param base64Msg - The base64 encoded cipher bytes
	 * @param privateKey - A reference to a private key in the key store
	 * @param transformation - The transformation in "algorithm/mode/padding" format
	 * @param saltOrIV - Initialization value appropriate for the algorithm
	 * @param iterations - The number of passes to make when turning a passphrase into a key
	 * @returns The original plaintext message
	 */
	decrypt(base64Msg: string, privateKey: KeyRef, transformation: string, saltOrIV: string, iterations: number): string

	/**
	 * Decrypts the passed bytes using the specified key and applying the transformations described by the specified parameters. Lower-level decryption API.
	 * @param encryptedBytes - The bytes to decrypt
	 * @param key - The key to use for decryption
	 * @param transformation - The transformation used to originally encrypt
	 * @param saltOrIV - The salt or IV to use
	 * @param iterations - The iterations to use
	 * @returns The decrypted bytes
	 */
	decryptBytes(encryptedBytes: Bytes, key: string, transformation: string, saltOrIV: string, iterations: number): Bytes

	/**
	 * Alternative method to decryptBytes(), which allows to use a key in the keystore for the decryption.
	 * @param encryptedBytes - The bytes to decrypt
	 * @param privateKey - A reference to a private key in the key store
	 * @param transformation - The transformation used to originally encrypt
	 * @param saltOrIV - The salt or IV to use
	 * @param iterations - The iterations to use
	 * @returns The decrypted bytes
	 */
	decryptBytes(encryptedBytes: Bytes, privateKey: KeyRef, transformation: string, saltOrIV: string, iterations: number): Bytes

	/**
	 * Encrypt the passed message by using the specified key and applying the transformations described by the specified parameters.
	 * @param message - The message to encrypt
	 * @param key - The key to use for encryption
	 * @param transformation - The transformation in "algorithm/mode/padding" format
	 * @param saltOrIV - Initialization value appropriate for the algorithm
	 * @param iterations - The number of passes to make when turning a passphrase into a key
	 * @returns The base64 encoded encrypted message
	 */
	encrypt(message: string, key: string, transformation: string, saltOrIV: string, iterations: number): string

	/**
	 * Alternative method to encrypt(), which allows you to use a key in the keystore for encryption.
	 * @param message - The message to encrypt
	 * @param publicKey - A reference to a certificate/public key in the key store
	 * @param transformation - The transformation in "algorithm/mode/padding" format
	 * @param saltOrIV - Initialization value appropriate for the algorithm
	 * @param iterations - The number of passes to make when turning a passphrase into a key
	 * @returns The base64 encoded encrypted message
	 */
	encrypt(message: string, publicKey: CertificateRef, transformation: string, saltOrIV: string, iterations: number): string

	/**
	 * Lower-level encryption API. Encrypts the passed bytes using the specified key and applying the transformations described by the specified parameters.
	 * @param messageBytes - The bytes to encrypt
	 * @param key - The key to use for encryption
	 * @param transformation - The transformation in "algorithm/mode/padding" format
	 * @param saltOrIV - Initialization value appropriate for the algorithm
	 * @param iterations - The number of passes to make when turning a passphrase into a key
	 * @returns The encrypted bytes
	 */
	encryptBytes(messageBytes: Bytes, key: string, transformation: string, saltOrIV: string, iterations: number): Bytes

	/**
	 * Alternative method to encryptBytes(), which allows to use a key in the keystore for the encryption.
	 * @param messageBytes - The bytes to encrypt
	 * @param publicKey - A reference to a certificate/public key in the key store
	 * @param transformation - The transformation in "algorithm/mode/padding" format
	 * @param saltOrIV - Initialization value appropriate for the algorithm
	 * @param iterations - The number of passes to make when turning a passphrase into a key
	 * @returns The encrypted bytes
	 */
	encryptBytes(messageBytes: Bytes, publicKey: CertificateRef, transformation: string, saltOrIV: string, iterations: number): Bytes
}
```
