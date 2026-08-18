# dw.order.BonusDiscountLineItem

## Overview
Placeholder line item representing an applied BonusChoiceDiscount; entitles a customer to select bonus products from a configured set.

## Description
Line item created by the promotions engine for BonusChoiceDiscounts. Acts as a placeholder to allow customers to add one or more bonus products to the basket from a configured list or rule-defined set. Provides access to the allowed bonus products, selected bonus product line items, promotion info, and limits on maximum bonus items.

## All Known Subclasses
(none)

```ts
declare class BonusDiscountLineItem extends PersistentObject {
	/** True when the promotion uses a rule to determine bonus products. */
	readonly bonusChoiceRuleBased: boolean

	/** Product line items representing the bonus products the customer selected. */
	readonly bonusProductLineItems: List

	/** List of bonus products the customer may choose from (may be empty for rule-based promotions). */
	readonly bonusProducts: List

	/** The coupon line item associated with this discount. */
	readonly couponLineItem: CouponLineItem

	/** Maximum number of bonus items allowed to select. */
	readonly maxBonusItems: number

	/** The promotion associated with this discount. */
	readonly promotion: Promotion

	/** The promotion ID associated with this discount. */
	readonly promotionID: string

	getBonusProductLineItems(): List
	getBonusProductPrice(product: Product): Money
	getBonusProducts(): List
	getCouponLineItem(): CouponLineItem
	getMaxBonusItems(): number
	getPromotion(): Promotion
	getPromotionID(): string
	isBonusChoiceRuleBased(): boolean
}
```
