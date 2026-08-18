# dw.order.ShippingMethod

## Overview
Represents a shipping method available for orders, including identifiers, display name and supported services.

## Description
Provides access to properties of a shipping method and helper routines to evaluate availability.

```ts
declare class ShippingMethod {
    /**
     * Identifier for the shipping method.
     */
    ID: string

    /**
     * Human readable display name.
     */
    displayName: string

    /**
     * Returns true when the method is applicable for the given shipment/context.
     * @param shipment unknown
     */
    isApplicable(shipment: unknown): boolean
}
```
