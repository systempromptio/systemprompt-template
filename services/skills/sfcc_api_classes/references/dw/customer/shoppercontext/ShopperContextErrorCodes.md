# dw.customer.shoppercontext.ShopperContextErrorCodes

## Overview
Defines error-code constants used to indicate why shopper context operations failed (validation, limits, feature toggles, internal errors, request type).

## Description
Helper class that exposes string constants representing error conditions when accessing, setting, or modifying Shopper Context (for example: limits exceeded, feature disabled, invalid arguments).

## All Known Subclasses


```ts
declare class ShopperContextErrorCodes  {
	/** Assignment qualifiers limit exceeded. */
	static ASSIGNMENT_QUALIFIERS_LIMIT_EXCEEDED: 'ASSIGNMENT_QUALIFIERS_LIMIT_EXCEEDED'

	/** Coupon codes limit exceeded. */
	static COUPON_CODES_LIMIT_EXCEEDED: 'COUPON_CODES_LIMIT_EXCEEDED'

	/** Custom qualifiers limit exceeded. */
	static CUSTOM_QUALIFIERS_LIMIT_EXCEEDED: 'CUSTOM_QUALIFIERS_LIMIT_EXCEEDED'

	/** Feature toggle disabled. */
	static FEATURE_DISABLED: 'FEATURE_DISABLED'

	/** Internal error occurred. */
	static INTERNAL_ERROR: 'INTERNAL_ERROR'

	/** Invalid argument provided. */
	static INVALID_ARGUMENT: 'INVALID_ARGUMENT'

	/** Invalid request type. */
	static INVALID_REQUEST_TYPE: 'INVALID_REQUEST_TYPE'

	/** Quota limit exceeded. */
	static QUOTA_LIMIT_EXCEEDED: 'QUOTA_LIMIT_EXCEEDED'
}
```
