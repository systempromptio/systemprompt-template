# dw.campaign.CampaignMgr

## Overview
Manager for campaigns and promotions. All methods are deprecated; use PromotionMgr instead.

## Description
Provides static methods to retrieve campaigns and promotions, and apply promotions to line item containers. All functionality has been superseded by PromotionMgr and PromotionPlan.

```ts
declare class CampaignMgr  {
	/**
	 * @deprecated Use PromotionMgr.getActiveCustomerPromotions() and PromotionPlan.getPromotions()
	 * Enabled promotions of active campaigns applicable for current customer and source code (excludes coupon-based promotions).
	 */
	readonly applicablePromotions: Collection

	/**
	 * @deprecated Use PromotionMgr instead
	 * Returns true if line item container contains bonus product line items.
	 */
	static applyBonusPromotions(lineItemCtnr: LineItemCtnr, promotions: Collection): boolean

	/**
	 * @deprecated Use PromotionMgr
	 * Applies applicable order promotions to the line item container.
	 */
	static applyOrderPromotions(lineItemCtnr: LineItemCtnr, promotions: Collection): boolean

	/**
	 * @deprecated Use PromotionMgr
	 * Applies applicable product promotions to the line item container.
	 */
	static applyProductPromotions(lineItemCtnr: LineItemCtnr, promotions: Collection): boolean

	/**
	 * @deprecated Use PromotionMgr
	 * Applies applicable shipping promotions to the line item container.
	 */
	static applyShippingPromotions(lineItemCtnr: LineItemCtnr, promotions: Collection): boolean

	/**
	 * @deprecated Use PromotionMgr.getActiveCustomerPromotions() and PromotionPlan.getProductPromotions(Product)
	 * Returns enabled promotions of active campaigns for current customer and source code where product is qualifying.
	 */
	static getApplicableConditionalPromotions(product: Product): Collection

	/**
	 * @deprecated Use PromotionMgr.getActiveCustomerPromotions() and PromotionPlan.getProductPromotions(Product)
	 * Returns enabled promotions of active campaigns for current customer and source code where product is discounted (excludes coupon-based).
	 */
	static getApplicablePromotions(product: Product): Collection

	/**
	 * @deprecated No replacement provided
	 * Returns enabled promotions of active campaigns for current customer, source code, and coupons in line item container.
	 */
	static getApplicablePromotions(lineItemCtnr: LineItemCtnr): Collection

	/**
	 * @deprecated Use PromotionMgr.getActiveCustomerPromotions() and PromotionPlan.getPromotions()
	 * Returns enabled promotions of active campaigns for current customer and source code (excludes coupon-based).
	 */
	static getApplicablePromotions(): Collection

	/**
	 * @deprecated Use PromotionMgr.getCampaign(String)
	 * Returns campaign by ID.
	 */
	static getCampaignByID(id: string): Campaign | null

	/**
	 * @deprecated Use PromotionMgr.getActivePromotions() and PromotionPlan.getProductPromotions(Product)
	 * Returns enabled promotions of active campaigns where product is qualifying (includes coupon-based).
	 */
	static getConditionalPromotions(product: Product): Collection

	/**
	 * @deprecated Coupons now relate to multiple promotions; returns first promotion only
	 * Returns promotion associated with coupon code.
	 */
	static getPromotion(couponCode: string): Promotion | null

	/**
	 * @deprecated Coupons now relate to multiple promotions; returns first promotion only
	 * Returns promotion associated with coupon code.
	 */
	static getPromotionByCouponCode(couponCode: string): Promotion | null

	/**
	 * @deprecated Use PromotionMgr.getPromotion(String)
	 * Returns promotion by ID.
	 */
	static getPromotionByID(id: string): Promotion | null

	/**
	 * @deprecated Use PromotionMgr.getActivePromotions() and PromotionPlan.getProductPromotions(Product)
	 * Returns enabled promotions of active campaigns where product is discounted (customer groups/source codes only).
	 */
	static getPromotions(product: Product): Collection
}
```
