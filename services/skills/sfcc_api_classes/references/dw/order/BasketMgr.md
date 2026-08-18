# dw.order.BasketMgr

## Overview
Static helper methods for managing baskets: create/get/delete baskets, temporary baskets, agent baskets, and retrieval helpers for the current session customer.

## Description
Provides static utility methods to create agent or temporary baskets, create a basket from an Order (for editing), delete baskets, retrieve current or stored baskets, and list baskets for the session customer. Methods enforce permissions and return appropriate exceptions for invalid operations.

## All Known Subclasses
(none)

```ts
declare class BasketMgr {
	/** Retrieve all open baskets for the logged in customer including temporary baskets. */
	static readonly baskets: List

	/** Returns the current valid basket or null. */
	static readonly currentBasket: Basket

	/** Returns the current valid basket or creates a new one. */
	static readonly currentOrNewBasket: Basket

	/** Returns the stored basket for the session customer or null. */
	static readonly storedBasket: Basket

	/** Retrieve all open temporary baskets for the logged in customer. */
	static readonly temporaryBaskets: List

	static createAgentBasket(): Basket
	static createBasketFromOrder(order: Order): Basket
	static createTemporaryBasket(): Basket
	static deleteBasket(basket: Basket): void
	static deleteTemporaryBasket(basket: Basket): void
	static getBasket(uuid: string): Basket
	static getBaskets(): List
	static getCurrentBasket(): Basket
	static getCurrentOrNewBasket(): Basket
	static getStoredBasket(): Basket
	static getTemporaryBasket(uuid: string): Basket
	static getTemporaryBaskets(): List
}
```
