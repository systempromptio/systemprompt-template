# dw.extensions.payments.SalesforcePaymentDetails

## Overview
Base class for Salesforce Payments payment details. Subclasses provide type-specific fields (card, bancontact, eps, ideal, klarna, etc.).

## Description
Contains common metadata for Salesforce Payments payments such as the payment type. Specific payment types provide additional details in subclasses.

All Known Subclasses
- SalesforceBancontactPaymentDetails
- SalesforceCardPaymentDetails
- SalesforceEpsPaymentDetails
- SalesforceIdealPaymentDetails
- SalesforceKlarnaPaymentDetails
- SalesforcePayPalPaymentDetails
- SalesforceSepaDebitPaymentDetails
- SalesforceVenmoPaymentDetails

```ts
declare class SalesforcePaymentDetails  {
    /** The payment type. */
    readonly type: string

    /** Returns the payment type. */
    getType(): string
}
```
