# dw.campaign.BonusChoiceDiscount

## Overview
Represents a choice of bonus products discount, e.g., "Choose 3 DVDs from a list of 20 options with your purchase of any DVD player."

## Description
Defines a discount where customers can select from a list of bonus products. The list may be static or rule-based.

```ts
declare class BonusChoiceDiscount extends Discount {
    /**
     * List of bonus products the customer can choose from for this discount.
     * @readonly
     */
    readonly bonusProducts: List;

    /**
     * The maximum number of bonus items a customer can select for this discount.
     * @readonly
     */
    readonly maxBonusItems: Number;

    /**
     * Returns true if this is a rule-based bonus choice discount.
     * @readonly
     */
    readonly ruleBased: boolean;

    /**
     * Get the effective price for the given bonus product.
     * @param product The bonus product to retrieve a price for.
     * @returns The price of the bonus product as a Number.
     */
    getBonusProductPrice(product: Product): Number;

    /**
     * Get the list of bonus products the customer can choose from for this discount.
     * @returns Ordered list of bonus products.
     */
    getBonusProducts(): List;

    /**
     * Returns the maximum number of bonus items a customer can select for this discount.
     * @returns Maximum number of bonus items.
     */
    getMaxBonusItems(): Number;

    /**
     * Returns true if this is a rule-based bonus choice discount.
     * @returns True if rule-based, false otherwise.
     */
    isRuleBased(): boolean;
}
```
