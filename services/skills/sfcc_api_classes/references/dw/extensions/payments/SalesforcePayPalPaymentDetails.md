# dw.extensions.payments.SalesforcePayPalPaymentDetails

## Overview
Payment details specific to PayPal payments; subclass of `SalesforcePaymentDetails` with PayPal-specific fields.

## Description
Includes PayPal capture identifier and payer email address when available.

```ts
declare class SalesforcePayPalPaymentDetails extends dw.extensions.payments.SalesforcePaymentDetails {
    captureID: string
    payerEmailAddress: string

    getCaptureID(): string
    getPayerEmailAddress(): string
}
```
