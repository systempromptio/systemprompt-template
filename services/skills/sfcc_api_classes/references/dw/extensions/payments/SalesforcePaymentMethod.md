# dw.extensions.payments.SalesforcePaymentMethod

## Overview
Represents a Salesforce Payments payment method. Contains identifiers and limited metadata safe to display to shoppers.

## Description
Payment method objects describe credentials used to attempt payment (card, bank account, PayPal, etc.). Fields vary by type; sensitive details are not exposed.

```ts
declare class SalesforcePaymentMethod  {
    static TYPE_AFTERPAY_CLEARPAY: 'afterpay_clearpay'
    static TYPE_BANCONTACT: 'bancontact'
    static TYPE_CARD: 'card'
    static TYPE_EPS: 'eps'
    static TYPE_IDEAL: 'ideal'
    static TYPE_KLARNA: 'klarna'
    static TYPE_SEPA_DEBIT: 'sepa_debit'

    /** The bank of this payment method, or null if not available. */
    readonly bank: string

    /** The bank code of this payment method, or null if not available. */
    readonly bankCode: string

    /** The bank name of this payment method, or null if not available. */
    readonly bankName: string

    /** The bank branch code of this payment method, or null if not available. */
    readonly branchCode: string

    /** The brand of this payment method, or null if not available. */
    readonly brand: string

    /** The country of this payment method, or null if not available. */
    readonly country: string

    /** Identifier of this payment method. */
    readonly ID: string

    /** Last 4 digits of the credential, or null if unavailable. */
    readonly last4: string

    /** Payment method category, or null if unavailable. */
    readonly paymentMethodCategory: string

    /** The type of this payment method. */
    readonly type: string

    /** Returns the bank or null. */
    getBank(): string

    /** Returns the bank code or null. */
    getBankCode(): string

    /** Returns the bank name or null. */
    getBankName(): string

    /** Returns the branch code or null. */
    getBranchCode(): string

    /** Returns the brand or null. */
    getBrand(): string

    /** Returns the country or null. */
    getCountry(): string

    /** Returns the identifier. */
    getID(): string

    /** Returns the last 4 digits or null. */
    getLast4(): string

    /** Returns the SalesforcePaymentDetails for this payment method using the payment instrument. */
    getPaymentDetails(paymentInstrument: import('dw.order.OrderPaymentInstrument')): import('dw.extensions.payments.SalesforcePaymentDetails')

    /** Returns the payment method category or null. */
    getPaymentMethodCategory(): string

    /** Returns the type of this payment method. */
    getType(): string
}
```
