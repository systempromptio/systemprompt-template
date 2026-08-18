 # dw.order.PaymentCard

 ## Overview
Represents payment cards and provides methods to access card attributes and status. Handles sensitive financial/cardholder data (PCI considerations).

## Description
Represents payment cards and provides methods to access the payment card attributes and status. Pay attention to PCI DSS requirements when handling card data.

## All Known Subclasses
None listed on class page.

```ts
declare class PaymentCard extends dw.object.PersistentObject {
    /** Returns 'true' if payment card is active (enabled). @readonly */
    active: boolean

    /** The unique card type of the payment card. @readonly */
    cardType: string

    /** The description of the payment card. @readonly */
    description: dw.content.MarkupText

    /** Reference to the payment card image. @readonly */
    image: dw.content.MediaFile

    /** The name of the payment card. @readonly */
    name: string

    /** Returns the unique card type of the payment card. */
    getCardType(): string

    /** Returns the description of the payment card. */
    getDescription(): dw.content.MarkupText

    /** Returns the reference to the payment card image. */
    getImage(): dw.content.MediaFile

    /** Returns the name of the payment card. */
    getName(): string

    /** Returns true if payment card is active (enabled). */
    isActive(): boolean

    /**
     * Returns true if this payment card is applicable for the specified customer, country and payment amount and the session currency.
     * @param customer Customer or null
     * @param countryCode Billing country code or null
     * @param paymentAmount Payment amount or null
     */
    isApplicable(customer: dw.customer.Customer | null, countryCode: string | null, paymentAmount: number | null): boolean

    /**
     * Verify the card against the provided values (omits CSC verification).
     * @param expiresMonth expiration month (1-12)
     * @param expiresYear expiration year (e.g., 2025)
     * @param cardNumber card number string
     */
    verify(expiresMonth: number, expiresYear: number, cardNumber: string): dw.system.Status

    /**
     * Verify the card against the provided values including CSC.
     * @param expiresMonth expiration month (1-12)
     * @param expiresYear expiration year (e.g., 2025)
     * @param cardNumber card number string
     * @param csc card security code string
     */
    verify(expiresMonth: number, expiresYear: number, cardNumber: string, csc: string): dw.system.Status

}
```
