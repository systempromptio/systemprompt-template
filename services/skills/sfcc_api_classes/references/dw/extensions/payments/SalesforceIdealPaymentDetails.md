# dw.extensions.payments.SalesforceIdealPaymentDetails

## Overview
Details for a Salesforce Payments payment of type TYPE_IDEAL. Contains iDEAL-specific fields such as the bank used.

## Description
Represents iDEAL payment details returned by Salesforce Payments. Use when inspecting iDEAL-specific metadata for a payment.

```ts
declare class SalesforceIdealPaymentDetails  {
    /** The bank used for the payment, or null if not known. */
    readonly bank: string

    /** Returns the bank used for the payment, or null if not known. */
    getBank(): string
}
```

All Known Subclasses

