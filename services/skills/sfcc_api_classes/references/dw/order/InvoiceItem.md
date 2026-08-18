 # dw.order.InvoiceItem

 ## Overview
Represents a specific item in an Invoice; references exactly one order item.

## Description
InvoiceItem holds price, quantity, captured/refunded amounts and parent-child relations for invoice lines.

## All Known Subclasses
 (none)

```ts
declare class InvoiceItem extends dw.object.Extensible {
    /**
     * Price of a single unit before discount application.
     * @readonly
     */
    basePrice: dw.value.Money

    /**
     * The captured amount for this item.
     */
    capturedAmount: dw.value.Money

    /**
     * The invoice number this item belongs to.
     * @readonly
     */
    invoiceNumber: string

    /**
     * Parent invoice item or null.
     */
    parentItem: dw.order.InvoiceItem

    /**
     * The quantity of this item.
     * @readonly
     */
    quantity: dw.value.Quantity

    /**
     * The refunded amount for this item.
     */
    refundedAmount: dw.value.Money

    getBasePrice(): dw.value.Money
    getCapturedAmount(): dw.value.Money
    getInvoiceNumber(): string
    getParentItem(): dw.order.InvoiceItem
    getQuantity(): dw.value.Quantity
    getRefundedAmount(): dw.value.Money

    setCapturedAmount(capturedAmount: dw.value.Money): void
    setParentItem(parentItem: dw.order.InvoiceItem): void
    setRefundedAmount(refundedAmount: dw.value.Money): void
}
```
