# dw.extensions.applepay.ApplePayHookResult

## Overview
Represents the outcome of an Apple Pay hook execution, including status, optional redirect, and JS event information. Also includes constants for standard Apple Pay error reasons.

## Description
Encapsulates the result returned by Apple Pay extension hooks: a `Status` describing the outcome, optional `redirect` URL, and optional JS `eventName`/`eventDetail` to dispatch in the browser. This type defines string constants for common Apple Pay error reasons (e.g. invalid shipping contact) and a `STATUS_REASON_DETAIL_KEY` used when adding details to a `Status`.

## All Known Subclasses


```ts
declare class ApplePayHookResult  {
	/**
	 * Error reason codes (string constants).
	 */
	static REASON_BILLING_ADDRESS: 'InvalidBillingPostalAddress'
	static REASON_FAILURE: 'Failure'
	static REASON_PIN_INCORRECT: 'PINIncorrect'
	static REASON_PIN_LOCKOUT: 'PINLockout'
	static REASON_PIN_REQUIRED: 'PINRequired'
	static REASON_SHIPPING_ADDRESS: 'InvalidShippingPostalAddress'
	static REASON_SHIPPING_CONTACT: 'InvalidShippingContact'
	static STATUS_REASON_DETAIL_KEY: 'reason'

	/**
	 * Detail to the JS custom event to dispatch (read-only).
	 */
	eventDetail: Object

	/**
	 * Name of the JS custom event to dispatch (read-only).
	 */
	eventName: string

	/**
	 * Optional redirect URL to navigate to in response to this result (read-only).
	 */
	redirect: URL

	/**
	 * Status describing the outcome (read-only).
	 */
	status: Status

	/**
	 * Constructs a result with the given status and optional redirect.
	 */
	constructor(status: Status, redirect: URL): ApplePayHookResult

	/**
	 * Returns the JS event detail object.
	 */
	getEventDetail(): Object

	/**
	 * Returns the JS event name.
	 */
	getEventName(): string

	/**
	 * Returns the redirect URL.
	 */
	getRedirect(): URL

	/**
	 * Returns the status describing the outcome.
	 */
	getStatus(): Status

	/**
	 * Sets the JS event name to dispatch for this result.
	 */
	setEvent(name: string): void

	/**
	 * Sets the JS event name and detail to dispatch for this result.
	 */
	setEvent(name: string, detail: Object): void
}
```
