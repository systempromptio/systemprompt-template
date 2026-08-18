 # dw.order.OrderProcessStatusCodes

 ## Overview
Constants representing order process status codes used when interacting with orders (cancel, edit, inventory reservation, etc.).

## Description
Contains constants representing different status codes for interacting with an order, such as cancelling or editing an order.

## All Known Subclasses
None listed on class page.

```ts
declare class OrderProcessStatusCodes {
    /** Indicates that a coupon in the order is not valid. */
    static COUPON_INVALID: 'COUPON_INVALID'

    /** Indicates that no inventory could be reserved for the order. */
    static INVENTORY_RESERVATION_FAILED: 'INVENTORY_RESERVATION_FAILED'

    /** Indicates the order has already been cancelled. */
    static ORDER_ALREADY_CANCELLED: 'ORDER_CANCELLED'

    /** Indicates the order has already been exported. */
    static ORDER_ALREADY_EXPORTED: 'ORDER_EXPORTED'

    /** Indicates the order has already failed. */
    static ORDER_ALREADY_FAILED: 'ORDER_FAILED'

    /** Indicates the order has already been replaced. */
    static ORDER_ALREADY_REPLACED: 'ORDER_REPLACED'

    /** Indicates the order contains gift certificates and cannot be used. */
    static ORDER_CONTAINS_GC: 'CANCEL_ORDER_GC'

    /** Indicates the order is not cancelled. */
    static ORDER_NOT_CANCELLED: 'ORDER_NOT_CANCELLED'

    /** Indicates the order is not failed. */
    static ORDER_NOT_FAILED: 'ORDER_NOT_FAILED'

    /** Indicates the order has not been placed. */
    static ORDER_NOT_PLACED: 'ORDER_NOT_PLACED'
}
```
