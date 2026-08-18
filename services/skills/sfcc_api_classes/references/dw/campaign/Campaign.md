# dw.campaign.Campaign

## Overview
Represents a set of experiences (promotions, slot configurations, sorting rules) deployed as a unit for a specified time frame with qualifiers like customer groups, source codes, and coupons.

## Description
A campaign can be scheduled with start/end dates or run open-ended. Qualifiers determine which customers it applies to (customer groups, source codes, coupons). Can be targeted to stores/store groups and applicable to online or in-store channels.

```ts
declare class Campaign extends ExtensibleObject {
	/**
	 * True if campaign is currently active (enabled and scheduled for now).
	 */
	readonly active: boolean

	/**
	 * True if campaign is applicable to store.
	 */
	readonly applicableInStore: boolean

	/**
	 * True if campaign is applicable to online site.
	 */
	readonly applicableOnline: boolean

	/**
	 * Coupons assigned to the campaign.
	 */
	readonly coupons: Collection

	/**
	 * Customer groups assigned to the campaign.
	 */
	readonly customerGroups: Collection

	/**
	 * Internal description of the campaign.
	 */
	readonly description: string

	/**
	 * True if campaign is enabled.
	 */
	readonly enabled: boolean

	/**
	 * End date of the campaign. Null if no end date (runs forever).
	 */
	readonly endDate: Date | null

	/**
	 * Unique campaign ID.
	 */
	readonly ID: string

	/**
	 * Promotions defined in this campaign (unordered).
	 */
	readonly promotions: Collection

	/**
	 * Source code groups assigned to the campaign.
	 */
	readonly sourceCodeGroups: Collection

	/**
	 * Start date of the campaign. Null if no start date (immediately effective).
	 */
	readonly startDate: Date | null

	/**
	 * Store groups assigned to the campaign.
	 */
	readonly storeGroups: Collection

	/**
	 * Stores assigned to the campaign.
	 */
	readonly stores: Collection

	/**
	 * Returns coupons assigned to the campaign.
	 */
	getCoupons(): Collection

	/**
	 * Returns customer groups assigned to the campaign.
	 */
	getCustomerGroups(): Collection

	/**
	 * Returns internal description of the campaign.
	 */
	getDescription(): string

	/**
	 * Returns end date. Null if campaign runs forever.
	 */
	getEndDate(): Date | null

	/**
	 * Returns unique campaign ID.
	 */
	getID(): string

	/**
	 * Returns promotions defined in this campaign (unordered).
	 */
	getPromotions(): Collection

	/**
	 * Returns source code groups assigned to the campaign.
	 */
	getSourceCodeGroups(): Collection

	/**
	 * Returns start date. Null if immediately effective.
	 */
	getStartDate(): Date | null

	/**
	 * Returns store groups assigned to the campaign.
	 */
	getStoreGroups(): Collection

	/**
	 * Returns stores assigned to the campaign.
	 */
	getStores(): Collection

	/**
	 * True if campaign is active (enabled and scheduled for now).
	 */
	isActive(): boolean

	/**
	 * True if campaign is applicable to store.
	 */
	isApplicableInStore(): boolean

	/**
	 * True if campaign is applicable to online site.
	 */
	isApplicableOnline(): boolean

	/**
	 * True if campaign is enabled.
	 */
	isEnabled(): boolean
}
```
