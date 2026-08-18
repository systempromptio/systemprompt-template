# dw.catalog.ProductAvailabilityLevels

## Overview
Encapsulates quantities by availability status (in stock, backorder, preorder, not available) for a product.

## Description
Provides read-only access to quantity totals per availability category and utility methods to retrieve those quantities.

```ts
declare class ProductAvailabilityLevels  {
    /** Backorder quantity (read-only). */
    readonly backorder: dw.value.Quantity

    /** Number of attributes that contain non-zero quantities (read-only). */
    readonly count: number

    /** Quantity in stock (read-only). */
    readonly inStock: dw.value.Quantity

    /** Quantity not available (read-only). */
    readonly notAvailable: dw.value.Quantity

    /** Pre-order quantity (read-only). */
    readonly preorder: dw.value.Quantity

    /** Returns the backorder quantity. */
    getBackorder(): dw.value.Quantity

    /** Returns the number of attributes that contain non-zero quantities. */
    getCount(): number

    /** Returns the quantity in stock. */
    getInStock(): dw.value.Quantity

    /** Returns the quantity that is not available. */
    getNotAvailable(): dw.value.Quantity

    /** Returns the pre-order quantity. */
    getPreorder(): dw.value.Quantity
}
```

## All Known Subclasses
None
