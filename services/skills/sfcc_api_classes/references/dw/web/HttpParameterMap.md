# dw.web.HttpParameterMap

## Overview
A map of HTTP parameters with support for parameter retrieval, filtering, and multipart form processing.

## Description
A map of HTTP parameters.

```
Object
  dw.web.HttpParameterMap
```

```ts
declare class HttpParameterMap  {
	/**
	 * The number of parameters in this http parameter map.
	 */
	readonly parameterCount: number

	/**
	 * A collection of all parameter names.
	 */
	readonly parameterNames: Set

	/**
	 * The HTTP request body as string (e.g. useful for XML posts). A body is only returned if the request is a POST or PUT request and was not sent with "application/x-www-form-urlencoded" encoding. If the request was sent with that encoding it is interpreted as form data and the body will be empty.
	 */
	readonly requestBodyAsString: string

	/**
	 * Returns the http parameter for the given key or an empty http parameter if no parameter is defined for that key.
	 * @param name - The key whose associated http parameter is to be returned
	 * @returns The http parameter or an empty http parameter
	 */
	get(name: Object): HttpParameter

	/**
	 * Returns the number of parameters in this http parameter map.
	 * @returns The number of parameters
	 */
	getParameterCount(): number

	/**
	 * Returns a sub-map containing all parameters that start with the given prefix. The prefix will be removed from the parameter names in the returned sub-map.
	 * @param prefix - The prefix to use when creating the sub-map
	 * @returns The sub-map containing the target parameters
	 */
	getParameterMap(prefix: string): HttpParameterMap

	/**
	 * Returns a collection of all parameter names.
	 * @returns A set of all parameter names
	 */
	getParameterNames(): Set

	/**
	 * Returns the HTTP request body as string (e.g. useful for XML posts). A body is only returned if the request is a POST or PUT request and was not sent with "application/x-www-form-urlencoded" encoding.
	 * @returns The http request body
	 */
	getRequestBodyAsString(): string

	/**
	 * Identifies if the parameter has been submitted.
	 * @param key - The parameter to check
	 * @returns True if the parameter has been submitted, false otherwise
	 */
	isParameterSubmitted(key: string): boolean

	/**
	 * Processes a form submission for an HTML form with encoding type "multipart/form-data". Form fields are available via get() without calling this method. Uploaded files need to be processed via the passed callback function.
	 * @param callback - Callback function called for each file upload part in the request
	 * @returns LinkedHashMap of uploaded files
	 */
	processMultipart(callback: Function): LinkedHashMap
}
```
