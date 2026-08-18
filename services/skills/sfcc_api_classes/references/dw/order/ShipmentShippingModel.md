# dw.order.ShipmentShippingModel

## Overview
Provides access to shipment-level shipping information (applicable/inapplicable methods and shipping cost).

## Description
Use `ShippingMgr.getShipmentShippingModel(Shipment)` to obtain the model for a shipment. It exposes collections of applicable and inapplicable shipping methods and shipping cost lookup.

```ts
declare class ShipmentShippingModel  {
    /** The active applicable shipping methods for the shipment related to this shipping model. @readonly */
    applicableShippingMethods: Collection

    /** The active inapplicable shipping methods for the shipment related to this shipping model. @readonly */
    inapplicableShippingMethods: Collection

    /** Returns the active applicable shipping methods for the shipment related to this shipping model. */
    getApplicableShippingMethods(): Collection

    /** Returns the active applicable shipping methods for this model and the specified shipping address. @param shippingAddressObj: unknown (OrderAddress-like object) */
    getApplicableShippingMethods(shippingAddressObj: Object): Collection

    /** Returns the active inapplicable shipping methods for the shipment related to this shipping model. */
    getInapplicableShippingMethods(): Collection

    /** Returns the active inapplicable shipping methods for this model and the specified shipping address. @param shippingAddressObj: unknown (OrderAddress-like object) */
    getInapplicableShippingMethods(shippingAddressObj: Object): Collection

    /** Returns the shipping cost object for the related shipment and the specified shipping method. @param shippingMethod: ShippingMethod */
    getShippingCost(shippingMethod: ShippingMethod): ShipmentShippingCost
}
```
