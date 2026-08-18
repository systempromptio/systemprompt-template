# dw.campaign.Coupon

## Overview
Represents a coupon in Commerce Cloud Digital, supporting single-code, multiple-codes, and system-generated code types with configurable redemption limits.

## Description
Provides access to coupon properties including type, codes, redemption limits, and associated promotions. Supports three coupon types: single fixed code, multiple predefined codes, or system-generated codes with optional prefix.

```ts
declare class Coupon extends PersistentObject {
	/**
	 * Coupon type: single-code.
	 */
	static TYPE_SINGLE_CODE: 'SINGLE_CODE'

	/**
	 * Coupon type: multiple-codes.
	 */
	static TYPE_MULTIPLE_CODES: 'MULTIPLE_CODES'

	/**
	 * Coupon type: system-codes.
	 */
	static TYPE_SYSTEM_CODES: 'SYSTEM_CODES'

	/**
	 * Prefix for system-codes coupons. Null for single-code or multiple-codes types.
	 */
	readonly codePrefix: string | null

	/**
	 * True if coupon is enabled.
	 */
	readonly enabled: boolean

	/**
	 * Coupon ID.
	 */
	readonly ID: string

	/**
	 * Next unissued code. For single-code, returns fixed code. For multi-code, returns next available. Null if all codes issued. Requires transaction.
	 */
	readonly nextCouponCode: string | null

	/**
	 * Coupon-based promotions assigned directly or through campaigns.
	 */
	readonly promotions: Collection

	/**
	 * Redemption limit per coupon code. Null if unlimited.
	 */
	readonly redemptionLimitPerCode: number | null

	/**
	 * Redemption limit per customer. Null if unlimited.
	 */
	readonly redemptionLimitPerCustomer: number | null

	/**
	 * Redemption limit per customer per time-frame. Null if no time-specific limit.
	 */
	readonly redemptionLimitPerTimeFrame: number | null

	/**
	 * Time-frame (days) for redemption limit per customer. Null if no time-specific limit.
	 */
	readonly redemptionLimitTimeFrame: number | null

	/**
	 * Coupon type: TYPE_SINGLE_CODE, TYPE_MULTIPLE_CODES, or TYPE_SYSTEM_CODES.
	 */
	readonly type: string

	/**
	 * Returns prefix for system-codes coupons. Null for other types or if no prefix defined.
	 */
	getCodePrefix(): string | null

	/**
	 * Returns coupon ID.
	 */
	getID(): string

	/**
	 * Returns next unissued code. For single-code, returns fixed code. For multi-code, returns next available. Null if all codes issued. Requires transaction.
	 */
	getNextCouponCode(): string | null

	/**
	 * Returns promotions assigned to coupon (directly or through campaigns), unordered.
	 */
	getPromotions(): Collection

	/**
	 * Returns redemption limit per code. Null if unlimited.
	 */
	getRedemptionLimitPerCode(): number | null

	/**
	 * Returns redemption limit per customer. Null if unlimited.
	 */
	getRedemptionLimitPerCustomer(): number | null

	/**
	 * Returns redemption limit per customer per time-frame. Null if no time-specific limit.
	 */
	getRedemptionLimitPerTimeFrame(): number | null

	/**
	 * Returns time-frame (days) for redemption limit. Null if no time-specific limit.
	 */
	getRedemptionLimitTimeFrame(): number | null

	/**
	 * Returns coupon type.
	 */
	getType(): string

	/**
	 * True if coupon is enabled.
	 */
	isEnabled(): boolean
}
```
