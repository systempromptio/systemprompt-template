# dw.extensions.payments.SalesforceKlarnaPaymentDetails

## Overview
Details for a Salesforce Payments payment of type TYPE_KLARNA. Contains Klarna-specific fields such as the payment method category.

## Description
Represents Klarna payment details returned by Salesforce Payments. Use when inspecting Klarna-specific metadata for a payment.

```ts
declare class SalesforceKlarnaPaymentDetails  {
    /** The payment method category used for the payment, or null if not known. */
    readonly paymentMethodCategory: string

    /** Returns the payment method category used for the payment, or null if not known. */
    getPaymentMethodCategory(): string
}
```

All Known Subclasses

