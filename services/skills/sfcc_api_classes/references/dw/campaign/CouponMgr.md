# dw.campaign.CouponMgr

## Overview
Manager class for accessing and managing coupons in the system.

## Description
Provides static methods to retrieve coupons by ID or code, get all coupons for a site, access redemption information, and perform data masking operations on coupon redemptions.

```ts
declare class CouponMgr  {
	/**
	 * Indicates that an error occurred because a valid data domain cannot be found for given siteID.
	 */
	static MR_ERROR_INVALID_SITE_ID: string

	/**
	 * All coupons in the current site in no specific order.
	 */
	static readonly coupons: Collection

	/**
	 * Returns the coupon with the specified ID.
	 * @param couponID the coupon identifier
	 * @returns Coupon with specified ID or null
	 */
	static getCoupon(couponID: string): Coupon

	/**
	 * Tries to find a coupon for the given coupon code.
	 * @param couponCode The coupon code to get the coupon for
	 * @returns The coupon with the matching coupon code or null if no coupon was found
	 */
	static getCouponByCode(couponCode: string): Coupon

	/**
	 * Returns all coupons in the current site in no specific order.
	 * @returns Coupons in current site
	 */
	static getCoupons(): Collection

	/**
	 * Returns list of CouponRedemptions for the specified coupon and coupon code, sorted by redemption date descending.
	 * @param couponID The coupon id to find redemption for
	 * @param couponCode The coupon code to find redemption for
	 * @returns A sorted list of CouponRedemptions for the specified coupon and coupon code or an empty list if no redemption record exists
	 */
	static getRedemptions(couponID: string, couponCode: string): Collection

	/**
	 * Mask customer email address in coupon redemptions for the given siteID and email address.
	 * @param siteID the site ID
	 * @param email the customer email address
	 * @returns The status of the masking result
	 */
	static maskRedemptions(siteID: string, email: string): Status
}
```