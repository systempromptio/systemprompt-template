# dw.order.TaxItem

## Overview
Represents a tax line tied to a TaxGroup with an amount.

## Description
TaxItem holds a `Money` amount and a reference to a `TaxGroup`. Use `getAmount()` and `getTaxGroup()` to read its values.

```ts
declare class TaxItem  {
	/** Gets the amount. */
	getAmount(): Money

	/** Returns the TaxGroup for this tax item. */
	getTaxGroup(): TaxGroup

	/** Read-only: amount */
	readonly amount: Money

	/** Read-only: taxGroup */
	readonly taxGroup: TaxGroup
}
```
