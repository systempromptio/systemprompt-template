# dw.order.ReturnItem

## Overview
Represents a physically returned order line item and related return data.

## Description
ReturnItem represents an item within a Return document. It holds returned quantity, reason code, base price, link to its ReturnCaseItem, and supports tax item manipulation and price adjustments.

```ts
declare class ReturnItem extends dw.object.Extensible {
	/** Price of a single unit before discount application (read-only). */
	getBasePrice(): dw.value.Money

	/** Note for this return item. */
	getNote(): string

	/** Parent ReturnItem or null. */
	getParentItem(): dw.order.ReturnItem | null

	/** Reason code for return item. */
	getReasonCode(): dw.value.EnumValue

	/** Related ReturnCaseItem (read-only). */
	getReturnCaseItem(): dw.order.ReturnCaseItem

	/** Quantity returned (may be N/A). */
	getReturnedQuantity(): dw.value.Quantity

	/** Return number this item belongs to (read-only). */
	getReturnNumber(): string

	/** Create and add a tax item to this return item. */
	addTaxItem(amount: dw.util.Decimal, taxGroup: dw.order.TaxGroup): dw.order.TaxItem

	/** Apply a price rate to the item prices. */
	applyPriceRate(factor: dw.util.Decimal, divisor: dw.util.Decimal, roundUp: boolean): void

	/** Setters for note, parent item, reason code, returned quantity, tax basis and tax items. */
	setNote(note: string): void

	setParentItem(parentItem: dw.order.ReturnItem): void

	setReasonCode(reasonCode: string): void

	setReturnedQuantity(quantity: dw.value.Quantity): void

	setTaxBasis(taxBasis: dw.value.Money): void

	setTaxItems(taxItems: dw.util.Collection): void

}
```
