# dw.campaign.Discount

## Overview
Base class for all discount types in the campaign system.

## Description
Superclass providing common properties and methods for all specific discount implementations. Contains discount type constants and promotion-related information.

## All Known Subclasses
AmountDiscount, BonusChoiceDiscount, BonusDiscount, FixedPriceDiscount, FixedPriceShippingDiscount, FreeDiscount, FreeShippingDiscount, PercentageDiscount, PercentageOptionDiscount, PriceBookPriceDiscount, TotalFixedPriceDiscount

```ts
declare class Discount  {
	/**
	 * Constant representing discounts of type amount.
	 */
	static TYPE_AMOUNT: string

	/**
	 * Constant representing discounts of type bonus.
	 */
	static TYPE_BONUS: string

	/**
	 * Constant representing discounts of type bonus choice.
	 */
	static TYPE_BONUS_CHOICE: string

	/**
	 * Constant representing discounts of type fixed-price.
	 */
	static TYPE_FIXED_PRICE: string

	/**
	 * Constant representing discounts of type fixed price shipping.
	 */
	static TYPE_FIXED_PRICE_SHIPPING: string

	/**
	 * Constant representing discounts of type free.
	 */
	static TYPE_FREE: string

	/**
	 * Constant representing discounts of type free shipping.
	 */
	static TYPE_FREE_SHIPPING: string

	/**
	 * Constant representing discounts of type percentage.
	 */
	static TYPE_PERCENTAGE: string

	/**
	 * Constant representing discounts of type percent off options.
	 */
	static TYPE_PERCENTAGE_OFF_OPTIONS: string

	/**
	 * Constant representing discounts of type price book price.
	 */
	static TYPE_PRICEBOOK_PRICE: string

	/**
	 * Constant representing discounts of type total fixed price.
	 */
	static TYPE_TOTAL_FIXED_PRICE: string

	/**
	 * The tier index by quantity Id of Product promotion.
	 */
	readonly itemPromotionTiers: Map

	/**
	 * The promotion this discount is based on.
	 */
	readonly promotion: Promotion

	/**
	 * The tier index for promotion at order level or bonus product.
	 */
	readonly promotionTier: number

	/**
	 * The quantity of the discount.
	 */
	readonly quantity: number

	/**
	 * The type of the discount.
	 */
	readonly type: string

	/**
	 * Returns the tier index by quantity Id of Product promotion.
	 * @returns Map of Tier index by quantity Id or empty map
	 */
	getItemPromotionTiers(): Map

	/**
	 * Returns the promotion this discount is based on.
	 * @returns Promotion related to this discount
	 */
	getPromotion(): Promotion

	/**
	 * Returns the tier index for promotion at order level or bonus product.
	 * @returns Tier index or null
	 */
	getPromotionTier(): number

	/**
	 * Returns the quantity of the discount.
	 * @returns Discount quantity
	 */
	getQuantity(): number

	/**
	 * Returns the type of the discount.
	 * @returns Discount type
	 */
	getType(): string
}
```