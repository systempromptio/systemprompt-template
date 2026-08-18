# dw.extensions.payments.SalesforcePaymentRequest

## Overview
Helper for building and formatting client-side payment request objects and configuring payment UI behavior (Stripe, PayPal, Buy Now flows).

## Description
Provides utilities to prepare payment request options, basket data, billing details, and to configure mounted payment components. Also exposes getters for request state and setters for options used by client-side integrations.

```ts
declare class SalesforcePaymentRequest  {
    /** Returns a JS object with payment request options (Stripe format) for the given basket and options. */
    static calculatePaymentRequestOptions(basket: import('dw.order.Basket'), options: Object): Object

    /** Returns a JS object containing payment request options in Stripe format. */
    static format(options: Object): Object

    /** Returns a JS object containing data used to prepare the shopper basket for Buy Now. */
    getBasketData(): Object

    /** Returns billing details JS object for creating a Stripe PaymentMethod. */
    getBillingDetails(): Object

    /** Returns true if card capture should be automatic. */
    getCardCaptureAutomatic(): boolean

    /** Returns set of excluded element types. */
    getExclude(): import('dw.util.Set')

    /** Returns the request identifier. */
    getID(): string

    /** Returns set of included element types. */
    getInclude(): import('dw.util.Set')

    /** Returns the DOM selector where to mount payment methods. */
    getSelector(): string

    /** Returns true if setup future usage is enabled. */
    getSetupFutureUsage(): boolean

    /** Returns the statement descriptor or null. */
    getStatementDescriptor(): string

    /** Set basket data from a JS object (sku, quantity, shippingMethod, options). */
    setBasketData(basketData: Object): void

    /** Set billing details JS object used when creating a Stripe PaymentMethod. */
    setBillingDetails(billingDetails: Object): void

    /** Set whether card capture should be automatic. */
    setCardCaptureAutomatic(cardCaptureAutomatic: boolean): void

    /** Set payment request options (total, displayItems, shippingOptions, etc.). */
    setOptions(options: Object): void

    /** Set PayPal Buttons API options. */
    setPayPalButtonsOptions(options: Object): void

    /** Set PayPal shipping preference string. */
    setPayPalShippingPreference(shippingPreference: string): void

    /** Set PayPal user action constant. */
    setPayPalUserAction(userAction: string): void

    /** Set controller to redirect to when shopper returns from 3rd party payment site. */
    setReturnController(returnController: string): void

    /** Enable or disable the save payment method control. */
    setSavePaymentMethodEnabled(savePaymentMethodEnabled: boolean): void

    /** Set whether to always save payment method for future off-session use. */
    setSetupFutureUsage(setupFutureUsage: boolean): void

    /** Set the statement descriptor used on customer statements for this request. */
    setStatementDescriptor(statementDescriptor: string): void

    /** Configure Stripe element creation options. */
    setStripeCreateElementOptions(element: string, options: Object): void

    /** Configure Stripe Elements options vector. */
    setStripeElementsOptions(options: Object): void
}
```
