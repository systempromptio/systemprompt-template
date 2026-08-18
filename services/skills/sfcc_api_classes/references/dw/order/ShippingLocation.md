# dw.order.ShippingLocation

## Overview
Represents a specific location for a shipment (address components and accessors).

## Description
Holds address fields and provides getters/setters for address components. Note: contains potentially sensitive personal information.

```ts
declare class ShippingLocation  {
    /** The shipping location's first address. */
    address1: string

    /** The shipping location's second address. */
    address2: string

    /** The shipping location's city. */
    city: string

    /** The shipping location's country code. */
    countryCode: string

    /** The shipping location's postal code. */
    postalCode: string

    /** The shipping location's post box. */
    postBox: string

    /** The shipping location's state code. */
    stateCode: string

    /** The shipping location's suite. */
    suite: string

    /** Constructs a new ShippingLocation. */
    ShippingLocation(): void

    /** Constructs a ShippingLocation from a CustomerAddress. @param address: CustomerAddress */
    ShippingLocation(address: CustomerAddress): void

    /** Constructs a ShippingLocation from an OrderAddress. @param address: OrderAddress */
    ShippingLocation(address: OrderAddress): void

    /** Getters */
    getAddress1(): string
    getAddress2(): string
    getCity(): string
    getCountryCode(): string
    getPostalCode(): string
    getPostBox(): string
    getStateCode(): string
    getSuite(): string

    /** Setters */
    setAddress1(aValue: string): void
    setAddress2(aValue: string): void
    setCity(aValue: string): void
    setCountryCode(aValue: string): void
    setPostalCode(aValue: string): void
    setPostBox(aValue: string): void
    setStateCode(aValue: string): void
    setSuite(aValue: string): void
}
```
