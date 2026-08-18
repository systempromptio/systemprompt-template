# dw.extensions.payments.SalesforcePayPalOrderPayer

## Overview
Payer information for a PayPal order in Salesforce Payments.

## Description
Read-only payer fields: email, given name, surname and phone.

```ts
declare class SalesforcePayPalOrderPayer  {
    emailAddress: string
    givenName: string
    phone: string
    surname: string

    getEmailAddress(): string
    getGivenName(): string
    getPhone(): string
    getSurname(): string
}
```
