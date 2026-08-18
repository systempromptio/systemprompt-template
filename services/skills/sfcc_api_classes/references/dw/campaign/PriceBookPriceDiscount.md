# dw.campaign.PriceBookPriceDiscount

## Overview
Discount indicating a product's price comes from a nonstandard price book (a separate sales price book other than the site's standard price book).

## Description
Read-only object describing a price-book-based discount. It exposes the price book identifier.

```ts
declare class PriceBookPriceDiscount extends dw.campaign.Discount {
	/**
	 * Identifier of the price book used to calculate the product price.
	 */
	priceBookID: String

	/**
	 * Returns the price book identifier.
	 * @returns Price book ID
	 */
	getPriceBookID(): String
}
```
