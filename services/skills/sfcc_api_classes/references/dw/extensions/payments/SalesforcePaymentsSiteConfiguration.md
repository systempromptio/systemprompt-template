# dw.extensions.payments.SalesforcePaymentsSiteConfiguration

## Overview
Represents site-specific configuration for Salesforce Payments (e.g., capture mode, express checkout, multi-step checkout).

## Description
Read-only view of payment-related site settings used by Salesforce Payments integration.

```ts
declare class SalesforcePaymentsSiteConfiguration  {
    /** true if credit-card capture is automatic for this site */
    cardCaptureAutomatic: boolean

    /** true if Express Checkout is enabled for this site */
    expressCheckoutEnabled: boolean

    /** true if Multi-Step Checkout is enabled for this site */
    multiStepCheckoutEnabled: boolean

    isCardCaptureAutomatic(): boolean
    isExpressCheckoutEnabled(): boolean
    isMultiStepCheckoutEnabled(): boolean
}
```
