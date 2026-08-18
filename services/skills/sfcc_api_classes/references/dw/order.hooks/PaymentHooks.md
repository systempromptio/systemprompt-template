# dw.order.hooks.PaymentHooks

## Overview
Interface for script hooks to customize order center payment functionality.

## Description
Represents all script hooks that can be registered to customize the order center payment functionality. Contains extension points (hook names) and functions called by each extension point. Hook functions must be defined inside a JavaScript source, exported, and registered via `hooks.json` in a site cartridge's `package.json`.

```ts
declare class PaymentHooks  {
	/**
	 * Extension point name for payment authorization (optional).
	 */
	static extensionPointAuthorize: 'dw.order.payment.authorize'

	/**
	 * Extension point name for credit card authorization (optional).
	 */
	static extensionPointAuthorizeCreditCard: 'dw.order.payment.authorizeCreditCard'

	/**
	 * Extension point name for payment capture.
	 */
	static extensionPointCapture: 'dw.order.payment.capture'

	/**
	 * Extension point name for reauthorization.
	 */
	static extensionPointReauthorize: 'dw.order.payment.reauthorize'

	/**
	 * Extension point name for payment refund.
	 */
	static extensionPointRefund: 'dw.order.payment.refund'

	/**
	 * Extension point name for releasing authorization.
	 */
	static extensionPointReleaseAuthorization: 'dw.order.payment.releaseAuthorization'

	/**
	 * Extension point name for validating authorization.
	 */
	static extensionPointValidateAuthorization: 'dw.order.payment.validateAuthorization'

	/**
	 * Authorizes payment for an order.
	 * @param order - the order
	 * @param paymentDetails - specified payment details
	 */
	authorize(order: Order, paymentDetails: OrderPaymentInstrument): Status

	/**
	 * Authorizes credit card payment for an order.
	 * @param order - the order
	 * @param paymentDetails - specified payment details
	 * @param cvn - the credit card verification number
	 */
	authorizeCreditCard(order: Order, paymentDetails: OrderPaymentInstrument, cvn: string): Status

	/**
	 * Captures payment for an invoice.
	 * @param invoice - the invoice
	 */
	capture(invoice: Invoice): Status

	/**
	 * Reauthorizes payment for an order.
	 * @param order - the order
	 */
	reauthorize(order: Order): Status

	/**
	 * Refunds payment for an invoice.
	 * @param invoice - the invoice
	 */
	refund(invoice: Invoice): Status

	/**
	 * Releases authorization for an order.
	 * @param order - the order
	 */
	releaseAuthorization(order: Order): Status

	/**
	 * Validates payment authorization for an order.
	 * @param order - the order
	 */
	validateAuthorization(order: Order): Status
}
```
