# dw.svc.HTTPService

## Overview
Provides HTTP service capabilities including request configuration, authentication, and response handling.

## Description
The HTTP Service will use the return value of the createRequest callback as the request body (if supported by the HTTP method). If this is an array of non-null HTTPRequestPart objects, then a multi-part request will be formed. Otherwise the object is converted to a String and used. See also XML.toXMLString() and JSON.stringify(Object), which must be explicitly called if needed.

## All Known Subclasses
HTTPFormService

```
Object
  dw.svc.Service
    dw.svc.HTTPService
```

```ts
declare class HTTPService extends Service {
	/**
	 * The authentication type.
	 */
	authentication: string

	/**
	 * The caching time to live value.
	 */
	cachingTTL: number

	/**
	 * The underlying HTTP client object.
	 * @readonly
	 */
	readonly client: HTTPClient

	/**
	 * The request body encoding to declare.
	 */
	encoding: string

	/**
	 * Determines whether host name verification is enabled.
	 */
	hostNameVerification: boolean

	/**
	 * Gets the identity used for mutual TLS (mTLS).
	 */
	identity: KeyRef

	/**
	 * The output file, or null if there is none.
	 */
	outFile: File

	/**
	 * The request method.
	 */
	requestMethod: string

	/**
	 * Adds an HTTP Header.
	 * @param name - Header name.
	 * @param val - Header value.
	 * @returns this HTTP Service.
	 */
	addHeader(name: string, val: string): HTTPService

	/**
	 * Adds a query parameter that will be appended to the URL.
	 * @param name - Parameter name.
	 * @param val - Parameter value.
	 * @returns this HTTP Service.
	 */
	addParam(name: string, val: string): HTTPService

	/**
	 * Returns the authentication type.
	 * @returns Authentication type.
	 */
	getAuthentication(): string

	/**
	 * Returns the caching time to live value.
	 * @returns The caching time to live value in seconds.
	 */
	getCachingTTL(): number

	/**
	 * Returns the underlying HTTP client object.
	 * @returns HTTP client object.
	 */
	getClient(): HTTPClient

	/**
	 * Returns the request body encoding to declare.
	 * @returns Request encoding.
	 */
	getEncoding(): string

	/**
	 * Determines whether host name verification is enabled.
	 * @returns true if verification is enabled, false otherwise
	 */
	getHostNameVerification(): boolean

	/**
	 * Gets the identity used for mutual TLS (mTLS).
	 * @returns Reference to the private key, or null if not configured
	 */
	getIdentity(): KeyRef

	/**
	 * Returns the output file, or null if there is none.
	 * @returns Output file or null.
	 */
	getOutFile(): File

	/**
	 * Returns the request method.
	 * @returns HTTP Request method.
	 */
	getRequestMethod(): string

	/**
	 * Sets the type of authentication. Valid values include "BASIC" and "NONE". The default value is BASIC.
	 * @param authentication - Type of authentication.
	 * @returns this HTTP Service.
	 */
	setAuthentication(authentication: string): HTTPService

	/**
	 * Enables caching for GET requests. This only caches status codes 2xx with a content length and size of less than 50k that are not immediately written to file. The URL and the user name are used as cache keys. Cache control information sent by the remote server is ignored.
	 * @param ttl - The time to live for the cached content in seconds. A value of 0 disables caching.
	 */
	setCachingTTL(ttl: number): HTTPService

	/**
	 * Sets the encoding of the request body (if any). The default value is UTF-8.
	 * @param encoding - Encoding of the request body.
	 * @returns this HTTP Service.
	 */
	setEncoding(encoding: string): HTTPService

	/**
	 * Sets whether certificate host name verification is enabled. The default value is true. Set it to false to disable host name verification.
	 * @param enable - true to enable host name verification or false to disable it.
	 * @returns this HTTP Service.
	 */
	setHostNameVerification(enable: boolean): HTTPService

	/**
	 * Sets the identity (private key) to use when mutual TLS (mTLS) is configured. If this is not set and mTLS is used then the private key will be chosen from the key store based on the host name. If this is set to a reference named "__NONE__" then no private key will be used even if one is requested by the remote server.
	 * @param keyRef - Reference to the private key
	 */
	setIdentity(keyRef: KeyRef): HTTPService

	/**
	 * Sets the output file in which to write the HTTP response body. The default behavior is to not write a file.
	 * @param outFile - Output file, or null to disable.
	 * @returns this HTTP Service.
	 */
	setOutFile(outFile: File): HTTPService

	/**
	 * Sets the HTTP request method. Valid values include GET, PUT, POST, and DELETE. The default value is POST.
	 * @param requestMethod - HTTP request method.
	 * @returns this HTTP Service.
	 */
	setRequestMethod(requestMethod: string): HTTPService
}
```
