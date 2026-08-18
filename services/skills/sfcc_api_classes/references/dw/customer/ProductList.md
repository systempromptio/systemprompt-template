# dw.customer.ProductList

## Overview
Represents a customer-owned list of items (wish list, gift registry, etc.).

## Description
Provides accessors and mutators for list metadata and items: creation, name, description, event details, shipping addresses, and item collections. Supports product and public item retrieval and management, purchases aggregation, and export status tracking.

```ts
declare class ProductList {
  /** Export status values: EXPORT_STATUS_NOTEXPORTED, EXPORT_STATUS_EXPORTED */
  static EXPORT_STATUS_NOTEXPORTED: EnumValue
  static EXPORT_STATUS_EXPORTED: EnumValue

  /** Read-only: unique system ID. */
  getID(): string

  /** Returns the item representing a gift certificate, or null. */
  getGiftCertificateItem(): ProductListItem

  /** Returns an item by ID or null. */
  getItem(ID: string): ProductListItem

  /** Returns all items. */
  getItems(): Collection

  /** Returns items referencing products. */
  getProductItems(): Collection

  /** Returns public items only. */
  getPublicItems(): Collection

  /** Returns aggregated purchases for all items. */
  getPurchases(): Collection

  /** Returns metadata accessors. */
  getName(): string
  getDescription(): string
  getType(): number
  getOwner(): Customer
  isAnonymous(): boolean
  isPublic(): boolean

  /** Returns and sets addresses and event-related fields. */
  getShippingAddress(): CustomerAddress
  getPostEventShippingAddress(): CustomerAddress
  getCurrentShippingAddress(): CustomerAddress
  getEventDate(): Date
  getEventCity(): string
  getEventState(): string
  getEventCountry(): string
  getEventType(): string
  getExportStatus(): EnumValue
  getLastExportTime(): Date

  /** Modifiers and creators. */
  createProductItem(product: Product): ProductListItem
  createRegistrant(): ProductListRegistrant
  createGiftCertificateItem(): ProductListItem
  removeItem(item: ProductListItem): void
  removeRegistrant(): void
  removeCoRegistrant(): void
  setName(name: string): void
  setDescription(description: string): void
  setPublic(flag: boolean): void
  setShippingAddress(address: CustomerAddress): void
  setPostEventShippingAddress(address: CustomerAddress): void
  setEventDate(eventDate: Date): void
  setEventCity(eventCity: string): void
  setEventState(eventState: string): void
  setEventCountry(eventCountry: string): void
  setEventType(eventType: string): void
}
```
