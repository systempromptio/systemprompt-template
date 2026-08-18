# dw.customer.shoppercontext.ShopperContext

## Overview
Represents shopper-level contextual data used to personalize storefront experiences (custom qualifiers, assignment qualifiers, geolocation, client IP, effective datetime, source code, coupon codes, and customer groups).

## Description
Manages context values applied to a shopper. When set, the context takes effect on the next request and can influence promotions, price books, and other behavior tied to customer groups, source codes, or store assignments.

## All Known Subclasses


```ts
declare class ShopperContext  {
	/** The assignment qualifiers from the Shopper Context. */
	assignmentQualifiers: Map

	/** The IP address of the client from the Shopper Context. */
	clientIP: string

	/** The coupon codes from the Shopper Context. */
	couponCodes: Set

	/** Customer group IDs from the Shopper Context. */
	customerGroupIDs: Set

	/** Custom session attributes as qualifiers. */
	customQualifiers: Map

	/** The effective date/time for which this context applies. */
	effectiveDateTime: Date

	/** The geographic location from the Shopper Context. */
	geolocation: Geolocation

	/** The source code for the Shopper Context. */
	sourceCode: string

	/**
	 * Constructor for ShopperContext.
	 */
	constructor(): ShopperContext

	/** Returns assignment qualifiers map. */
	getAssignmentQualifiers(): Map

	/** Returns client IP string. */
	getClientIP(): string

	/** Returns coupon codes set. */
	getCouponCodes(): Set

	/** Returns customer group IDs set. */
	getCustomerGroupIDs(): Set

	/** Returns custom qualifiers map. */
	getCustomQualifiers(): Map

	/** Returns effective Date. */
	getEffectiveDateTime(): Date

	/** Returns Geolocation object. */
	getGeolocation(): Geolocation

	/** Returns source code string. */
	getSourceCode(): string

	/** Sets assignment qualifiers. */
	setAssignmentQualifiers(assignmentQualifiers: Map): void

	/** Sets client IP. */
	setClientIP(clientIP: string): void

	/** Sets coupon codes. */
	setCouponCodes(couponCodes: Set): void

	/** Sets customer group IDs. */
	setCustomerGroupIDs(customerGroupIDs: Set): void

	/** Sets custom qualifiers. */
	setCustomQualifiers(customQualifiers: Map): void

	/** Sets effective date/time. */
	setEffectiveDateTime(effectiveDateTime: Date): void

	/** Sets Geolocation. */
	setGeolocation(geolocation: Geolocation): void

	/** Sets source code. */
	setSourceCode(sourceCode: string): void
}
```
