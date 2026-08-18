# dw.order.ShippingOrder

## Overview
Represents a shipping order used to specify items that should be shipped; commonly exported to warehouse systems.

## Description
A ShippingOrder groups shipping order items, status, shipping method and tracking information. Status is calculated from item statuses.

```ts
declare class ShippingOrder extends Extensible {
    /** Sorting and qualifier constants (ORDERBY_ITEMID, ORDERBY_ITEMPOSITION, ORDERBY_UNSORTED, QUALIFIER_PRODUCTITEMS, QUALIFIER_SERVICEITEMS) */
    static ORDERBY_ITEMID: Object
    static ORDERBY_ITEMPOSITION: Object
    static ORDERBY_UNSORTED: Object
    static QUALIFIER_PRODUCTITEMS: Object
    static QUALIFIER_SERVICEITEMS: Object

    static STATUS_CANCELLED: 'CANCELLED'
    static STATUS_CONFIRMED: 'CONFIRMED'
    static STATUS_SHIPPED: 'SHIPPED'
    static STATUS_WAREHOUSE: 'WAREHOUSE'

    /** Returns null or the previously created Invoice. @readonly */
    invoice: Invoice | null

    /** Returns invoice number or null. @readonly */
    invoiceNumber: string | null

    /** A FilteringCollection of shipping order items. @readonly */
    items: FilteringCollection

    /** Shipping date or null. */
    shipDate: Date | null

    /** The shipping address (link to OrderAddress). */
    shippingAddress: OrderAddress | null

    /** The shipping method for the order. @readonly */
    shippingMethod: ShippingMethod | null

    /** Gets the shipping order number. @readonly */
    shippingOrderNumber: string

    /** Gets the status EnumValue. @readonly */
    status: EnumValue

    /** Gets tracking infos collection. @readonly */
    trackingInfos: Collection

    /** Adds a tracking info and returns it. @param trackingInfoID: string */
    addTrackingInfo(trackingInfoID: string): TrackingInfo

    /** Creates an invoice from this shipping order. */
    createInvoice(): Invoice

    /** Creates an invoice with invoiceNumber. @param invoiceNumber: string */
    createInvoice(invoiceNumber: string): Invoice

    // (Many additional methods present on the original page are documented in the HTML; include them when needed.)
}
```
