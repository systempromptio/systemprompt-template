 # dw.order.GiftCertificateLineItem

 ## Overview
Represents a Gift Certificate line item in the cart. When an order is processed a Gift Certificate is created from this line item.

## Description
Represents a Gift Certificate line item in the cart. Contains recipient/sender information and links to the created Gift Certificate when processed.

## All Known Subclasses
 (none)

```ts
declare class GiftCertificateLineItem extends dw.order.LineItem {
    /**
     * The ID of the gift certificate that this line item was used to create.
     * @readonly
     */
    giftCertificateID: string

    /**
     * The message to include in the email of the recipient.
     * @readonly
     */
    message: string

    /**
     * The associated ProductListItem.
     * @readonly
     */
    productListItem: dw.customer.ProductListItem

    /**
     * The email address of the recipient.
     * @readonly
     */
    recipientEmail: string

    /**
     * The recipient's name.
     * @readonly
     */
    recipientName: string

    /**
     * The sender's name or null if undefined.
     * @readonly
     */
    senderName: string

    /**
     * The associated Shipment.
     * @readonly
     */
    shipment: dw.order.Shipment

    /**
     * Returns the ID of the gift certificate that this line item was used to create.
     */
    getGiftCertificateID(): string

    /**
     * Returns the message to include in the email of the person receiving the gift certificate line item.
     */
    getMessage(): string

    /**
     * Returns the associated ProductListItem.
     */
    getProductListItem(): dw.customer.ProductListItem

    /**
     * Returns the recipient email address.
     */
    getRecipientEmail(): string

    /**
     * Returns the recipient name.
     */
    getRecipientName(): string

    /**
     * Returns the sender name or null if undefined.
     */
    getSenderName(): string

    /**
     * Returns the associated Shipment.
     */
    getShipment(): dw.order.Shipment

    /**
     * Sets the associated gift certificate ID.
     * @param id
     */
    setGiftCertificateID(id: string): void

    /**
     * Sets the message for the recipient email.
     * @param message
     */
    setMessage(message: string): void

    /**
     * Sets the associated ProductListItem.
     * @param productListItem
     */
    setProductListItem(productListItem: dw.customer.ProductListItem): void

    /**
     * Sets the recipient email address.
     * @param recipientEmail
     */
    setRecipientEmail(recipientEmail: string): void

    /**
     * Sets the recipient name.
     * @param recipient
     */
    setRecipientName(recipient: string): void

    /**
     * Sets the sender name.
     * @param sender
     */
    setSenderName(sender: string): void

    /**
     * Associates the line item with a shipment.
     * @param shipment
     */
    setShipment(shipment: dw.order.Shipment): void
}
```
