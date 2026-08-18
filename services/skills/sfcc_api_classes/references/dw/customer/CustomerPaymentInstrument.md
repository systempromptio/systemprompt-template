# dw.customer.CustomerPaymentInstrument

## Overview
Represents a payment instrument stored on a customer's profile (credit card, bank transfer, etc.).

## Description
Extends persistent/encrypted/payment-instrument classes and exposes methods for accessing (masked or decrypted) sensitive fields depending on execution context and permissions. Use with PCI care.

```ts
declare class CustomerPaymentInstrument extends dw.order.PaymentInstrument {
    /**
     * Driver's license number for bank account (sensitive, masked unless secure context).
     */
    bankAccountDriversLicense: string

    /**
     * Bank account number (sensitive, masked unless secure context).
     */
    bankAccountNumber: string

    /**
     * Credit card number (sensitive, masked unless secure context).
     */
    creditCardNumber: string

    static getBankAccountDriversLicense(): string
    static getBankAccountNumber(): string
    static getCreditCardNumber(): string
}
```
