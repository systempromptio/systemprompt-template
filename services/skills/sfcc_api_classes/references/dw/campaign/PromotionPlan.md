# dw.campaign.PromotionPlan

## Overview
Represents a collection of promotions (active or upcoming) used to display promotions on storefront pages or to calculate discounts via PromotionMgr.

## Description
PromotionPlan exposes sorted collections of promotions and helper methods to query promotions by type, product, payment method, or shipping method. The class also provides constants to control sort order.

```ts
declare class PromotionPlan  {
	/**
	 * Sort by exclusivity then rank, promotion class, discount type, best discount, and ID.
	 */
	static SORT_BY_EXCLUSIVITY: Number = 1

	/**
	 * Sort by start date ascending.
	 */
	static SORT_BY_START_DATE: Number = 2

	/**
	 * Collection of order promotions in the plan.
	 */
	orderPromotions: dw.util.Collection

	/**
	 * Collection of product promotions in the plan.
	 */
	productPromotions: dw.util.Collection

	/**
	 * All promotions in the plan, sorted by exclusivity.
	 */
	promotions: dw.util.Collection

	/**
	 * Collection of shipping promotions in the plan.
	 */
	shippingPromotions: dw.util.Collection

	/**
	 * Returns promotions filtered as order promotions.
	 */
	getOrderPromotions(): dw.util.Collection

	/**
	 * Returns promotions for a given payment card.
	 */
	getPaymentCardPromotions(paymentCard: dw.order.PaymentCard): dw.util.Collection

	/**
	 * Returns promotions for a given payment method.
	 */
	getPaymentMethodPromotions(paymentMethod: dw.order.PaymentMethod): dw.util.Collection

	/**
	 * Returns product promotions in the plan.
	 */
	getProductPromotions(): dw.util.Collection

	/**
	 * Returns promotions related to a product.
	 */
	getProductPromotions(product: dw.catalog.Product): dw.util.Collection

	/**
	 * Remove a promotion from this plan.
	 */
	removePromotion(promotion: dw.campaign.Promotion): void
}
```
