# dw.rpc.SOAPUtil

## Overview
Utility class for SOAP web services providing WS-Security features including signing, encryption, and authentication.

## Description
Provides methods for setting SOAP headers and constants for WS-Security configuration. Supports constructing HashMap configurations for ws-security actions such as signing and encryption. Handles sensitive security-related data per PCI DSS v3 requirements 2, 4, and 12. Deprecated; use webreferences2 and WSUtil instead.

```ts
declare class SOAPUtil  {
	/**
	 * WS-Security action property name
	 */
	static WS_ACTION: 'action'
	
	/**
	 * WS-Security encryption: the encryption/decryption keystore alias name
	 */
	static WS_ENC_PROP_KEYSTORE_ALIAS: '__EncryptionPropKeystoreAlias'
	
	/**
	 * WS-Security encryption: the encryption/decryption keystore password
	 */
	static WS_ENC_PROP_KEYSTORE_PW: '__EncryptionPropKeystorePassword'
	
	/**
	 * WS-Security encryption: the encryption/decryption keystore type (jks or pkcs12), default is jks
	 */
	static WS_ENC_PROP_KEYSTORE_TYPE: '__EncryptionPropKeystoreType'
	
	/**
	 * WS-Security action: encrypt the message
	 */
	static WS_ENCRYPT: 'Encrypt'
	
	/**
	 * WS-Security encryption: defines which parts of the request are encrypted
	 */
	static WS_ENCRYPTION_PARTS: 'encryptionParts'
	
	/**
	 * WS-Security encryption: the user's name for encryption
	 */
	static WS_ENCRYPTION_USER: 'encryptionUser'
	
	/**
	 * WS-Security action: no security
	 */
	static WS_NO_SECURITY: 'NoSecurity'
	
	/**
	 * WS-Security password type parameter for UsernameToken action
	 */
	static WS_PASSWORD_TYPE: 'passwordType'
	
	/**
	 * WS-Security password of type digest
	 */
	static WS_PW_DIGEST: 'PasswordDigest'
	
	/**
	 * WS-Security password of type text (clear text)
	 */
	static WS_PW_TEXT: 'PasswordText'
	
	/**
	 * A secrets map with username/password entries for password callback
	 */
	static WS_SECRETS_MAP: '__SecretsMap'
	
	/**
	 * WS-Security signature: sets the signature digest algorithm
	 */
	static WS_SIG_DIGEST_ALGO: 'signatureDigestAlgorithm'
	
	/**
	 * WS-Security signature: the signature keystore alias name
	 */
	static WS_SIG_PROP_KEYSTORE_ALIAS: '__SignaturePropKeystoreAlias'
	
	/**
	 * WS-Security signature: the signature keystore password
	 */
	static WS_SIG_PROP_KEYSTORE_PW: '__SignaturePropKeystorePassword'
	
	/**
	 * WS-Security: the signature keystore type (jks or pkcs12), default is jks
	 */
	static WS_SIG_PROP_KEYSTORE_TYPE: '__SignaturePropKeystoreType'
	
	/**
	 * WS-Security action: sign the message
	 */
	static WS_SIGNATURE: 'Signature'
	
	/**
	 * WS-Security signature: defines which parts of the request are signed
	 */
	static WS_SIGNATURE_PARTS: 'signatureParts'
	
	/**
	 * WS-Security signature: the user's name for signature
	 */
	static WS_SIGNATURE_USER: 'signatureUser'
	
	/**
	 * WS-Security action: add a timestamp to the security header
	 */
	static WS_TIMESTAMP: 'Timestamp'
	
	/**
	 * WS-Security user name
	 */
	static WS_USER: 'user'
	
	/**
	 * WS-Security action: add a UsernameToken identification
	 */
	static WS_USERNAME_TOKEN: 'UsernameToken'
	
	/**
	 * Returns an HTTP request header property value using the specified key
	 * @param svc - service stub returned from getService()
	 * @param key - the header property key
	 * @returns HTTP request header property value or null
	 */
	static getHTTPRequestHeader(svc: Object, key: string): string
	
	/**
	 * Returns an HTTP response header property value using the specified key
	 * @param svc - service stub returned from getService()
	 * @param key - the header property key
	 * @returns HTTP response header property value or null
	 */
	static getHTTPResponseHeader(svc: Object, key: string): string
	
	/**
	 * Sets a new SOAPHeaderElement in the SOAP request with the namespace of the XML content
	 * @param svc - service stub returned from getService()
	 * @param xml - string with arbitrary XML content
	 */
	static setHeader(svc: Object, xml: string): void
	
	/**
	 * Sets a new SOAPHeaderElement in the SOAP request with the namespace of the XML content
	 * @param svc - service stub returned from getService()
	 * @param xml - string with arbitrary XML content
	 * @param mustUnderstand - SOAP mustUnderstand flag
	 */
	static setHeader(svc: Object, xml: string, mustUnderstand: boolean): void
	
	/**
	 * Creates a new SOAPHeaderElement with the name and namespace and places the given XML into it
	 * @param svc - service stub returned from getService()
	 * @param namespace - namespace URI
	 * @param name - element name
	 * @param xml - string with arbitrary XML content
	 */
	static setHeader(svc: Object, namespace: string, name: string, xml: string): void
	
	/**
	 * Creates a new SOAPHeaderElement with the name and namespace and places the given XML into it
	 * @param svc - service stub returned from getService()
	 * @param namespace - namespace URI
	 * @param name - element name
	 * @param xml - string with arbitrary XML content
	 * @param mustUnderstand - SOAP mustUnderstand flag
	 */
	static setHeader(svc: Object, namespace: string, name: string, xml: string, mustUnderstand: boolean): void
	
	/**
	 * Creates a new SOAPHeaderElement with the name and namespace and places the given XML into it
	 * @param svc - service stub returned from getService()
	 * @param namespace - namespace URI
	 * @param name - element name
	 * @param xml - string with arbitrary XML content
	 * @param mustUnderstand - SOAP mustUnderstand flag
	 * @param actor - SOAP actor URI
	 */
	static setHeader(svc: Object, namespace: string, name: string, xml: string, mustUnderstand: boolean, actor: string): void
	
	/**
	 * Creates a new SOAPHeaderElement with the name and namespace and places the given XML into it
	 * @param svc - service stub returned from getService()
	 * @param namespace - namespace URI
	 * @param name - element name
	 * @param xml - object with XML content
	 */
	static setHeader(svc: Object, namespace: string, name: string, xml: Object): void
	
	/**
	 * Creates a new SOAPHeaderElement with the name and namespace and places the given XML into it
	 * @param svc - service stub returned from getService()
	 * @param namespace - namespace URI
	 * @param name - element name
	 * @param xml - object with XML content
	 * @param mustUnderstand - SOAP mustUnderstand flag
	 */
	static setHeader(svc: Object, namespace: string, name: string, xml: Object, mustUnderstand: boolean): void
	
	/**
	 * Creates a new SOAPHeaderElement with the name and namespace and places the given XML into it
	 * @param svc - service stub returned from getService()
	 * @param namespace - namespace URI
	 * @param name - element name
	 * @param xml - object with XML content
	 * @param mustUnderstand - SOAP mustUnderstand flag
	 * @param actor - SOAP actor URI
	 */
	static setHeader(svc: Object, namespace: string, name: string, xml: Object, mustUnderstand: boolean, actor: string): void
	
	/**
	 * Sets an HTTP request header property using the specified key and value
	 * @param svc - service stub returned from getService()
	 * @param key - header property key
	 * @param value - header property value
	 */
	static setHTTPRequestHeader(svc: Object, key: string, value: string): void
	
	/**
	 * Sets the WS-Security configuration for the request and response
	 * @param svc - service stub returned from getService()
	 * @param requestConfigMap - configuration map for request
	 * @param responseConfigMap - configuration map for response
	 */
	static setWSSecurityConfig(svc: Object, requestConfigMap: Object, responseConfigMap: Object): void
}
```
