# dw.extensions.paymentrequest.PaymentRequestHooks

## Overview
Extension points for Payment Request flow (getPaymentRequest, shipping changes, place order, authorize payment); executed in a transaction.

## Description
PaymentRequestHooks interface contains extension points for customizing Payment Request flows and handling Payment Request events. Hooks are executed in a transaction and registered via a `hooks` JSON in a site cartridge.

##
```ts
declare class PaymentRequestHooks  {
	static extensionPointAbort: 'dw.extensions.paymentrequest.abort'
	static extensionPointGetPaymentRequest: 'dw.extensions.paymentrequest.getPaymentRequest'
	static extensionPointPaymentAcceptedAuthorizeOrderPayment: 'dw.extensions.paymentrequest.paymentAccepted.authorizeOrderPayment'
	static extensionPointPaymentAcceptedPlaceOrder: 'dw.extensions.paymentrequest.paymentAccepted.placeOrder'
	static extensionPointShippingAddressChange: 'dw.extensions.paymentrequest.shippingAddressChange'
	static extensionPointShippingOptionChange: 'dw.extensions.paymentrequest.shippingOptionChange'

	/** Called after Payment Request UI canceled. */
	abort(basket: Basket): PaymentRequestHookResult
	/** Called after shopper accepts Payment Request to authorize payment. */
	authorizeOrderPayment(order: Order, response: Object): Status
	/** Called to provide PaymentRequest constructor parameters. */
	getPaymentRequest(basket: Basket, parameters: Object): PaymentRequestHookResult
	/** Called after payment authorized when order is ready to be placed. */
	placeOrder(order: Order): PaymentRequestHookResult
	/** Called after handling shipping address change. */
	shippingAddressChange(basket: Basket, details: Object): PaymentRequestHookResult
	/** Called after handling shipping option change. */
	shippingOptionChange(basket: Basket, shippingMethod: ShippingMethod, details: Object): PaymentRequestHookResult
}
```
