# dw.extensions.paymentrequest.PaymentRequestHookResult

## Overview
Represents the result of a Payment Request hook: status, optional redirect URL, and optional JS custom event information.

## Description
Result object returned by PaymentRequest hooks describing the outcome (a `Status`), optional redirect `URL`, and optional event name/detail for client-side handling. Marked deprecated in favor of Salesforce Payments Google Pay support.

##
```ts
declare class PaymentRequestHookResult  {
	/** Read-only event detail object for a custom JS event. */
	readonly eventDetail: Object
	/** Read-only event name for a custom JS event. */
	readonly eventName: string
	/** Read-only redirect URL. */
	readonly redirect: URL
	/** Read-only status describing the outcome. */
	readonly status: Status

	/** Constructs a result with the given status and optional redirect. */
	PaymentRequestHookResult(status: Status, redirect: URL)

	getEventDetail(): Object
	getEventName(): string
	getRedirect(): URL
	getStatus(): Status
	setEvent(name: string): void
	setEvent(name: string, detail: Object): void
}
```
