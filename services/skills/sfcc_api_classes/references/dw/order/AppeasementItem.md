# dw.order.AppeasementItem

## Overview
Represents an item within an Appeasement tied to a single OrderItem; supports parent/child relationships between appeasement items.

## Description
An AppeasementItem represents a single item of an Appeasement and is associated with one OrderItem (typically a ProductLineItem). Items are created via Appeasement.addItems. When the related Appeasement is COMPLETED only custom attributes may be changed. Supports parent-child relationships with limits to prevent cycles and excessive depth.

## All Known Subclasses
(none)

```ts
declare class AppeasementItem extends AbstractItem {
	/** The number of the Appeasement to which this item belongs. */
	readonly appeasementNumber: string

	/** Returns null or the parent item. */
	readonly parentItem: AppeasementItem

	getAppeasementNumber(): string
	getParentItem(): AppeasementItem
	setParentItem(parentItem: AppeasementItem): void
}
```
