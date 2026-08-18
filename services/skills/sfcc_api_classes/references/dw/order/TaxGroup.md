# dw.order.TaxGroup

## Overview
Definition of a tax group with type, rate, caption and description.

## Description
TaxGroup contains metadata about a tax (type/key), an optional percentage `rate`, a human `caption` and a longer `description`. Includes a static `create` factory and getters for properties.

```ts
declare class TaxGroup  {
	/** Create a TaxGroup. @param taxType string tax type/key @param caption human caption @param description human description @param taxRate decimal (1.0 == 100%) */
	static create(taxType: string, caption: string, description: string, taxRate: Decimal): TaxGroup

	/** Gets the caption. */
	getCaption(): string

	/** Gets the description. */
	getDescription(): string

	/** Gets the percentage amount of the rate. */
	getRate(): number

	/** Gets the tax type (key). */
	getTaxType(): string

	/** Read-only: caption */
	readonly caption: string

	/** Read-only: description */
	readonly description: string

	/** Read-only: rate */
	readonly rate: number

	/** Read-only: taxType */
	readonly taxType: string
}
```
