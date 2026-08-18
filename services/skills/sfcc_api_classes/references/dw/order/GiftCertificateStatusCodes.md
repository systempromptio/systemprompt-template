 # dw.order.GiftCertificateStatusCodes

 ## Overview
Status code constants for gift certificate redemption errors.

## Description
Helper class containing status code string constants returned when gift certificate operations fail.

## All Known Subclasses
 (none)

```ts
declare class GiftCertificateStatusCodes  {
    /**
     * Indicates the gift certificate was in a different currency than the basket.
     */
    static GIFTCERTIFICATE_CURRENCY_MISMATCH: 'GIFTCERTIFICATE_CURRENCY_MISMATCH'

    /**
     * Indicates the gift certificate is disabled.
     */
    static GIFTCERTIFICATE_DISABLED: 'GIFTCERTIFICATE_DISABLED'

    /**
     * Indicates insufficient balance on the gift certificate.
     */
    static GIFTCERTIFICATE_INSUFFICIENT_BALANCE: 'GIFTCERTIFICATE_INSUFFICIENT_BALANCE'

    /**
     * Indicates the gift certificate was not found.
     */
    static GIFTCERTIFICATE_NOT_FOUND: 'GIFTCERTIFICATE_NOT_FOUND'

    /**
     * Indicates the gift certificate is pending and unavailable for use.
     */
    static GIFTCERTIFICATE_PENDING: 'GIFTCERTIFICATE_PENDING'

    /**
     * Indicates the gift certificate has been fully redeemed.
     */
    static GIFTCERTIFICATE_REDEEMED: 'GIFTCERTIFICATE_REDEEMED'
}
```
