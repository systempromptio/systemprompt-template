# dw.catalog.StoreMgr

## Overview
Helper methods to retrieve stores and store groups, and to search stores by location or postal code.

## Description
Provides static utilities for accessing store data for the current site: list store groups, get a store or
store group by id, read the session store id, set the session store id, and search stores by coordinates
or postal code with optional query filters.

```ts
declare class StoreMgr  {
    /** All the store groups of the current site. */
    static allStoreGroups: Collection

    /** Get the store id associated with the current session (may be null). */
    static storeIDFromSession: string

    /** Returns all the store groups of the current site. */
    static getAllStoreGroups(): Collection

    /** Returns the store with the specified id or null if not found. @param storeID */
    static getStore(storeID: string): Store

    /** Returns the store group with the specified id or null if not found. @param storeGroupID */
    static getStoreGroup(storeGroupID: string): StoreGroup

    /** Get the store id associated with the current session. */
    static getStoreIDFromSession(): string

    /**
     * Search for stores by geographic coordinates. Returns a LinkedHashMap of Store to distance.
     * @param latitude
     * @param longitude
     * @param distanceUnit 'mi'|'km'
     * @param maxDistance
     * @param queryString optional filter query
     * @param args optional query args
     */
    static searchStoresByCoordinates(latitude: number, longitude: number, distanceUnit: string, maxDistance: number, queryString?: string, ...args: Object[]): LinkedHashMap

    /** Convenience overload without query filters. */
    static searchStoresByCoordinates(latitude: number, longitude: number, distanceUnit: string, maxDistance: number): LinkedHashMap

    /** Search stores by country/postal code with optional filters. */
    static searchStoresByPostalCode(countryCode: string, postalCode: string, distanceUnit: string, maxDistance: number, queryString?: string, ...args: Object[]): LinkedHashMap

    /** Convenience overload without query filters. */
    static searchStoresByPostalCode(countryCode: string, postalCode: string, distanceUnit: string, maxDistance: number): LinkedHashMap

    /** Set the store id for the current session (null removes it). */
    static setStoreIDToSession(storeID: string): void
}
```
