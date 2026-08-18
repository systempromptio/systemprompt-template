# dw.order.PaymentTransaction

## Overview
Represents a payment transaction (authorization, capture, credit, reversal).

## Description
PaymentTransaction encapsulates the details of a payment transaction including amount, processor, instrument, transaction id and type. Several TYPE_* constants indicate transaction kinds.

```ts
declare class PaymentTransaction extends dw.object.PersistentObject {
    /** Constant representing the authorization type of payment transaction. */
    static TYPE_AUTH: 'AUTH'

    /** Constant representing the authorization reversal type of payment transaction. */
    static TYPE_AUTH_REVERSAL: 'AUTH_REVERSAL'

    /** Constant representing the capture type of payment transaction. */
    static TYPE_CAPTURE: 'CAPTURE'

    /** Constant representing the credit type of payment transaction. */
    static TYPE_CREDIT: 'CREDIT'

    /** The payment service-specific account id. */
    accountID: string

    /** The payment service-specific account type. */
    accountType: string

    /** The amount of the transaction. */
    amount: dw.value.Money

    /** The payment instrument related to this payment transaction. @readonly */
    paymentInstrument: dw.order.OrderPaymentInstrument

    /** The payment processor related to this payment transaction. */
    paymentProcessor: dw.order.PaymentProcessor

    /** The payment service-specific transaction id. */
    transactionID: string

    /** The value of the transaction type (EnumValue). */
    type: dw.value.EnumValue

    /** Returns the payment service-specific account id. */
    getAccountID(): string

    /** Returns the payment service-specific account type. */
    getAccountType(): string

    /** Returns the amount of the transaction. */
    getAmount(): dw.value.Money

    /** Returns the payment instrument related to this payment transaction. */
    getPaymentInstrument(): dw.order.OrderPaymentInstrument

    /** Returns the payment processor related to this payment transaction. */
    getPaymentProcessor(): dw.order.PaymentProcessor

    /** Returns the payment service-specific transaction id. */
    getTransactionID(): string

    /** Returns the EnumValue transaction type. */
    getType(): dw.value.EnumValue

    /** Sets the payment service-specific account id. */
    setAccountID(accountID: string): void

    /** Sets the payment service-specific account type. */
    setAccountType(accountType: string): void

    /** Sets the amount of the transaction. */
    setAmount(amount: dw.value.Money): void

    /** Sets the payment processor related to this payment transaction. */
    setPaymentProcessor(paymentProcessor: dw.order.PaymentProcessor): void

    /** Sets the payment service-specific transaction id. */
    setTransactionID(transactionID: string): void

    /** Sets the value of the transaction type. */
    setType(type: string): void
}
```
