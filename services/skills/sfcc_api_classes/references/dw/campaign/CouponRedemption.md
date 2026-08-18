# dw.campaign.CouponRedemption

## Overview
Represents a redeemed coupon with redemption details.

## Description
Contains information about a coupon redemption including the customer email, order number, and redemption date.

```ts
declare class CouponRedemption  {
	/**
	 * Returns email of redeeming customer.
	 */
	readonly customerEmail: string

	/**
	 * Returns number of the order the code was redeemed with.
	 */
	readonly orderNo: string

	/**
	 * Returns date of redemption.
	 */
	readonly redemptionDate: Date

	/**
	 * Returns email of redeeming customer.
	 * @returns email of redeeming customer
	 */
	getCustomerEmail(): string

	/**
	 * Returns number of the order the code was redeemed with.
	 * @returns number of the order the code was redeemed with
	 */
	getOrderNo(): string

	/**
	 * Returns date of redemption.
	 * @returns date of redemption
	 */
	getRedemptionDate(): Date
}
```