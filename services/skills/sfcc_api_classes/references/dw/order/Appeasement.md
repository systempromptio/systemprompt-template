# dw.order.Appeasement

## Overview
Represents a shopper request for an order credit; contains appeasement items tied to order items and supports statuses like OPEN and COMPLETED.

## Description
An Appeasement represents a shopper request for an order credit. It contains one or more appeasement items, each associated with an OrderItem (typically a ProductLineItem). Appeasements have statuses (OPEN, COMPLETED). Provides methods to add items, create an Invoice from the appeasement, and access metadata such as reason code, note, and status.

## All Known Subclasses
(none)

```ts
declare class Appeasement extends AbstractItemCtnr {
	/** ORDERBY_ITEMID - sort by item id (use with FilteringCollection.sort). */
	static ORDERBY_ITEMID: object

	/** ORDERBY_ITEMPOSITION - sort by related order item position. */
	static ORDERBY_ITEMPOSITION: object

	/** ORDERBY_UNSORTED - unsorted as-is. */
	static ORDERBY_UNSORTED: object

	/** QUALIFIER_PRODUCTITEMS - select product items. */
	static QUALIFIER_PRODUCTITEMS: object

	/** QUALIFIER_SERVICEITEMS - select service items. */
	static QUALIFIER_SERVICEITEMS: object

	/** Constant for Appeasement Status COMPLETED */
	static STATUS_COMPLETED: 'COMPLETED'

	/** Constant for Appeasement Status OPEN */
	static STATUS_OPEN: 'OPEN'

	/** The appeasement number. */
	readonly appeasementNumber: string

	/** Returns null or the previously created Invoice. */
	readonly invoice: Invoice

	/** Returns null or the invoice number. */
	readonly invoiceNumber: string

	/** A filtering collection of the appeasement items. */
	readonly items: FilteringCollection

	/** The reason code for the appeasement. */
	readonly reasonCode: EnumValue

	/** The reason note for the appeasement. */
	readonly reasonNote: string

	/** Gets the status of this appeasement. */
	readonly status: EnumValue

	addItems(totalAmount: Money, orderItems: List): void
	createInvoice(): Invoice
	createInvoice(invoiceNumber: string): Invoice
	getAppeasementNumber(): string
	getInvoice(): Invoice
	getInvoiceNumber(): string
	getItems(): FilteringCollection
	getReasonCode(): EnumValue
	getReasonNote(): string
	getStatus(): EnumValue
	setReasonCode(reasonCode: string): void
	setReasonNote(reasonNote: string): void
	setStatus(appeasementStatus: string): void
}
```
