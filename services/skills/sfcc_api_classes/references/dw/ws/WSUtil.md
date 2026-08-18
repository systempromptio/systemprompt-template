# dw.ws.WSUtil

## Overview
Utility class for performing SOAP-based operations for Web Services.

## Description
Provides methods for setting SOAP headers and constants for well-known header names. Used to set connection and request timeout values for Web Service calls. Works with WebReference2 for web service operations.

```ts
declare class WSUtil  {
	/**
	 * X.509 Certificate is included in message as Base-64 encoded BinarySecurityToken.
	 */
	static KEY_ID_TYPE_DIRECT_REFERENCE: 'DirectReference'

	/**
	 * Key Identifier method for Encryption - references EncryptedKey Element rather than public key.
	 */
	static KEY_ID_TYPE_ENC_KEY_SHA1: 'EncryptedKeySHA1'

	/**
	 * Issuer Name and Serial Number of X.509 Certificate included in KeyInfo Element.
	 */
	static KEY_ID_TYPE_ISSUE_SERIAL: 'IssuerSerial'

	/**
	 * Certificate referenced via Base-64 encoding of Subject Key Identifier.
	 */
	static KEY_ID_TYPE_SKI_IDENTIFIER: 'SKIKeyIdentifier'

	/**
	 * Certificate referenced via SHA-1 Thumbprint. Certificate may or may not be included.
	 */
	static KEY_ID_TYPE_THUMBPRINT: 'Thumbprint'

	/**
	 * Certificate included directly in KeyInfo element.
	 */
	static KEY_ID_TYPE_X509_KEY_IDENTIFIER: 'X509KeyIdentifier'

	/**
	 * WS-Security action property name. Allowed values: WS_NO_SECURITY, WS_TIMESTAMP, WS_ENCRYPT, WS_SIGNATURE, WS_USERNAME_TOKEN or space-separated list.
	 */
	static WS_ACTION: 'action'

	/**
	 * Defines which key identifier type to use for encryption. Default: KEY_ID_TYPE_ISSUE_SERIAL.
	 */
	static WS_ENC_KEY_ID: 'encryptionKeyIdentifier'

	/**
	 * WS-Security Encryption: keystore alias name.
	 */
	static WS_ENC_PROP_KEYSTORE_ALIAS: '__EncryptionPropKeystoreAlias'

	/**
	 * WS-Security Encryption: keystore password.
	 */
	static WS_ENC_PROP_KEYSTORE_PW: '__EncryptionPropKeystorePassword'

	/**
	 * WS-Security Encryption: keystore type (jks, pkcs12, or managed). Default: jks.
	 */
	static WS_ENC_PROP_KEYSTORE_TYPE: '__EncryptionPropKeystoreType'

	/**
	 * WS-Security action: Encrypt the message.
	 */
	static WS_ENCRYPT: 'Encrypt'

	/**
	 * WS-Security Encryption: Defines which parts of request shall be encrypted.
	 */
	static WS_ENCRYPTION_PARTS: 'encryptionParts'

	/**
	 * WS-Security Encryption: user's name for encryption.
	 */
	static WS_ENCRYPTION_USER: 'encryptionUser'

	/**
	 * WS-Security action: No security.
	 */
	static WS_NO_SECURITY: 'NoSecurity'

	/**
	 * WS-Security password type: Parameter for UsernameToken action to define encoding. Values: PW_DIGEST or PW_TEXT.
	 */
	static WS_PASSWORD_TYPE: 'passwordType'

	/**
	 * WS-Security password type "digest": Use password digest.
	 */
	static WS_PW_DIGEST: 'PasswordDigest'

	/**
	 * WS-Security password type "text": Send password in clear.
	 */
	static WS_PW_TEXT: 'PasswordText'

	/**
	 * Secrets map with username and password entries for password callback object.
	 */
	static WS_SECRETS_MAP: '__SecretsMap'

	/**
	 * WS-Security Signature: Defines signature digest algorithm.
	 */
	static WS_SIG_DIGEST_ALGO: 'signatureDigestAlgorithm'

	/**
	 * Defines which key identifier type to use for signature. Default: KEY_ID_TYPE_DIRECT_REFERENCE.
	 */
	static WS_SIG_KEY_ID: 'signatureKeyIdentifier'

	/**
	 * WS-Security Signature: keystore alias name.
	 */
	static WS_SIG_PROP_KEYSTORE_ALIAS: '__SignaturePropKeystoreAlias'

	/**
	 * WS-Security Signature: keystore password.
	 */
	static WS_SIG_PROP_KEYSTORE_PW: '__SignaturePropKeystorePassword'

	/**
	 * WS-Security Signature: keystore type (jks, pkcs12, or managed). Default: jks.
	 */
	static WS_SIG_PROP_KEYSTORE_TYPE: '__SignaturePropKeystoreType'

	/**
	 * WS-Security action: Sign the message.
	 */
	static WS_SIGNATURE: 'Signature'

	/**
	 * WS-Security Signature: Defines which parts of request shall be signed.
	 */
	static WS_SIGNATURE_PARTS: 'signatureParts'

	/**
	 * WS-Security Signature: user's name for signature.
	 */
	static WS_SIGNATURE_USER: 'signatureUser'

	/**
	 * WS-Security action: Add timestamp to security header.
	 */
	static WS_TIMESTAMP: 'Timestamp'

	/**
	 * WS-Security user name.
	 */
	static WS_USER: 'user'

	/**
	 * WS-Security action: Add UsernameToken identification.
	 */
	static WS_USERNAME_TOKEN: 'UsernameToken'

	/**
	 * WSUtil constructor.
	 */
	constructor()

	/**
	 * Adds a header element to the SOAP Header. Each header element should be XML with a namespace URI.
	 * @param port - The port
	 * @param xml - The header element XML with namespace URI
	 * @param mustUnderstand - Directs target endpoint to validate payload
	 * @param actor - URI identifying intended recipient
	 */
	static addSOAPHeader(port: Object, xml: Object, mustUnderstand: boolean, actor: string): void

	/**
	 * Adds a header element to the SOAP Header. Each header element should be XML with a namespace URI.
	 * @param port - The port
	 * @param xml - The header element XML as String with namespace URI
	 * @param mustUnderstand - Directs target endpoint to validate payload
	 * @param actor - URI identifying intended recipient
	 */
	static addSOAPHeader(port: Object, xml: string, mustUnderstand: boolean, actor: string): void

	/**
	 * Removes all SOAP header elements from port's request context.
	 * @param port - Port returned from WebReference2 getService methods
	 */
	static clearSOAPHeaders(port: Object): void

	/**
	 * Creates javax.xml.ws.Holder instance wrapping specified element. Used when WSDL operation requires holder for input/output.
	 * @param element - Element to wrap in Holder
	 * @returns The holder
	 */
	static createHolder(element: Object): Object

	/**
	 * Returns connection timeout value for the port.
	 * @param port - Port returned from WebReference2 getService methods
	 * @returns Connection timeout value
	 */
	static getConnectionTimeout(port: Object): number

	/**
	 * Returns HTTP request header property value using specified key. Null if key doesn't represent an HTTP header property.
	 * @param port - Port returned from WebReference2 getService methods
	 * @param key - Header property key
	 * @returns HTTP request header property value or null
	 */
	static getHTTPRequestHeader(port: Object, key: string): string

	/**
	 * Returns value of SOAP request property using specified key. Property keys defined in Port constants.
	 * @param key - The key to use
	 * @param port - Port on which property is set
	 * @returns Property value
	 */
	static getProperty(key: string, port: Object): Object

	/**
	 * Returns read timeout value for request made on specified port. Error thrown if request exceeds timeout.
	 * @param port - Port returned from WebReference2 getService methods
	 * @returns Request timeout value
	 */
	static getRequestTimeout(port: Object): number

	/**
	 * Returns value of response property using specified key.
	 * @param key - The key to use
	 * @param port - Port returned from WebReference2 getService methods
	 * @returns Property value
	 */
	static getResponseProperty(key: string, port: Object): Object

	/**
	 * Returns true if HTTP request may be chunked, false otherwise.
	 * @param port - The port
	 * @returns Whether chunking is allowed
	 */
	static isAllowChunking(port: Object): boolean

	/**
	 * Indicate that HTTP chunked Transfer-Encoding may be used.
	 * @param port - The port
	 * @param allow - Whether to allow chunking
	 */
	static setAllowChunking(port: Object, allow: boolean): void

	/**
	 * Sets connection timeout for the port.
	 * @param timeoutInMilliseconds - Timeout in milliseconds
	 * @param port - Port returned from WebReference2 getService methods
	 */
	static setConnectionTimeout(timeoutInMilliseconds: number, port: Object): void

	/**
	 * Sets HTTP request header property using specified key and value.
	 * @param port - Port returned from WebReference2 getService methods
	 * @param key - Header property key
	 * @param value - Header property value
	 */
	static setHTTPRequestHeader(port: Object, key: string, value: string): void

	/**
	 * Set SOAP request property using specified key and value.
	 * @param key - Property key
	 * @param value - Property value
	 * @param port - Port returned from WebReference2 getService methods
	 */
	static setProperty(key: string, value: Object, port: Object): void

	/**
	 * Sets read timeout value for request made on specified port.
	 * @param timeoutInMilliseconds - Timeout in milliseconds
	 * @param port - Port returned from WebReference2 getService methods
	 */
	static setRequestTimeout(timeoutInMilliseconds: number, port: Object): void

	/**
	 * Set user name and password to use with Basic authentication.
	 * @param userName - User name
	 * @param password - Password
	 * @param port - Port returned from WebReference2 getService methods
	 */
	static setUserNamePassword(userName: string, password: string, port: Object): void

	/**
	 * Set WS-Security configuration for request and response based on defined constants.
	 * @param port - The port
	 * @param requestConfigMap - Request configuration map
	 * @param responseConfigMap - Response configuration map
	 */
	static setWSSecurityConfig(port: Object, requestConfigMap: Object, responseConfigMap: Object): void
}
```
