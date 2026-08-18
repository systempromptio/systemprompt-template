# dw.extensions.payments.SalesforcePayPalOrderAddress

## Overview
PayPal order shipping/address representation used by Salesforce Payments.

## Description
Read-only address fields used on PayPal orders (lines, administrative areas, country, postal code, full name).

```ts
declare class SalesforcePayPalOrderAddress  {
    addressLine1: string
    addressLine2: string
    adminArea1: string
    adminArea2: string
    countryCode: string
    fullName: string
    postalCode: string

    getAddressLine1(): string
    getAddressLine2(): string
    getAdminArea1(): string
    getAdminArea2(): string
    getCountryCode(): string
    getFullName(): string
    getPostalCode(): string
}
```
