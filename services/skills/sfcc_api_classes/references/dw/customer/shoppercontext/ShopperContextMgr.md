# dw.customer.shoppercontext.ShopperContextMgr

## Overview
Static helper methods to create, retrieve, evaluate, and remove `ShopperContext` for composable/headless or hybrid storefronts.

## Description
Provides static operations to manage Shopper Context for a shopper. Methods include `getGeolocation()`, `getShopperContext()`, `removeShopperContext()`, and `setShopperContext(ShopperContext, boolean)`. Methods may throw `ShopperContextException` on failure.

## All Known Subclasses


```ts
declare class ShopperContextMgr  {
	/** Read-only geolocation for clientIP if available. */
	static geolocation: Geolocation

	/** Returns the ShopperContext if exists, otherwise null. */
	static shopperContext: ShopperContext

	/** Returns Geolocation computed from clientIP in current ShopperContext. */
	static getGeolocation(): Geolocation

	/** Returns the ShopperContext if present. */
	static getShopperContext(): ShopperContext

	/** Removes ShopperContext for the customer. */
	static removeShopperContext(): void

	/** Sets or overwrites ShopperContext for the customer.
	 * @param shopperContext The shopper context to set.
	 * @param evaluateContextWithClientIP If true, evaluate and save clientIP in the context.
	 */
	static setShopperContext(shopperContext: ShopperContext, evaluateContextWithClientIP: boolean): void
}
```
