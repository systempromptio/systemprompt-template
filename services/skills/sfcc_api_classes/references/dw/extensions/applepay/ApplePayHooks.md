# dw.extensions.applepay.ApplePayHooks

## Overview
Hook interface defining extension points for Apple Pay integration (getRequest, shippingContactSelected, payment authorization, etc.).

## Description
Describes the available extension points for customizing Apple Pay checkout behavior. Hooks are executed in a transaction and must be exported functions located inside site cartridges with a `hooks` entry in `package.json`. The interface lists constants for hook names and methods for each extension point (authorizeOrderPayment, createOrder, failOrder, getRequest, cancel, paymentMethodSelected, placeOrder, prepareBasket, shippingContactSelected, shippingMethodSelected).

## All Known Subclasses


```ts
declare class ApplePayHooks  {
	/**
	 * Hook name constants (string values identifying extension points).
	 */
	static extensionPointCancel: 'dw.extensions.applepay.cancel'
	static extensionPointGetRequest: 'dw.extensions.applepay.getRequest'
	static extensionPointPaymentAuthorizedAuthorizeOrderPayment: 'dw.extensions.applepay.paymentAuthorized.authorizeOrderPayment'
	static extensionPointPaymentAuthorizedCreateOrder: 'dw.extensions.applepay.paymentAuthorized.createOrder'
	static extensionPointPaymentAuthorizedFailOrder: 'dw.extensions.applepay.paymentAuthorized.failOrder'
	static extensionPointPaymentAuthorizedPlaceOrder: 'dw.extensions.applepay.paymentAuthorized.placeOrder'
	static extensionPointPaymentMethodSelected: 'dw.extensions.applepay.paymentMethodSelected'
	static extensionPointPrepareBasket: 'dw.extensions.applepay.prepareBasket'
	static extensionPointShippingContactSelected: 'dw.extensions.applepay.shippingContactSelected'
	static extensionPointShippingMethodSelected: 'dw.extensions.applepay.shippingMethodSelected'

	/**
	 * Called to authorize the Apple Pay payment for the order.
	 */
	authorizeOrderPayment(order: Order, event: Object): Status

	/**
	 * Called after the Apple Pay payment sheet was canceled.
	 */
	cancel(basket: Basket): ApplePayHookResult

	/**
	 * Called after handling the ApplePayPaymentAuthorizedEvent to create an Order.
	 */
	createOrder(basket: Basket, event: Object): Order

	/**
	 * Called when payment authorization failed for the given order.
	 */
	failOrder(order: Order, status: Status): ApplePayHookResult

	/**
	 * Called to obtain the Apple Pay JS PaymentRequest for the basket.
	 */
	getRequest(basket: Basket, request: Object): ApplePayHookResult

	/**
	 * Called after payment method selection events.
	 */
	paymentMethodSelected(basket: Basket, event: Object, response: Object): ApplePayHookResult

	/**
	 * Called after payment has been authorized and the order is ready to be placed.
	 */
	placeOrder(order: Order): ApplePayHookResult

	/**
	 * Prepare the basket for Apple Pay checkout.
	 */
	prepareBasket(basket: Basket, parameters: Object): ApplePayHookResult

	/**
	 * Called after handling the ApplePayShippingContactSelectedEvent.
	 */
	shippingContactSelected(basket: Basket, event: Object, response: Object): ApplePayHookResult

	/**
	 * Called after handling the ApplePayShippingMethodSelectedEvent.
	 */
	shippingMethodSelected(basket: Basket, shippingMethod: ShippingMethod, event: Object, response: Object): ApplePayHookResult
}
```
