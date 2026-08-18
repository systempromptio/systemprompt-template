# dw.order.PaymentStatusCodes

## Overview
Helper class containing status code constants returned from payment card validation.

## Description
Contains string constants identifying common payment card validation errors (invalid card number, expiration date, security code length, etc.).

```ts
declare class PaymentStatusCodes {
    /** The code indicates that the credit card number is incorrect. */
    static CREDITCARD_INVALID_CARD_NUMBER: 'CREDITCARD_INVALID_CARD_NUMBER'

    /** The code indicates that the credit card is expired. */
    static CREDITCARD_INVALID_EXPIRATION_DATE: 'CREDITCARD_INVALID_EXPIRATION_DATE'

    /** The code indicates that the credit card security code length is invalid. */
    static CREDITCARD_INVALID_SECURITY_CODE: 'CREDITCARD_INVALID_SECURITY_CODE'
}
```
