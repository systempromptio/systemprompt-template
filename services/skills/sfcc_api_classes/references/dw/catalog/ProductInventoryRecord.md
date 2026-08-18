# dw.catalog.ProductInventoryRecord

## Overview
Represents inventory and availability data for a single product. Includes allocation, on-order, ATS, stock level, turnover, and reservation quantities.

## Description
Holds inventory details for a product. Many properties are read-only, and under Omnichannel Inventory (OCI) several fields map directly to OCI concepts (e.g., ATS, ATF). When OCI is enabled, setters throw exceptions and custom attributes are unsupported.


```ts
declare class ProductInventoryRecord extends dw.object.ExtensibleObject {
	/** The allocation quantity currently set. */
	static allocation: dw.value.Quantity

	/** Date when allocation was initialized or reset. */
	static allocationResetDate: Date

	/** Available-to-sell quantity (ATS). */
	static ATS: dw.value.Quantity

	/** Whether the product is backorderable. */
	static backorderable: boolean

	/** Custom attributes object (not supported under OCI). */
	static custom: dw.object.CustomAttributes

	/** Expected in-stock date. */
	static inStockDate: Date

	/** On-hand quantity (deprecated) use `getStockLevel()`. */
	static onHand: dw.value.Quantity

	/** Quantity currently on order. */
	static onOrder: dw.value.Quantity

	/** Whether the product is perpetual. */
	static perpetual: boolean

	/** Whether the product is preorderable. */
	static preorderable: boolean

	/** Quantity allocated for preorder/backorder. */
	static preorderBackorderAllocation: dw.value.Quantity

	/** Quantity reserved. */
	static reserved: dw.value.Quantity

	/** Current stock level (allocation - turnover). */
	static stockLevel: dw.value.Quantity

	/** Sum of inventory transactions since allocation reset. */
	static turnover: dw.value.Quantity

	/** Returns metadata about this object. */
	describe(): dw.object.ObjectTypeDefinition

	getAllocation(): dw.value.Quantity

	getAllocationResetDate(): Date

	getATS(): dw.value.Quantity

	getCustom(): dw.object.CustomAttributes

	getInStockDate(): Date

	getOnHand(): dw.value.Quantity

	getOnOrder(): dw.value.Quantity

	getPreorderBackorderAllocation(): dw.value.Quantity

	getReserved(): dw.value.Quantity

	getStockLevel(): dw.value.Quantity

	getTurnover(): dw.value.Quantity

	isBackorderable(): boolean

	isPerpetual(): boolean

	isPreorderable(): boolean

	setAllocation(quantity: number): void

	setAllocation(quantity: number, allocationResetDate: Date): void

	setBackorderable(backorderableFlag: boolean): void

	setInStockDate(inStockDate: Date): void

	setPerpetual(perpetualFlag: boolean): void

	setPreorderable(preorderableFlag: boolean): void

	setPreorderBackorderAllocation(quantity: number): void
}
```
