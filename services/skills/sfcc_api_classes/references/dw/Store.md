# dw.catalog.Store

## Overview
Represents a physical store in Commerce Cloud Digital.

## Description
Contains store contact, location, hours, inventory linkage and flags for POS/store locator functionality.

## All Known Subclasses
None

```ts
declare class Store extends dw.object.ExtensibleObject {
    /** Primary address line. */
    address1: string

    /** Secondary address line. */
    address2: string

    /** City. */
    city: string

    /** Country code as EnumValue. */
    countryCode: dw.value.EnumValue

    /** Deprecated: indicates POS enabled; use isPosEnabled() instead. */
    demandwarePosEnabled: boolean

    /** Email contact. */
    email: string

    /** Fax number. */
    fax: string

    /** Store identifier. */
    ID: string

    /** Store image. */
    image: dw.content.MediaFile

    /** Associated inventory list or null. */
    inventoryList: dw.catalog.ProductInventoryList | null

    /** Inventory list ID or null. */
    inventoryListID: string | null

    /** Latitude coordinate. */
    latitude: number

    /** Longitude coordinate. */
    longitude: number

    /** Store name. */
    name: string

    /** Phone number. */
    phone: string

    /** POS enabled flag. */
    posEnabled: boolean

    /** Postal code. */
    postalCode: string

    /** State code. */
    stateCode: string

    /** Store events markup. */
    storeEvents: dw.content.MarkupText

    /** Collection of StoreGroup instances this store belongs to. */
    storeGroups: dw.util.Collection

    /** Store hours markup. */
    storeHours: dw.content.MarkupText

    /** Indicates if store locator is enabled. */
    storeLocatorEnabled: boolean

    /** Returns address1. */
    getAddress1(): string

    /** Returns address2. */
    getAddress2(): string

    /** Returns city. */
    getCity(): string

    /** Returns country code EnumValue. */
    getCountryCode(): dw.value.EnumValue

    /** Returns email. */
    getEmail(): string

    /** Returns fax. */
    getFax(): string

    /** Returns store ID. */
    getID(): string

    /** Returns store image. */
    getImage(): dw.content.MediaFile

    /** Returns associated inventory list or null. */
    getInventoryList(): dw.catalog.ProductInventoryList | null

    /** Returns inventory list ID or null. */
    getInventoryListID(): string | null

    /** Returns latitude. */
    getLatitude(): number

    /** Returns longitude. */
    getLongitude(): number

    /** Returns name. */
    getName(): string

    /** Returns phone. */
    getPhone(): string

    /** Returns postal code. */
    getPostalCode(): string

    /** Returns state code. */
    getStateCode(): string

    /** Returns store events markup. */
    getStoreEvents(): dw.content.MarkupText

    /** Returns store groups collection. */
    getStoreGroups(): dw.util.Collection

    /** Returns store hours markup. */
    getStoreHours(): dw.content.MarkupText

    /** Deprecated: returns demandware POS enabled flag. */
    isDemandwarePosEnabled(): boolean

    /** Returns whether POS is enabled for this store. */
    isPosEnabled(): boolean

    /** Returns whether store locator is enabled. */
    isStoreLocatorEnabled(): boolean
}
```
