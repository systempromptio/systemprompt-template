# dw.extensions.payments.SalesforceEpsPaymentDetails

## Overview
Details for a Salesforce Payments payment of type TYPE_EPS. Contains EPS-specific fields such as the bank used.

## Description
Represents EPS payment details returned by Salesforce Payments. Use when inspecting EPS-specific metadata for a payment.

```ts
declare class SalesforceEpsPaymentDetails  {
    /** The bank used for the payment, or null if not known. */
    readonly bank: string

    /** Returns the bank used for the payment, or null if not known. */
    getBank(): string
}
```

All Known Subclasses

