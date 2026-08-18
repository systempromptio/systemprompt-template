# dw.order.ShippingMgr

## Overview
Manager utilities for shipping methods and shipping-related lookups.

## Description
Provides methods to query and retrieve available shipping methods, rules, and configurations.

```ts
declare class ShippingMgr {
    /**
     * Returns a list of available `ShippingMethod` for the given shipment.
     * @param shipment unknown
     */
    static getShipmentShippingMethods(shipment: unknown): Array<any>

    /**
     * Finds a shipping method by ID.
     * @param id string
     */
    static getShippingMethod(id: string): any
}
```
