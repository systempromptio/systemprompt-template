# dw.order.ReturnCase

## Overview
Represents a return case (RMA) grouping return items and returns for an order.

## Description
A ReturnCase is a container for returns associated with an Order. It holds ReturnCaseItems and Returns and tracks status (NEW, CONFIRMED, PARTIAL_RETURNED, RETURNED, CANCELLED). Use it to create Returns or Invoice documents for returned goods.

## All Known Subclasses
 (none)

```ts
declare class ReturnCase extends dw.object.Extensible {
	/** Sorting by item id. Use with getItems() and FilteringCollection.sort(Object). */
	static ORDERBY_ITEMID: unknown

	/** Sorting by the position of the related order item. */
	static ORDERBY_ITEMPOSITION: unknown

	/** Unsorted ordering. */
	static ORDERBY_UNSORTED: unknown

	/** Qualifier selecting product items. */
	static QUALIFIER_PRODUCTITEMS: unknown

	/** Qualifier selecting service items. */
	static QUALIFIER_SERVICEITEMS: unknown

	/** constant for ReturnCase Status CANCELLED */
	static STATUS_CANCELLED: 'CANCELLED'

	/** constant for ReturnCase Status CONFIRMED */
	static STATUS_CONFIRMED: 'CONFIRMED'

	/** constant for ReturnCase Status NEW */
	static STATUS_NEW: 'NEW'

	/** constant for ReturnCase Status PARTIAL_RETURNED */
	static STATUS_PARTIAL_RETURNED: 'PARTIAL_RETURNED'

	/** constant for ReturnCase Status RETURNED */
	static STATUS_RETURNED: 'RETURNED'

	/** Returns null or the previously created Invoice. */
	getInvoice(): dw.order.Invoice | null

	/** Returns null or the invoice-number. */
	getInvoiceNumber(): string | null

	/** Access the collection of ReturnCaseItem objects (FilteringCollection). */
	getItems(): dw.util.FilteringCollection

	/** The mandatory return case number identifying this document. */
	getReturnCaseNumber(): string

	/** Return collection of Returns associated with this ReturnCase. */
	getReturns(): dw.util.Collection

	/** Returns whether this is an RMA (read-only). */
	isRMA(): boolean

	/** Gets the return case status (read-only). */
	getStatus(): dw.value.EnumValue

	/** Attempt to confirm the ReturnCase. Throws IllegalStateException if not STATUS_NEW. */
	confirm(): void

	/** Creates a new Invoice based on this ReturnCase. */
	createInvoice(): dw.order.Invoice

	/** Creates a new Invoice with specified number. */
	createInvoice(invoiceNumber: string): dw.order.Invoice

	/** Creates a new ReturnCaseItem for a given order item id. */
	createItem(orderItemID: string): dw.order.ReturnCaseItem

	/** Creates and associates a new Return with given number. */
	createReturn(returnNumber: string): dw.order.Return

	/** Creates a new Return with a generated number. */
	createReturn(): dw.order.Return

}
```
