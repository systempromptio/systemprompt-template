# dw.campaign.CampaignStatusCodes

## Overview
Deprecated class formerly used to contain various coupon status codes.

## Description
This class is deprecated. Use CouponStatusCodes instead for coupon status constants.

```ts
declare class CampaignStatusCodes  {
	/**
	 * Indicates that the coupon has already been applied to the basket.
	 * @deprecated Use CouponStatusCodes.COUPON_CODE_ALREADY_IN_BASKET or CouponStatusCodes.COUPON_ALREADY_IN_BASKET instead.
	 */
	static COUPON_ALREADY_APPLIED: string

	/**
	 * Indicates that the coupon has already been redeemed.
	 * @deprecated Use CouponStatusCodes.COUPON_CODE_ALREADY_REDEEMED instead.
	 */
	static COUPON_ALREADY_REDEEMED: string

	/**
	 * Indicates that the coupon is not currently redeemable.
	 * @deprecated Use CouponStatusCodes.COUPON_DISABLED, CouponStatusCodes.COUPON_CODE_UNKNOWN, CouponStatusCodes.REDEMPTION_LIMIT_EXCEEDED, CouponStatusCodes.CUSTOMER_REDEMPTION_LIMIT_EXCEEDED, CouponStatusCodes.TIMEFRAME_REDEMPTION_LIMIT_EXCEEDED, or CouponStatusCodes.NO_APPLICABLE_PROMOTION instead.
	 */
	static COUPON_NOT_REDEEMABLE: string

	/**
	 * Indicates that the coupon code is not valid.
	 * @deprecated Use CouponStatusCodes.COUPON_CODE_UNKNOWN instead.
	 */
	static COUPON_UNKNOWN: string
}
```