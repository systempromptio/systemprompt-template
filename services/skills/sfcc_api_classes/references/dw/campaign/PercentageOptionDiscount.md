# dw.campaign.PercentageOptionDiscount

## Overview
Represents a percentage-off options discount in a discount plan (for example, "50% off monogramming on shirts").

## Description
Read-only description object for a percentage option discount. Use accessors to read the discount percentage; instances are provided by the campaign/promotion APIs.

```ts
declare class PercentageOptionDiscount extends dw.campaign.Discount {
	/**
	 * The percentage discount value (for example 10.00 for 10%).
	 */
	percentage: Number

	/**
	 * Returns the percentage discount value.
	 * @returns Discount percentage value (e.g. 10.00)
	 */
	getPercentage(): Number
}
```
