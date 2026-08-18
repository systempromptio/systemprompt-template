# dw.catalog.PriceBook

## Overview
Represents a price book, which defines product prices in a specific currency and supports online/offline status, parent-child relationships, and metadata.

## Description
A PriceBook contains product pricing information for a given currency. Price books can be organized in a parent-child hierarchy, support online/offline scheduling, and provide localized metadata. Not directly instantiable.

## Inheritance
Object → PersistentObject → ExtensibleObject → PriceBook

```ts
declare class PriceBook extends ExtensibleObject {
	/**
	 * Currency code of the price book.
	 */
	readonly currencyCode: string
	/**
	 * Description of the price book.
	 */
	readonly description: string | null
	/**
	 * Display name of the price book.
	 */
	readonly displayName: string | null
	/**
	 * Price book ID.
	 */
	readonly ID: string
	/**
	 * Online status of the price book.
	 */
	readonly online: boolean
	/**
	 * Online status flag.
	 */
	readonly onlineFlag: boolean
	/**
	 * Date from which the price book is online.
	 */
	readonly onlineFrom: Date | null
	/**
	 * Date until which the price book is online.
	 */
	readonly onlineTo: Date | null
	/**
	 * Parent price book, or null if none.
	 */
	readonly parentPriceBook: PriceBook | null

	/**
	 * Returns the currency code of the price book.
	 */
	getCurrencyCode(): string
	/**
	 * Returns the description of the price book.
	 */
	getDescription(): string | null
	/**
	 * Returns the display name of the price book.
	 */
	getDisplayName(): string | null
	/**
	 * Returns the price book ID.
	 */
	getID(): string
	/**
	 * Returns the online status of the price book.
	 */
	getOnline(): boolean
	/**
	 * Returns the online status flag.
	 */
	getOnlineFlag(): boolean
	/**
	 * Returns the date from which the price book is online.
	 */
	getOnlineFrom(): Date | null
	/**
	 * Returns the date until which the price book is online.
	 */
	getOnlineTo(): Date | null
	/**
	 * Returns the parent price book, or null if none.
	 */
	getParentPriceBook(): PriceBook | null
}
```
