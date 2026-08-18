 # dw.order.GiftCertificateMgr

 ## Overview
Static helper methods to create and retrieve GiftCertificates.

## Description
Contains static methods to create Gift Certificates and to retrieve them by code.

## All Known Subclasses
 (none)

```ts
declare class GiftCertificateMgr  {
    /**
     * Deprecated: error code for disabled gift certificates.
     */
    static GC_ERROR_DISABLED: 'GIFTCERTIFICATE-100'

    /**
     * Creates a GiftCertificate with the specified amount and code.
     * @param amount The monetary amount.
     * @param code The code to assign to the gift certificate.
     */
    static createGiftCertificate(amount: number, code: string): dw.order.GiftCertificate

    /**
     * Creates a GiftCertificate with the specified amount and an auto-generated code.
     * @param amount The monetary amount.
     */
    static createGiftCertificate(amount: number): dw.order.GiftCertificate

    /**
     * Returns the GiftCertificate identified by the specified code.
     * @param giftCertificateCode The gift certificate code.
     */
    static getGiftCertificate(giftCertificateCode: string): dw.order.GiftCertificate
}
```
