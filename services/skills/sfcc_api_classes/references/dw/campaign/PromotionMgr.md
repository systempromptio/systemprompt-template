# dw.campaign.PromotionMgr

## Overview
Static manager for retrieving and applying promotions and discounts. Provides methods to fetch active/upcoming promotions, calculate discounts, and apply them to line item containers.

## Description
PromotionMgr orchestrates the three-step promotion application process: determine active customer promotions, calculate applicable discounts, and apply discounts to a line item container. It also offers granular methods to inspect campaigns and promotion plans.

```ts
declare class PromotionMgr  {
	/**
	 * All promotions scheduled for now and applicable for session/customer/source code.
	 */
	activeCustomerPromotions: dw.campaign.PromotionPlan

	/**
	 * All promotions scheduled for now regardless of customer or source code.
	 */
	activePromotions: dw.campaign.PromotionPlan

	/**
	 * All campaigns of the current site.
	 */
	campaigns: dw.util.Collection

	/**
	 * All promotions of the current site.
	 */
	promotions: dw.util.Collection

	/**
	 * Identifies active promotions, calculates applicable discounts and applies them to a line item container.
	 */
	static applyDiscounts(lineItemCtnr: dw.order.LineItemCtnr): void

	/**
	 * Applies discounts from a prepared discount plan to its associated line item container.
	 */
	static applyDiscounts(discountPlan: dw.campaign.DiscountPlan): void

	/**
	 * Returns active customer promotions as a PromotionPlan.
	 */
	static getActiveCustomerPromotions(ignoreCouponCondition?: boolean): dw.campaign.PromotionPlan

	/**
	 * Returns all campaigns of the current site.
	 */
	static getCampaigns(): dw.util.Collection

	/**
	 * Returns the campaign identified by the specified ID.
	 */
	static getCampaign(id: String): dw.campaign.Campaign

	/**
	 * Returns discounts applicable for the specified line item container.
	 */
	static getDiscounts(lineItemCtnr: dw.order.LineItemCtnr, promotionPlan?: dw.campaign.PromotionPlan): dw.campaign.DiscountPlan

	/**
	 * Returns the promotion with the specified ID.
	 */
	static getPromotion(id: String): dw.campaign.Promotion

	/**
	 * Returns all promotions of the current site.
	 */
	static getPromotions(): dw.util.Collection

	/**
	 * Returns upcoming promotions (preview time in hours) as a PromotionPlan.
	 */
	static getUpcomingPromotions(previewTime: Number): dw.campaign.PromotionPlan
}
```
