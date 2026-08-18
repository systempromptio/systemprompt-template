# dw.order.Basket

## Overview
Represents a shopping cart (basket) with line items, inventory reservation controls, and basket metadata (temporary, agent, editing an order).

## Description
The Basket class represents a shopping cart. It exposes whether the basket was created by an agent, inventory reservation expiry, references to an order being edited, tax rounding mode, and whether the basket is temporary. It provides methods to reserve/release inventory, update currency, start checkout, and set business/channel types.

## All Known Subclasses
(none)

```ts
declare class Basket extends LineItemCtnr {
	/** Returns if the basket was created by an agent. */
	readonly agentBasket: boolean

	/** The timestamp when the inventory for this basket expires (or null). */
	readonly inventoryReservationExpiry: Date

	/** The order that this basket represents if editing, otherwise null. */
	readonly orderBeingEdited: Order

	/** The order number being edited, or null. */
	readonly orderNoBeingEdited: string

	/** True if tax was calculated with grouped taxation. */
	readonly taxRoundedAtGroup: boolean

	/** True if the basket is temporary. */
	readonly temporary: boolean

	getInventoryReservationExpiry(): Date
	getOrderBeingEdited(): Order
	getOrderNoBeingEdited(): string
	isAgentBasket(): boolean
	isTaxRoundedAtGroup(): boolean
	isTemporary(): boolean
	releaseInventory(): Status
	reserveInventory(): Status
	reserveInventory(reservationDurationInMinutes: number): Status
	reserveInventory(reservationDurationInMinutes: number, removeIfNotAvailable: boolean): Status
	setBusinessType(aType: number): void
	setChannelType(aType: number): void
	setCustomerNo(customerNo: string): void
	startCheckout(): void
	updateCurrency(): void
}
```
