# dw.extensions.payments.SalesforcePayPalOrder

## Overview
Representation of a PayPal order within Salesforce Payments. Contains amount, payer, shipping and identifiers.

## Description
Provides read-only access to PayPal order fields and helpers to retrieve related payment instruments and Salesforce payment details.

```ts
declare class SalesforcePayPalOrder  {
    static TYPE_PAYPAL: 'paypal'
    static TYPE_VENMO: 'venmo'

    amount: dw.value.Money
    captureID: string
    completed: boolean
    ID: string
    payer: dw.extensions.payments.SalesforcePayPalOrderPayer
    shipping: dw.extensions.payments.SalesforcePayPalOrderAddress

    getAmount(): dw.value.Money
    getCaptureID(): string
    getID(): string
    getPayer(): dw.extensions.payments.SalesforcePayPalOrderPayer
    getPaymentDetails(paymentInstrument: dw.order.OrderPaymentInstrument): dw.extensions.payments.SalesforcePaymentDetails
    getPaymentInstrument(basket: dw.order.Basket): dw.order.OrderPaymentInstrument
    getPaymentInstrument(order: dw.order.Order): dw.order.OrderPaymentInstrument
    getShipping(): dw.extensions.payments.SalesforcePayPalOrderAddress
    isCompleted(): boolean
}
```
