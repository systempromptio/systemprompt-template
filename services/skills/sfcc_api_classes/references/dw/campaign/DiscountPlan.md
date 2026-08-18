# dw.campaign.DiscountPlan

## Overview
Container for a set of discounts applied to a line item container.

## Description
Represents a collection of discounts returned by PromotionMgr for applicable promotions on a line item container. Provides access to different types of discounts and methods to manage the discount plan.

```ts
declare class DiscountPlan  {
	/**
	 * Order discounts that the LineItemCtnr "almost" qualifies for based on merchandise total.
	 */
	readonly approachingOrderDiscounts: Collection

	/**
	 * All bonus discounts contained in the discount plan.
	 */
	readonly bonusDiscounts: Collection

	/**
	 * Line item container associated with discount plan.
	 */
	readonly lineItemCtnr: LineItemCtnr

	/**
	 * Percentage and amount order discounts contained in the discount plan.
	 */
	readonly orderDiscounts: Collection

	/**
	 * Get the collection of order discounts that the LineItemCtnr "almost" qualifies for.
	 * @returns Collection of approaching order discounts ordered by condition threshold ascending
	 */
	getApproachingOrderDiscounts(): Collection

	/**
	 * Get approaching shipping discounts for the passed shipment.
	 * @param shipment The shipment to calculate approaching discounts for
	 * @returns Collection of approaching shipping discounts ordered by condition threshold ascending
	 */
	getApproachingShippingDiscounts(shipment: Shipment): Collection

	/**
	 * Get approaching shipping discounts for shipment filtered by shipping method.
	 * @param shipment The shipment to calculate approaching discounts for
	 * @param shippingMethod The shipping method to filter by
	 * @returns Collection of approaching shipping discounts ordered by condition threshold ascending
	 */
	getApproachingShippingDiscounts(shipment: Shipment, shippingMethod: ShippingMethod): Collection

	/**
	 * Get approaching shipping discounts for shipment filtered by shipping methods.
	 * @param shipment The shipment to calculate approaching discounts for
	 * @param shippingMethods The shipping methods to filter by
	 * @returns Collection of approaching shipping discounts ordered by condition threshold ascending
	 */
	getApproachingShippingDiscounts(shipment: Shipment, shippingMethods: Collection): Collection

	/**
	 * Returns all bonus discounts contained in the discount plan.
	 * @returns All bonus discounts contained in discount plan
	 */
	getBonusDiscounts(): Collection

	/**
	 * Returns line item container associated with discount plan.
	 * @returns Line item container associated with plan
	 */
	getLineItemCtnr(): LineItemCtnr

	/**
	 * Returns the percentage and amount order discounts contained in the discount plan.
	 * @returns Order discounts contained in the discount plan
	 */
	getOrderDiscounts(): Collection

	/**
	 * Returns the percentage, amount and fix price discounts associated with the specified product line item.
	 * @param productLineItem Product line item
	 * @returns Discounts associated with specified product line item
	 */
	getProductDiscounts(productLineItem: ProductLineItem): Collection

	/**
	 * Returns the product-shipping discounts associated with the specified product line item.
	 * @param productLineItem Product line item
	 * @returns Product-shipping discounts associated with specified product line item
	 */
	getProductShippingDiscounts(productLineItem: ProductLineItem): Collection

	/**
	 * Returns the percentage, amount and fix price discounts associated with the specified shipment.
	 * @param shipment the shipment for which to fetch discounts
	 * @returns Discounts associated with specified shipment
	 */
	getShippingDiscounts(shipment: Shipment): Collection

	/**
	 * Removes the specified discount from the discount plan.
	 * @param discount Discount to be removed
	 */
	removeDiscount(discount: Discount): void
}
```