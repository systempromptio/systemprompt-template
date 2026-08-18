# dw.order.GiftCertificate

## Overview
Represents a gift certificate with amount, balance, recipient/sender details and status.

## Description
Provides accessors and mutators for gift certificate data (amount, balance, codes, recipient info, status and flags). Includes helper methods to mask codes and check enabled state.

```ts
declare class GiftCertificate extends ExtensibleObject {
    /** Original amount on the gift certificate. */
    amount: Money

    /** Remaining balance on the gift certificate. */
    balance: Money

    /** Description string for the gift certificate. */
    description: string

    /** The redemption code sent to the recipient. */
    giftCertificateCode: string

    /** Masked gift certificate code with all but last 4 chars replaced. */
    maskedGiftCertificateCode: string

    /** Merchant identifier for the gift certificate. */
    merchantID: string

    /** Message included in recipient email. */
    message: string

    /** Associated order number. */
    orderNo: string

    /** Recipient email address. */
    recipientEmail: string

    /** Recipient name. */
    recipientName: string

    /** Sender name or null. */
    senderName: string

    /** Status code: STATUS_PENDING, STATUS_ISSUED, STATUS_PARTIALLY_REDEEMED, STATUS_REDEEMED. */
    status: number

    /** Returns the original amount. */
    getAmount(): Money

    /** Returns the current balance. */
    getBalance(): Money

    /** Returns the description. */
    getDescription(): string

    /** Returns the gift certificate code. */
    getGiftCertificateCode(): string

    /** Returns the ID (deprecated, same as getGiftCertificateCode). */
    getID(): string

    /** Returns the masked gift certificate code (last 4 visible). */
    getMaskedGiftCertificateCode(): string

    /** Returns the masked code leaving `ignore` characters unmasked. */
    getMaskedGiftCertificateCode(ignore: number): string

    /** Returns the merchant ID. */
    getMerchantID(): string

    /** Returns the message for recipient email. */
    getMessage(): string

    /** Returns the associated order number. */
    getOrderNo(): string

    /** Returns the recipient email. */
    getRecipientEmail(): string

    /** Returns the recipient name. */
    getRecipientName(): string

    /** Returns the sender name or null. */
    getSenderName(): string

    /** Returns the numeric status. */
    getStatus(): number

    /** Returns true if the gift certificate is enabled. */
    isEnabled(): boolean

    /** Sets the description. */
    setDescription(description: string): void

    /** Enables or disables the gift certificate. */
    setEnabled(enabled: boolean): void

    /** Sets the message for recipient email. */
    setMessage(message: string): void

    /** Sets the order number. */
    setOrderNo(orderNo: string): void

    /** Sets recipient email. */
    setRecipientEmail(recipientEmail: string): void

    /** Sets recipient name. */
    setRecipientName(recipient: string): void

    /** Sets sender name. */
    setSenderName(sender: string): void

    /** Sets the status code. */
    setStatus(status: number): void

    static STATUS_PENDING: number
    static STATUS_ISSUED: number
    static STATUS_PARTIALLY_REDEEMED: number
    static STATUS_REDEEMED: number
}
```
