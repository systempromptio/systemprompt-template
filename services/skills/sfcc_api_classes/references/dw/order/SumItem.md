# dw.order.SumItem

## Overview
Container representing a subtotal or grand total with prices and tax breakdown.

## Description
SumItem provides access to `grossPrice`, `netPrice`, `tax`, `taxBasis`, and an iterable `taxItems` collection. Use `getGrossPrice()`, `getNetPrice()`, `getTax()`, `getTaxBasis()` and `getTaxItems()` to read values.

```ts
declare class SumItem  {
	/** Gross price of SumItem. */
	getGrossPrice(): Money

	/** Net price of SumItem. */
	getNetPrice(): Money

	/** Total tax for SumItem. */
	getTax(): Money

	/** Price on which tax calculation is based. */
	getTaxBasis(): Money

	/** Tax items representing a tax breakdown for the SumItem. */
	getTaxItems(): Collection<TaxItem>

	/** Read-only property: grossPrice */
	readonly grossPrice: Money

	/** Read-only property: netPrice */
	readonly netPrice: Money

	/** Read-only property: tax */
	readonly tax: Money

	/** Read-only property: taxBasis */
	readonly taxBasis: Money

	/** Read-only property: taxItems */
	readonly taxItems: Collection<TaxItem>
}
```
