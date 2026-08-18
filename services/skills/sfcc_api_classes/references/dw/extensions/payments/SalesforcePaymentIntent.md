# dw.extensions.payments.SalesforcePaymentIntent

## Overview
Represents a Salesforce Payments payment intent. Tracks amount, status, associated payment method, and future-usage setup.

## Description
A payment intent is created when a shopper is ready to pay. It becomes confirmed when the shopper provides information acceptable to authorize payment. Use this class to inspect intent state and related metadata.

```ts
declare class SalesforcePaymentIntent  {
    /** Payment intent setup future usage: 'off_session' */
    static SETUP_FUTURE_USAGE_OFF_SESSION: 'off_session'

    /** Payment intent setup future usage: 'on_session' */
    static SETUP_FUTURE_USAGE_ON_SESSION: 'on_session'

    /** The amount of this payment intent. */
    readonly amount: import('dw.value.Money')

    /** True if this payment intent can be canceled. */
    readonly cancelable: boolean

    /** The client secret of this payment intent. */
    readonly clientSecret: string

    /** True if this payment intent has been confirmed. */
    readonly confirmed: boolean

    /** Identifier of this payment intent. */
    readonly ID: string

    /** The payment method for this payment intent, or null. */
    readonly paymentMethod: import('dw.extensions.payments.SalesforcePaymentMethod')

    /** True if this payment intent can be refunded. */
    readonly refundable: boolean

    /** Setup future usage value or null. */
    readonly setupFutureUsage: string

    /** Returns the amount of this payment intent. */
    getAmount(): import('dw.value.Money')

    /** Returns the client secret. */
    getClientSecret(): string

    /** Returns the identifier of this payment intent. */
    getID(): string

    /** Returns the OrderPaymentInstrument for this intent in the given basket, or null. */
    getPaymentInstrument(basket: import('dw.order.Basket')): import('dw.order.OrderPaymentInstrument')

    /** Returns the OrderPaymentInstrument for this intent in the given order, or null. */
    getPaymentInstrument(order: import('dw.order.Order')): import('dw.order.OrderPaymentInstrument')

    /** Returns the payment method for this payment intent, or null. */
    getPaymentMethod(): import('dw.extensions.payments.SalesforcePaymentMethod')

    /** Returns setup future usage value or null. */
    getSetupFutureUsage(): string

    /** Returns true if the intent is cancelable. */
    isCancelable(): boolean

    /** Returns true if the intent is confirmed. */
    isConfirmed(): boolean

    /** Returns true if the intent is refundable. */
    isRefundable(): boolean
}
```
