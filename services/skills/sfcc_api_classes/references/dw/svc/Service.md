# dw.svc.Service

## Overview
Base class of Services.

## Description
A service represents a call-specific configuration. Any configuration set here is local to the currently executing call.

## All Known Subclasses
FTPService, HTTPFormService, HTTPService, SOAPService

```ts
declare class Service  {
	/**
	 * The Service Configuration.
	 * @readonly
	 */
	configuration: ServiceConfig

	/**
	 * The ID of the currently associated Credential.
	 */
	credentialID: string

	/**
	 * Whether this service is executing in mock mode.
	 */
	mock: boolean

	/**
	 * The object returned by createRequest.
	 * @readonly
	 */
	requestData: Object

	/**
	 * The object returned by the service. Only useful after call() completes, and is the same as the object inside the Result.
	 * @readonly
	 */
	response: Object

	/**
	 * Whether this service will throw an error when encountering a problem.
	 */
	throwOnError: boolean

	/**
	 * The current URL, excluding any custom query parameters.
	 */
	URL: string

	/**
	 * Invokes the service.
	 * @param args - Arguments to pass. If there is a single argument and that argument is an array, each item in the array will become a separate argument. This can be avoided by explicitly forming a List, enclosing the array in another array, or sending a second argument.
	 */
	call(...args: Object[]): Result

	/**
	 * Returns the Service Configuration.
	 */
	getConfiguration(): ServiceConfig

	/**
	 * Returns the ID of the currently associated Credential.
	 */
	getCredentialID(): string

	/**
	 * Returns the object returned by createRequest.
	 */
	getRequestData(): Object

	/**
	 * Returns the object returned by the service. Only useful after call() completes.
	 */
	getResponse(): Object

	/**
	 * Returns the current URL, excluding any custom query parameters.
	 */
	getURL(): string

	/**
	 * Returns whether this service is executing in mock mode.
	 */
	isMock(): boolean

	/**
	 * Returns whether this service will throw an error when encountering a problem.
	 */
	isThrowOnError(): boolean

	/**
	 * Overrides the Credential by the credential object with the given ID. If the URL is also overridden, that URL will continue to override the URL in this credential.
	 * @param id - Credential ID. It must exist.
	 */
	setCredentialID(id: string): Service

	/**
	 * Forces the mock mode to be enabled.
	 */
	setMock(): Service

	/**
	 * Forces a Service to throw an error when there is a problem instead of returning a Result with non-OK status.
	 */
	setThrowOnError(): Service

	/**
	 * Overrides the URL to the given value. Any query parameters (if applicable) will be appended to this URL.
	 * @param url - Force the URL to the given value.
	 */
	setURL(url: string): Service
}
```
