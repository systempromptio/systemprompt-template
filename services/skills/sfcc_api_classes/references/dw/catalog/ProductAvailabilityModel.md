# dw.catalog.ProductAvailabilityModel

## Overview
Provides availability information for a single product, including status, SKU coverage, time-to-out-of-stock, and quantity breakdowns.

## Description
Supports queries for availability and orderability, and returns detailed availability levels for a requested quantity. Includes constants for availability statuses and convenience properties for common checks.

```ts
declare class ProductAvailabilityModel  {
    /** "BACKORDER" */
    static AVAILABILITY_STATUS_BACKORDER: 'BACKORDER'

    /** "IN_STOCK" */
    static AVAILABILITY_STATUS_IN_STOCK: 'IN_STOCK'

    /** "NOT_AVAILABLE" */
    static AVAILABILITY_STATUS_NOT_AVAILABLE: 'NOT_AVAILABLE'

    /** "PREORDER" */
    static AVAILABILITY_STATUS_PREORDER: 'PREORDER'

    /** Availability ratio (0-1) for the product (read-only). */
    readonly availability: number

    /** Availability status for the product's minimum-orderable-quantity (read-only). */
    readonly availabilityStatus: string

    /** True if the product is in stock for the MOQ (read-only). */
    readonly inStock: boolean

    /** Associated ProductInventoryRecord (read-only). */
    readonly inventoryRecord: dw.catalog.ProductInventoryRecord | null

    /** True if product is orderable for the MOQ (read-only). */
    readonly orderable: boolean

    /** SKU coverage ratio for master/set products (read-only). */
    readonly SKUCoverage: number

    /** Estimated hours until out of stock (read-only). */
    readonly timeToOutOfStock: number

    /** Returns availability ratio. */
    getAvailability(): number

    /** Returns ProductAvailabilityLevels for the given quantity (throws if quantity <= 0). */
    getAvailabilityLevels(quantity: number): dw.catalog.ProductAvailabilityLevels

    /** Returns availability status for MOQ. */
    getAvailabilityStatus(): string

    /** Returns associated ProductInventoryRecord or null. */
    getInventoryRecord(): dw.catalog.ProductInventoryRecord | null

    /** Returns SKU coverage ratio. */
    getSKUCoverage(): number

    /** Returns estimated hours until out of stock. */
    getTimeToOutOfStock(): number

    /** Returns true if product is in stock for given quantity. */
    isInStock(quantity: number): boolean

    /** Convenience isInStock() for MOQ. */
    isInStock(): boolean

    /** Returns true if product is orderable for given quantity. */
    isOrderable(quantity: number): boolean

    /** Convenience isOrderable() for MOQ. */
    isOrderable(): boolean
}
```

## All Known Subclasses
None

