# dw.system.RESTErrorResponse

## Overview
REST error response compliant with RFC 9457, created via RESTResponseMgr.

## Description
Represents a REST error response compliant with RFC 9457. Instantiated only via `createError` methods in RESTResponseMgr. Custom attributes render with "c_" prefix.

```ts
declare class RESTErrorResponse  {
	/**
	 * Custom attributes associated with the error response object, stored for the lifetime of the response.
	 * @readonly
	 */
	readonly custom: CustomAttributes

	/**
	 * Returns all custom attributes associated with the error response object.
	 */
	getCustom(): CustomAttributes

	/**
	 * Sends the RESTErrorResponse object as HTTP error response to the client, adhering to RFC 9457. Sets "Content-Type" to "application/problem+json", HTTP status code to statusCode, and constructs body from type, title, detail and custom attributes. Custom attributes are rendered with "c_" prefix.
	 * @throws {IllegalStateException} If the RESTErrorResponse object is already rendered.
	 * @throws {Exception} If there is an error while serializing the RESTErrorResponse object.
	 */
	render(): void
}
```
