# dw.svc.Result

## Overview
Represents the result of a service call.

## Description
Encapsulates the outcome of a service invocation, including status, response object, error details, and availability information. Provides constants for status codes and unavailability reasons.

```ts
declare class Result  {
	/**
	 * Status indicating a general service error.
	 */
	static ERROR: 'ERROR'

	/**
	 * Status indicating a successful service call.
	 */
	static OK: 'OK'

	/**
	 * Status indicating the service is unavailable. This includes timeouts, rate limits, and remote server issues.
	 */
	static SERVICE_UNAVAILABLE: 'SERVICE_UNAVAILABLE'

	/**
	 * Unavailable reason: No call was made because the circuit breaker prevented it.
	 */
	static UNAVAILABLE_CIRCUIT_BROKEN: 'CIRCUIT_BROKEN'

	/**
	 * Unavailable reason: No call was made because the service was not configured correctly.
	 */
	static UNAVAILABLE_CONFIG_PROBLEM: 'CONFIG_PROBLEM'

	/**
	 * Unavailable reason: No call was made because the service is disabled.
	 */
	static UNAVAILABLE_DISABLED: 'DISABLED'

	/**
	 * Unavailable reason: No call was made because the rate limit was hit.
	 */
	static UNAVAILABLE_RATE_LIMITED: 'RATE_LIMITED'

	/**
	 * Unavailable reason: A real call was made but a timeout occurred.
	 */
	static UNAVAILABLE_TIMEOUT: 'TIMEOUT'

	/**
	 * Error-specific code if applicable (e.g., HTTP response code for HTTPService).
	 * @readonly
	 */
	error: number

	/**
	 * Error message on a non-OK status.
	 * @readonly
	 */
	errorMessage: string

	/**
	 * Whether the response is the result of a "mock" service call.
	 * @readonly
	 */
	mockResult: boolean

	/**
	 * Extra error message on failure (if any).
	 * @readonly
	 */
	msg: string

	/**
	 * The actual object returned by the service when the status is OK.
	 * @readonly
	 */
	object: Object

	/**
	 * Whether the service call was successful.
	 * @readonly
	 */
	ok: boolean

	/**
	 * The status code. "OK" on success. Failure codes include "ERROR" and "SERVICE_UNAVAILABLE". If "SERVICE_UNAVAILABLE", unavailableReason is guaranteed to be non-null.
	 * @readonly
	 */
	status: string

	/**
	 * The reason the status is SERVICE_UNAVAILABLE.
	 * @readonly
	 */
	unavailableReason: string

	/**
	 * Constructs a new result instance.
	 */
	constructor()

	/**
	 * Returns error-specific code if applicable (e.g., HTTP response code for HTTPService).
	 */
	getError(): number

	/**
	 * Returns error message on a non-OK status.
	 */
	getErrorMessage(): string

	/**
	 * Returns extra error message on failure (if any).
	 */
	getMsg(): string

	/**
	 * Returns the actual object returned by the service when the status is OK.
	 */
	getObject(): Object

	/**
	 * Returns the status code. "OK" on success. Failure codes include "ERROR" and "SERVICE_UNAVAILABLE". If "SERVICE_UNAVAILABLE", unavailableReason is guaranteed to be non-null.
	 */
	getStatus(): string

	/**
	 * Returns the reason the status is SERVICE_UNAVAILABLE, or null if the status is not SERVICE_UNAVAILABLE.
	 */
	getUnavailableReason(): string

	/**
	 * Returns whether the response is the result of a "mock" service call.
	 */
	isMockResult(): boolean

	/**
	 * Returns whether the service call was successful.
	 */
	isOk(): boolean

	/**
	 * Returns a string representation of the result.
	 */
	toString(): string
}
```
