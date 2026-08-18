# dw.extensions.paymentapi.PaymentApiHooks

## Overview
Extension points to customize Payment API authorization requests and responses; executed inside a transaction.

## Description
PaymentApiHooks interface contains extension points for customizing Payment API requests for authorization and their responses. Hooks run in a transaction and are registered via a `hooks` JSON in a site cartridge.

##
```ts
declare class PaymentApiHooks  {
	/** Extension point names */
	static extensionPointAfterAuthorization: 'dw.extensions.paymentapi.afterAuthorization'
	static extensionPointBeforeAuthorization: 'dw.extensions.paymentapi.beforeAuthorization'

	/** Called after response handling for an authorization request; return non-null Status to end execution. */
	afterAuthorization(order: Order, payment: OrderPaymentInstrument, custom: Object, status: Status): Status

	/** Called before making an authorization request; return non-null Status to end execution. */
	beforeAuthorization(order: Order, payment: OrderPaymentInstrument, custom: Object): Status
}
```
