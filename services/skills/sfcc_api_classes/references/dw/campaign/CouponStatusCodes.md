# dw.campaign.CouponStatusCodes

## Overview
Helper class providing status codes for coupon validation failures when adding to cart or checking redemption validity.

## Description
Contains constants indicating why a coupon code cannot be added to cart or why an existing coupon in cart is no longer valid for redemption.

```ts
declare class CouponStatusCodes  {
	/**
	 * Coupon is valid for redemption and assigned to applicable promotions.
	 */
	static APPLIED: 'APPLIED'

	/**
	 * Another code from the same multi-code/system coupon already in basket.
	 */
	static COUPON_ALREADY_IN_BASKET: 'COUPON_ALREADY_IN_BASKET'

	/**
	 * Coupon code already added to basket.
	 */
	static COUPON_CODE_ALREADY_IN_BASKET: 'COUPON_CODE_ALREADY_IN_BASKET'

	/**
	 * Multi-code/system coupon code already redeemed.
	 */
	static COUPON_CODE_ALREADY_REDEEMED: 'COUPON_CODE_ALREADY_REDEEMED'

	/**
	 * Coupon not found for code or code not found.
	 */
	static COUPON_CODE_UNKNOWN: 'COUPON_CODE_UNKNOWN'

	/**
	 * Coupon is not enabled.
	 */
	static COUPON_DISABLED: 'COUPON_DISABLED'

	/**
	 * Redemptions per code and customer exceeded.
	 */
	static CUSTOMER_REDEMPTION_LIMIT_EXCEEDED: 'CUSTOMER_REDEMPTION_LIMIT_EXCEEDED'

	/**
	 * Coupon not assigned to an active promotion.
	 */
	static NO_ACTIVE_PROMOTION: 'NO_ACTIVE_PROMOTION'

	/**
	 * Coupon assigned to active promotions but none currently applicable.
	 */
	static NO_APPLICABLE_PROMOTION: 'NO_APPLICABLE_PROMOTION'

	/**
	 * Redemptions per code exceeded (typically single-code coupons).
	 */
	static REDEMPTION_LIMIT_EXCEEDED: 'REDEMPTION_LIMIT_EXCEEDED'

	/**
	 * Redemptions per code, customer, and time-frame exceeded.
	 */
	static TIMEFRAME_REDEMPTION_LIMIT_EXCEEDED: 'TIMEFRAME_REDEMPTION_LIMIT_EXCEEDED'
}
```
