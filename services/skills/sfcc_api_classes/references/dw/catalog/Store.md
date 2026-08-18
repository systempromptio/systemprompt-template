# dw.catalog.Store

## Overview
Represents a store in Commerce Cloud Digital with address, contact information, inventory, and configuration settings.

## Description
Represents a store in Commerce Cloud Digital. Stores have physical address information, contact details, geolocation coordinates, associated inventory lists, and can be configured for point-of-sale and store locator functionality.

```ts
declare class Store extends ExtensibleObject {
	/**
	 * Address line 1 of the store
	 */
	readonly address1: string

	/**
	 * Address line 2 of the store
	 */
	readonly address2: string

	/**
	 * City of the store
	 */
	readonly city: string

	/**
	 * Country code of the store
	 */
	readonly countryCode: EnumValue

	/**
	 * Email of the store
	 */
	readonly email: string

	/**
	 * Fax number of the store
	 */
	readonly fax: string

	/**
	 * ID of the store
	 */
	readonly ID: string

	/**
	 * Store image
	 */
	readonly image: MediaFile

	/**
	 * Inventory list associated with the store, or null if not associated
	 */
	readonly inventoryList: ProductInventoryList | null

	/**
	 * Inventory list ID associated with the store, or null if not associated
	 */
	readonly inventoryListID: string | null

	/**
	 * Latitude coordinate of the store
	 */
	readonly latitude: number

	/**
	 * Longitude coordinate of the store
	 */
	readonly longitude: number

	/**
	 * Name of the store
	 */
	readonly name: string

	/**
	 * Phone number of the store
	 */
	readonly phone: string

	/**
	 * Flag indicating this store uses Commerce Cloud Store for point-of-sale
	 */
	readonly posEnabled: boolean

	/**
	 * Postal code of the store
	 */
	readonly postalCode: string

	/**
	 * State code of the store
	 */
	readonly stateCode: string

	/**
	 * Store events information
	 */
	readonly storeEvents: MarkupText

	/**
	 * All store groups this store belongs to
	 */
	readonly storeGroups: Collection

	/**
	 * Store hours information
	 */
	readonly storeHours: MarkupText

	/**
	 * Flag indicating if store locator is enabled for this store
	 */
	readonly storeLocatorEnabled: boolean

	/**
	 * Returns the address1 of the store.
	 */
	getAddress1(): string

	/**
	 * Returns the address2 of the store.
	 */
	getAddress2(): string

	/**
	 * Returns the city of the store.
	 */
	getCity(): string

	/**
	 * Returns the countryCode of the store.
	 */
	getCountryCode(): EnumValue

	/**
	 * Returns the email of the store.
	 */
	getEmail(): string

	/**
	 * Returns the fax of the store.
	 */
	getFax(): string

	/**
	 * Returns the ID of the store.
	 */
	getID(): string

	/**
	 * Returns the store image.
	 */
	getImage(): MediaFile

	/**
	 * Returns the inventory list the store is associated with.
	 */
	getInventoryList(): ProductInventoryList | null

	/**
	 * Returns the inventory list id the store is associated with.
	 */
	getInventoryListID(): string | null

	/**
	 * Returns the latitude of the store.
	 */
	getLatitude(): number

	/**
	 * Returns the longitude of the store.
	 */
	getLongitude(): number

	/**
	 * Returns the name of the store.
	 */
	getName(): string

	/**
	 * Returns the phone of the store.
	 */
	getPhone(): string

	/**
	 * Returns the postalCode of the store.
	 */
	getPostalCode(): string

	/**
	 * Returns the stateCode of the store.
	 */
	getStateCode(): string

	/**
	 * Returns the storeEvents of the store.
	 */
	getStoreEvents(): MarkupText

	/**
	 * Returns all the store groups this store belongs to.
	 */
	getStoreGroups(): Collection

	/**
	 * Returns the storeHours of the store.
	 */
	getStoreHours(): MarkupText

	/**
	 * Returns the posEnabled flag for the Store.
	 */
	isPosEnabled(): boolean

	/**
	 * Returns the storeLocatorEnabled flag for the store.
	 */
	isStoreLocatorEnabled(): boolean
}
```
