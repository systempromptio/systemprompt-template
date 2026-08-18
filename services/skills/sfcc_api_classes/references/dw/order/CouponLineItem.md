# dw.order.CouponLineItem

## Overview
Represents a redeemed coupon stored on a Basket; exposes status and related price adjustments.

## Description
The CouponLineItem class is used to store redeemed coupons in the Basket. It provides access
to the coupon code, its applied status, related price adjustments and (deprecated) promotion info.

```ts
declare class CouponLineItem extends ExtensibleObject {
    /** Identifies if the coupon is currently applied in the basket. */
    applied: boolean

    /** True if the line item represents a coupon associated with a campaign. */
    basedOnCampaign: boolean

    /** Bonus discount line items triggered by this coupon. */
    bonusDiscountLineItems: Collection

    /** The coupon code. */
    couponCode: string

    /** Price adjustments triggered by this coupon. */
    priceAdjustments: Collection

    /** The promotion related to the coupon line item (deprecated). */
    promotion: Promotion

    /** The id of the related promotion (deprecated). */
    promotionID: string

    /** Detailed error/status code for this coupon line item. */
    statusCode: string

    /** True when the coupon line item is considered valid. */
    valid: boolean

    /**
     * Associates the specified price adjustment with this coupon line item.
     * @param priceAdjustment PriceAdjustment to associate with the coupon line item.
     */
    associatePriceAdjustment(priceAdjustment: PriceAdjustment): void

    /** Returns the bonus discount line items triggered by this coupon. */
    getBonusDiscountLineItems(): Collection

    /** Returns the coupon code. */
    getCouponCode(): string

    /** Returns the price adjustments triggered by this coupon. */
    getPriceAdjustments(): Collection

    /**
     * Returns the promotion related to the coupon line item.
     * @deprecated A coupon code may be associated with multiple promotions; returns one for backward-compat.
     */
    getPromotion(): Promotion

    /**
     * Returns the id of a related promotion.
     * @deprecated May return an arbitrary associated promotion id for backward compatibility.
     */
    getPromotionID(): string

    /** Returns a detailed error/status code for this coupon line item. */
    getStatusCode(): string

    /** True if the coupon is currently applied in the basket. */
    isApplied(): boolean

    /** True if the line item represents a campaign coupon (false for custom codes). */
    isBasedOnCampaign(): boolean

    /** True if the coupon code is valid (status in APPLIED or NO_APPLICABLE_PROMOTION). */
    isValid(): boolean

}
```
