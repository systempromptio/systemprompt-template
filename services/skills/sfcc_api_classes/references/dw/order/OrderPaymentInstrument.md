 # dw.order.OrderPaymentInstrument

 ## Overview
Represents any payment instrument used to pay orders (credit card, bank transfer). Provides standard methods for credit card payment and can be extended by custom attributes for other payment methods.

## Description
Represents any payment instrument used to pay orders, such as credit card or bank transfer. The object defines standard methods for credit card payment, and can be extended by attributes appropriate for other payment methods.

## All Known Subclasses
None listed on class page.

```ts
declare class OrderPaymentInstrument extends dw.order.PaymentInstrument {
    /**
     * The driver's license associated with a bank account if the calling context meets security criteria.
     * @readonly
     */
    bankAccountDriversLicense: string

    /**
     * The account number if the calling context meets security criteria.
     * @readonly
     */
    bankAccountNumber: string

    /**
     * The sum of the captured amounts (calculated on the fly).
     * @readonly
     */
    capturedAmount: dw.value.Money

    /**
     * The de-crypted credit card number if calling context meets security criteria.
     * @readonly
     */
    creditCardNumber: string

    /**
     * The Payment Transaction for this Payment Instrument or null.
     * @readonly
     */
    paymentTransaction: dw.order.PaymentTransaction

    /**
     * The sum of the refunded amounts (calculated on the fly).
     * @readonly
     */
    refundedAmount: dw.value.Money

    /**
     * Returns the driver's license associated with a bank account if the calling context meets security criteria.
     */
    getBankAccountDriversLicense(): string

    /**
     * Returns the account number if the calling context meets security criteria.
     */
    getBankAccountNumber(): string

    /**
     * Returns the sum of the captured amounts.
     */
    getCapturedAmount(): dw.value.Money

    /**
     * Returns the de-crypted credit card number if the calling context meets security criteria.
     */
    getCreditCardNumber(): string

    /**
     * Returns the Payment Transaction for this Payment Instrument or null.
     */
    getPaymentTransaction(): dw.order.PaymentTransaction

    /**
     * Returns the sum of the refunded amounts.
     */
    getRefundedAmount(): dw.value.Money

}
```
