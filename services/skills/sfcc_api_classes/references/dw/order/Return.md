# dw.order.Return

## Overview
Represents a physical customer return and contains 1..n ReturnItems; associated with one ReturnCase.

## Description
The Return represents a physical customer return, contains ReturnItems, and is associated with a ReturnCase. Status values include STATUS_NEW and STATUS_COMPLETED.

```ts
declare class Return extends dw.object.Extensible {
    /** Sorting by item id. */
    static ORDERBY_ITEMID: Object

    /** Sorting by item position. */
    static ORDERBY_ITEMPOSITION: Object

    /** Unsorted ordering. */
    static ORDERBY_UNSORTED: Object

    /** Selects product items. */
    static QUALIFIER_PRODUCTITEMS: Object

    /** Selects service items. */
    static QUALIFIER_SERVICEITEMS: Object

    /** Constant for Return Status COMPLETED */
    static STATUS_COMPLETED: 'COMPLETED'

    /** Constant for Return Status NEW */
    static STATUS_NEW: 'NEW'

    /** Returns null or the previously created Invoice. */
    readonly invoice: dw.order.Invoice

    /** Returns null or the invoice-number. */
    readonly invoiceNumber: string

    /** The ReturnItems contained in the Return. */
    readonly items: dw.util.FilteringCollection

    /** A note for the return. */
    note: string

    /** The ReturnCase associated with this Return. */
    readonly returnCase: dw.order.ReturnCase

    /** The return number identifying this return. */
    readonly returnNumber: string

    /** Gets the return status. */
    status: dw.value.EnumValue

    createInvoice(): dw.order.Invoice
    createInvoice(invoiceNumber: string): dw.order.Invoice
    createItem(returnCaseItemID: string): dw.order.ReturnItem
    getInvoice(): dw.order.Invoice
    getInvoiceNumber(): string
    getItems(): dw.util.FilteringCollection
    getNote(): string
    getReturnCase(): dw.order.ReturnCase
    getReturnNumber(): string
    getStatus(): dw.value.EnumValue
    setNote(note: string): void
    setStatus(statusName: string): void
}
```
