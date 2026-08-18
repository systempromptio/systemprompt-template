# dw.extensions.payments.SalesforceVenmoPaymentDetails

## Overview
Details for a Salesforce Payments Venmo payment. Includes capture ID and payer email.

## Description
Details to a Salesforce Payments Venmo payment (PayPal Venmo). See Salesforce Payments documentation for configuration and access.

```ts
declare class SalesforceVenmoPaymentDetails extends dw.extensions.payments.SalesforcePaymentDetails {
  /** ID of the capture against the PayPal Venmo order, or null if not known. */
  captureID: string | null

  /** Email address of the payer for the PayPal Venmo order, or null if not known. */
  payerEmailAddress: string | null

  /** Returns the ID of the capture against the PayPal Venmo order, or null if not known. */
  getCaptureID(): string | null

  /** Returns the email address of the payer for the PayPal Venmo order, or null if not known. */
  getPayerEmailAddress(): string | null
}
```

All Known Subclasses

None
