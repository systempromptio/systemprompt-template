# dw.svc.HTTPServiceDefinition

## Overview
HTTP service definition (deprecated).

## Description
Represents an HTTP Service Definition. The HTTP Service will use the return value of the createRequest callback as the request body (if supported by the HTTP method). If this is an array of non-null HTTPRequestPart objects, then a multi-part request will be formed. Otherwise the object is converted to a String and used. See also XML.toXMLString() and JSON.stringify(Object), which must be explicitly called if needed. No longer available as of version 19.10. This class is only used with the deprecated ServiceRegistry. Use the LocalServiceRegistry instead, which allows configuration on the HTTPService directly.

## All Known Subclasses
HTTPFormServiceDefinition

```
Object
  dw.svc.ServiceDefinition
    dw.svc.HTTPServiceDefinition
```

```ts
declare class HTTPServiceDefinition extends ServiceDefinition {
	/**
	 * The authentication type.
	 */
	authentication: string

	/**
	 * The caching time to live value.
	 */
	cachingTTL: number

	/**
	 * The request body encoding to declare.
	 */
	encoding: string

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
	 * @returns this HTTP Service Definition.
	 */
	addHeader(name: string, val: string): HTTPServiceDefinition

	/**
	 * Adds a query parameter that will be appended to the URL.
	 * @param name - Parameter name.
	 * @param val - Parameter value.
	 * @returns this HTTP Service Definition.
	 */
	addParam(name: string, val: string): HTTPServiceDefinition

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
	 * Returns the request body encoding to declare.
	 * @returns Request encoding.
	 */
	getEncoding(): string

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
	 * @returns this HTTP Service Definition.
	 */
	setAuthentication(authentication: string): HTTPServiceDefinition

	/**
	 * Enables caching for GET requests. This only caches status codes 2xx with a content length and size of less than 50k that are not immediately written to file. The URL and the user name are used as cache keys. Cache control information sent by the remote server is ignored.
	 * @param ttl - The time to live for the cached content in seconds. A value of 0 or less disables caching.
	 */
	setCachingTTL(ttl: number): HTTPServiceDefinition

	/**
	 * Sets the encoding of the request body (if any). The default value is UTF-8.
	 * @param encoding - Encoding of the request body.
	 * @returns this HTTP Service Definition.
	 */
	setEncoding(encoding: string): HTTPServiceDefinition

	/**
	 * Sets the output file in which to write the HTTP response body. The default behavior is to not write a file.
	 * @param outFile - Output file, or null to disable.
	 * @returns this HTTP Service Definition.
	 */
	setOutFile(outFile: File): HTTPServiceDefinition

	/**
	 * Sets the HTTP request method. Valid values include GET, PUT, POST, and DELETE. The default value is POST.
	 * @param requestMethod - HTTP request method.
	 * @returns this HTTP Service Definition.
	 */
	setRequestMethod(requestMethod: string): HTTPServiceDefinition
}
```
