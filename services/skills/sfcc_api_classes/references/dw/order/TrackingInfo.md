# dw.order.TrackingInfo

## Overview
Provides basic information about a tracking info used by shipping orders and refs.

## Description
An instance is identified by an ID and can be referenced from ShippingOrderItems using TrackingRefs.

```ts
declare class TrackingInfo extends dw.object.Extensible {
    /**
     * Get the Carrier.
     */
    carrier: string

    /**
     * Get the service (ship method) of the used carrier.
     */
    carrierService: string

    /**
     * Mandatory identifier for this tracking information.
     * @readonly
     */
    ID: string

    /**
     * Get the ship date.
     */
    shipDate: Date

    /**
     * Gets the shipping order.
     * @readonly
     */
    shippingOrder: unknown

    /**
     * Get the tracking number.
     */
    trackingNumber: string

    /**
     * Gets the tracking refs (shipping order items) assigned to this tracking info.
     * @readonly
     */
    trackingRefs: dw.util.Collection

    /**
     * Get the id of the shipping warehouse.
     */
    warehouseID: string

    /**
     * Get the Carrier.
     * @returns {string}
     */
    getCarrier(): string

    /**
     * Get the service (ship method) of the used carrier.
     * @returns {string}
     */
    getCarrierService(): string

    /**
     * Get the mandatory identifier for this tracking information.
     * @returns {string}
     */
    getID(): string

    /**
     * Get the ship date.
     * @returns {Date}
     */
    getShipDate(): Date

    /**
     * Gets the shipping order.
     * @returns {dw.order.ShippingOrder}
     */
    getShippingOrder(): unknown

    /**
     * Get the tracking number.
     * @returns {string}
     */
    getTrackingNumber(): string

    /**
     * Gets the tracking refs (shipping order items) assigned to this tracking info.
     * @returns {dw.util.Collection}
     */
    getTrackingRefs(): dw.util.Collection

    /**
     * Get the id of the shipping warehouse.
     * @returns {string}
     */
    getWarehouseID(): string

    /**
     * Set the Carrier.
     * @param {string} carrier
     */
    setCarrier(carrier: string): void

    /**
     * Set the service (ship method) of the used carrier.
     * @param {string} carrierService
     */
    setCarrierService(carrierService: string): void

    /**
     * Set the ship date.
     * @param {Date} shipDate
     */
    setShipDate(shipDate: Date): void

    /**
     * Set the TrackingNumber.
     * @param {string} trackingNumber
     */
    setTrackingNumber(trackingNumber: string): void

    /**
     * Set the id of the shipping warehouse.
     * @param {string} warehouseID
     */
    setWarehouseID(warehouseID: string): void
}
```
