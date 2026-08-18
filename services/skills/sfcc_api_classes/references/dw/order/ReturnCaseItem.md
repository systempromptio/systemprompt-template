# dw.order.ReturnCaseItem

## Overview
Represents an item within a ReturnCase, tracks authorized quantity, status, and related returns.

## Description
A ReturnCaseItem links to an OrderItem and defines the quantity authorized for return. It holds metadata like base price, reason code, note, parent item, and the collection of ReturnItems created for this case item.

```ts
declare class ReturnCaseItem extends dw.object.Extensible {
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

	/** Authorized quantity for this ReturnCaseItem (may be N/A). */
	getAuthorizedQuantity(): dw.value.Quantity

	/** Price of a single unit before discount application (read-only). */
	getBasePrice(): dw.value.Money

	/** Note for this return case item. */
	getNote(): string

	/** Parent item or null. */
	getParentItem(): dw.order.ReturnCaseItem | null

	/** Reason code for the return case item. */
	getReasonCode(): dw.value.EnumValue

	/** ReturnCase number this item belongs to (read-only). */
	getReturnCaseNumber(): string

	/** Unsorted collection of ReturnItem objects associated (read-only). */
	getReturnItems(): dw.util.Collection

	/** Gets the status (EnumValue). */
	getStatus(): dw.value.EnumValue

	/** Create a new ReturnItem for this ReturnCaseItem assigned to the given Return. */
	createReturnItem(returnNumber: string): dw.order.ReturnItem

	/** Set the authorized quantity for this item. */
	setAuthorizedQuantity(authorizedQuantity: dw.value.Quantity): void

	/** Set a note for this return case item. */
	setNote(note: string): void

	/** Set a parent item. */
	setParentItem(parentItem: dw.order.ReturnCaseItem): void

	/** Set reason code by string. */
	setReasonCode(reasonCode: string): void

	/** Set status string. */
	setStatus(statusString: string): void

}
```
