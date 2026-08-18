# dw.extensions.payments.SalesforceSepaDebitPaymentDetails

## Overview
Details for a Salesforce Payments SEPA Direct Debit payment. Contains the last4 account digits and accessors.

## Description
Details to a Salesforce Payments payment of type SEPA Direct Debit. See Salesforce Payments documentation for configuration and access.

```ts
declare class SalesforceSepaDebitPaymentDetails extends dw.extensions.payments.SalesforcePaymentDetails {
  /** The last 4 digits of the account number, or null if not known. */
  last4: string | null

  /** Returns the last 4 digits of the account number, or null if not known. */
  getLast4(): string | null
}
```

All Known Subclasses

None
