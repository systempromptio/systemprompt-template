# dw.campaign.Promotion

## Overview
Represents a promotion defined in a campaign or AB test. Provides accessors for metadata (ID, name, images, dates, tags) and to calculate promotional pricing for products.

## Description
Promotion instances are persistent objects that describe promotional rules and associated discounts. Methods allow querying applicability, retrieving associated coupons, customer groups, source code groups, and calculating promotional prices for specified products.

All Known Subclasses

```ts
declare class Promotion extends dw.object.PersistentObject {
	/**
	 * Returns the callout message for the promotion.
	 */
	getCalloutMsg(): dw.content.MarkupText

	/**
	 * Returns the campaign this promotion belongs to.
	 */
	getCampaign(): dw.campaign.Campaign

	/**
	 * Returns combinable promotions (IDs) this promotion can be combined with.
	 */
	getCombinablePromotions(): String[]

	/**
	 * Returns a conditional description (deprecated).
	 */
	getConditionalDescription(): dw.content.MarkupText

	/**
	 * Returns coupons assigned to the promotion or its campaign.
	 */
	getCoupons(): dw.util.Collection

	/**
	 * Returns custom attributes for this promotion.
	 */
	getCustom(): dw.object.CustomAttributes

	/**
	 * Returns customer groups assigned to promotion or its campaign.
	 */
	getCustomerGroups(): dw.util.Collection

	/**
	 * Returns detailed description of the promotion.
	 */
	getDetails(): dw.content.MarkupText

	/**
	 * Returns the effective end date for this promotion instance.
	 */
	getEndDate(): Date

	/**
	 * Returns exclusivity setting (EXCLUSIVITY_* constants).
	 */
	getExclusivity(): String

	/**
	 * Returns the unique promotion ID.
	 */
	getID(): String

	/**
	 * Returns promotion image reference.
	 */
	getImage(): dw.content.MediaFile

	/**
	 * Returns last modified date.
	 */
	getLastModified(): Date

	/**
	 * Returns promotions that are mutually exclusive with this one (IDs).
	 */
	getMutuallyExclusivePromotions(): String[]

	/**
	 * Returns promotion name.
	 */
	getName(): String

	/**
	 * Returns promotional price for a product when applicable.
	 */
	getPromotionalPrice(product: dw.catalog.Product): dw.value.Money

	/**
	 * Returns promotional price for a product using an option model.
	 */
	getPromotionalPrice(product: dw.catalog.Product, optionModel: dw.catalog.ProductOptionModel): dw.value.Money

	/**
	 * Returns promotion class (PROMOTION_CLASS_* constants).
	 */
	getPromotionClass(): String

	/**
	 * Returns how qualifiers are matched (QUALIFIER_MATCH_MODE_* constants).
	 */
	getQualifierMatchMode(): String

	/**
	 * Returns numeric rank for ordering promotions.
	 */
	getRank(): Number

	/**
	 * Returns source code groups assigned to promotion or its campaign.
	 */
	getSourceCodeGroups(): dw.util.Collection

	/**
	 * Returns effective start date.
	 */
	getStartDate(): Date

	/**
	 * Returns tags for the promotion.
	 */
	getTags(): String[]

	/**
	 * Returns true if promotion is currently active.
	 */
	isActive(): boolean

	/**
	 * Returns true if promotion is based on a single coupon (deprecated).
	 */
	isBasedOnCoupon(): boolean

	/**
	 * Returns true if promotion is based on coupons.
	 */
	isBasedOnCoupons(): boolean

	/**
	 * Returns true if promotion is based on customer groups.
	 */
	isBasedOnCustomerGroups(): boolean

	/**
	 * Returns true if promotion is based on source codes.
	 */
	isBasedOnSourceCodes(): boolean

	/**
	 * Returns true if promotion is enabled.
	 */
	isEnabled(): boolean

	/**
	 * Returns true if promotion is refinable.
	 */
	isRefinable(): boolean
}
```
