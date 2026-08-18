 # dw.order.Invoice

 ## Overview
Represents a debit or credit invoice created from order post-processing APIs.

## Description
Invoice objects represent debit or credit invoices created by ShippingOrder, Appeasement, ReturnCase, or Return APIs. Provides constants for sorting and status/type qualifiers and access to items and transactions.

## All Known Subclasses
 (none)

```ts
declare class Invoice extends dw.object.Extensible {
    /** Sorting by creation date. Use with FilteringCollection.sort */
    static ORDERBY_CREATION_DATE: Object

    /** Sorting by item id. */
    static ORDERBY_ITEMID: Object

    /** Sorting by item position. */
    static ORDERBY_ITEMPOSITION: Object

    /** Reverse order sorting. */
    static ORDERBY_REVERSE: Object

    /** Unsorted ordering. */
    static ORDERBY_UNSORTED: Object

    /** Selects capture transactions. */
    static QUALIFIER_CAPTURE: Object

    /** Selects product items. */
    static QUALIFIER_PRODUCTITEMS: Object

    /** Selects refund transactions. */
    static QUALIFIER_REFUND: Object

    /** Selects service items. */
    static QUALIFIER_SERVICEITEMS: Object

    /** Invoice status: failed. */
    static STATUS_FAILED: 'FAILED'

    /** Invoice status: manual. */
    static STATUS_MANUAL: 'MANUAL'

    /** Invoice status: not paid. */
    static STATUS_NOT_PAID: 'NOT_PAID'

    /** Invoice status: paid. */
    static STATUS_PAID: 'PAID'

    /** Qualifier for appeasement invoices. */
    static TYPE_APPEASEMENT: Object

    /** Qualifier for credit invoices. */
    static TYPE_CREDIT: Object

    /** Qualifier for debit invoices. */
    static TYPE_DEBIT: Object

    /**
     * Returns payment transactions related to the invoice.
     */
    getPaymentTransactions(): dw.util.Collection

    /**
     * Returns invoice items (lines) for the invoice.
     */
    getItems(): dw.util.Collection
}
```
