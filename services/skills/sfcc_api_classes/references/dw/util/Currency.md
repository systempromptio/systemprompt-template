# dw.util.Currency

## Overview
Represents a currency supported by the system with ISO 4217 code, symbol, name, and fraction digits.

## Description
Represents a currency supported by the system.

```
Object
  dw.util.Currency
```

```ts
declare class Currency  {
	/**
	 * Gets the ISO 4217 mnemonic currency code of this currency.
	 */
	readonly currencyCode: string

	/**
	 * Gets the default number of fraction digits used with this currency. For example, the default number of fraction digits for the Euro is 2, while for the Japanese Yen it's 0.
	 */
	readonly defaultFractionDigits: number

	/**
	 * Gets a long name for this currency. e.g. "United States Dollar". The returned name is the one stored in the system for this currency. Currently only English names are available, but in the future this method may return a locale-specific name.
	 */
	readonly name: string

	/**
	 * Gets the symbol of this currency. e.g. "$" for the US Dollar.
	 */
	readonly symbol: string

	/**
	 * Returns a Currency instance for the given currency code, or null if there is no such currency.
	 * @param currencyCode - the ISO 4217 mnemonic code of the currency
	 * @returns the Currency instance for the given currency code
	 */
	static getCurrency(currencyCode: string): Currency

	/**
	 * Gets the ISO 4217 mnemonic currency code of this currency.
	 * @returns the ISO 4217 mnemonic currency code of this currency
	 */
	getCurrencyCode(): string

	/**
	 * Gets the default number of fraction digits used with this currency. For example, the default number of fraction digits for the Euro is 2, while for the Japanese Yen it's 0.
	 * @returns the default number of fraction digits used with this currency
	 */
	getDefaultFractionDigits(): number

	/**
	 * Gets a long name for this currency. e.g. "United States Dollar". The returned name is the one stored in the system for this currency. Currently only English names are available, but in the future this method may return a locale-specific name.
	 * @returns a long name for this currency
	 */
	getName(): string

	/**
	 * Gets the symbol of this currency. e.g. "$" for the US Dollar.
	 * @returns the symbol of this currency
	 */
	getSymbol(): string

	/**
	 * Returns the ISO 4217 mnemonic currency code of this currency.
	 * @returns the ISO 4217 mnemonic currency code of this currency
	 */
	toString(): string
}
```
