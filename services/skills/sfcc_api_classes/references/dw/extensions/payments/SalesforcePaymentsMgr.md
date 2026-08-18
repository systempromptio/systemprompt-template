# dw.extensions.payments.SalesforcePaymentsMgr

## Overview
Manager API for common Salesforce Payments operations: payment intents, saved payment methods, PayPal orders, refunds and site config retrieval.

## Description
Provides static helper methods for creating/updating payment intents, managing saved payment methods, retrieving payment details and site configuration, and refunding captures.

```ts
declare class SalesforcePaymentsMgr  {
    /** Refund reason: duplicate */
    static REFUND_REASON_DUPLICATE: 'DUPLICATE'
    /** Refund reason: fraudulent */
    static REFUND_REASON_FRAUDULENT: 'FRAUDULENT'
    /** Refund reason: requested by customer */
    static REFUND_REASON_REQUESTED_BY_CUSTOMER: 'REQUESTED_BY_CUSTOMER'

    /** Returns payment intent for a basket or null. */
    static getPaymentIntent(basket: dw.order.Basket): dw.extensions.payments.SalesforcePaymentIntent
    static getPaymentIntent(order: dw.order.Order): dw.extensions.payments.SalesforcePaymentIntent

    /** Returns payments site configuration or null. */
    static getPaymentsSiteConfig(): dw.extensions.payments.SalesforcePaymentsSiteConfiguration

    /** Creates or updates a payment intent; returns Status 'OK' or 'ERROR'. */
    static createPaymentIntent(...args: any[]): dw.system.Status
    static updatePaymentIntent(...args: any[]): dw.system.Status
    static refundPaymentIntent(paymentIntent: any, amount?: dw.value.Money, refundProperties?: Object): dw.system.Status

    /** Manage saved payment methods and attached methods for customers. */
    static getSavedPaymentMethods(customer: dw.customer.Customer): dw.util.Collection
    static savePaymentMethod(customer: dw.customer.Customer, paymentMethod: any): void
    static removeSavedPaymentMethod(paymentMethod: any): void
    static detachPaymentMethod(paymentMethod: any): void

    /** PayPal helpers */
    static getPayPalOrder(basket: dw.order.Basket): dw.extensions.payments.SalesforcePayPalOrder
    static getPayPalOrder(order: dw.order.Order): dw.extensions.payments.SalesforcePayPalOrder

    /** Set or read Salesforce Payment details on payment instruments. */
    static setPaymentDetails(paymentInstrument: dw.order.OrderPaymentInstrument, paymentDetails: dw.extensions.payments.SalesforcePaymentDetails): void
    static getPaymentDetails(paymentInstrument: dw.order.OrderPaymentInstrument): dw.extensions.payments.SalesforcePaymentDetails
}
```
