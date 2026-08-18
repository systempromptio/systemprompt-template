# dw.order.LineItem

## Overview
Common line item base class.

## Description
Common line item base class.

## All Known Subclasses
GiftCertificateLineItem, PriceAdjustment, ProductLineItem, ProductShippingLineItem, ShippingLineItem

```ts
declare class LineItem extends ExtensibleObject {
	/** The base price for the line item (may be net or gross). */
	basePrice: Money

	/** The gross price for the line item, including tax. */
	grossPrice: Money

	/** The line item container (read-only). */
	/** @readonly */
	lineItemCtnr: LineItemCtnr

	/** The display text for the line item. */
	lineItemText: string

	/** The net price for the line item, excluding tax. */
	netPrice: Money

	/** Get the price for the line item (read-only). */
	/** @readonly */
	price: Money

	/** The numeric price value (same as getPrice().getValue()). */
	priceValue: number

	/** The tax amount for the line item. */
	tax: Money

	/** Tax basis used to calculate tax (read-only). */
	/** @readonly */
	taxBasis: Money

	/** The tax class ID or null. */
	taxClassID: string

	/** The decimal tax rate (e.g. 0.175 for 17.5%). */
	taxRate: number

	/** Returns the base price. */
	getBasePrice(): Money

	/** Returns the gross price. */
	getGrossPrice(): Money

	/** Returns the line item container. */
	getLineItemCtnr(): LineItemCtnr

	/** Returns the display text. */
	getLineItemText(): string

	/** Returns the net price. */
	getNetPrice(): Money

	/** Returns the price (net or gross depending on pricing policy). */
	getPrice(): Money

	/** Returns the numeric price value. */
	getPriceValue(): number

	/** Returns the tax amount. */
	getTax(): Money

	/** Returns the tax basis. */
	getTaxBasis(): Money

	/** Returns the tax class ID or null. */
	getTaxClassID(): string

	/** Returns the tax rate. */
	getTaxRate(): number

	/** Sets the base price. Deprecated: use updatePrice(Money). */
	setBasePrice(aValue: Money): void

	/** Sets the gross price. Deprecated: use updatePrice(Money). */
	setGrossPrice(aValue: Money): void

	/** Sets the display text. */
	setLineItemText(aText: string): void

	/** Sets the net price. Deprecated: use updatePrice(Money). */
	setNetPrice(aValue: Money): void

	/** Sets price attributes from a numeric value. */
	setPriceValue(value: number): void

	/** Sets the tax amount. */
	setTax(aValue: Money): void

	/** Sets the tax class ID. */
	setTaxClassID(aValue: string): void

	/** Sets the tax rate. */
	setTaxRate(taxRate: number): void

	/** Updates price attributes based on the given Money. Deprecated. */
	updatePrice(price: Money): void

	/** Updates tax using tax rate. */
	updateTax(taxRate: number): void

	/** Updates tax using tax rate and tax basis. */
	updateTax(taxRate: number, taxBasis: Money): void

	/** Updates the tax amount directly. */
	updateTaxAmount(tax: Money): void
}
```
