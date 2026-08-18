# dw.campaign.BonusDiscount

## Overview
Represents a bonus discount in the discount plan, e.g., "Get a free DVD with your purchase of any DVD player."

## Description
Provides bonus products associated with a discount that are in stock, online, and assigned to the site catalog.

```ts
declare class BonusDiscount extends Discount {
	/**
	 * Bonus products associated with this discount that are in stock, online, and assigned to site catalog.
	 */
	readonly bonusProducts: Collection

	/**
	 * Returns bonus products associated with this discount that are in stock, online, and assigned to site catalog.
	 */
	getBonusProducts(): Collection
}
```
