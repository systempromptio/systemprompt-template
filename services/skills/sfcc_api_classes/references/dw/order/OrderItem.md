# dw.order.OrderItem

## Overview
Represents an item on an order (base or concrete depending on type).

## Description
OrderItem provides access to order-specific item details and pricing on an order.

## All Known Subclasses

```ts
declare class OrderItem extends ExtensibleObject {
    /** Returns the UUID of the item. */
    getUUID(): string

    /** Returns the product ID or null. */
    getProductID(): string

    /** Returns the product name or null. */
    getProductName(): string

    /** Returns the quantity for this item. */
    getQuantity(): number

    /** Returns the price for this item. */
    getPrice(): Money

    /** Returns the total price for this item. */
    getTotalPrice(): Money

    /** Sets the quantity for this item. */
    setQuantity(qty: number): void

    /** Sets the price for this item. */
    setPrice(price: Money): void
}
```
