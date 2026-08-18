# dw.campaign.AmountDiscount

## Overview
Represents an amount-off discount in the discount plan, e.g., "$10 off all orders $100 or more".

## Description
Used to define a fixed amount discount for qualifying orders or items.

```ts
declare class AmountDiscount extends Discount {
    /**
     * The discount amount, e.g., 10.00 for a "$10 off" discount.
     * @readonly
     */
    readonly amount: Number;

    /**
     * Create an amount-discount on the fly.
     * @param amount Amount off, e.g., 15.00 for a "$15 off" discount.
     */
    constructor(amount: Number);

    /**
     * Returns the discount amount.
     * @returns Discount amount.
     */
    getAmount(): Number;
}
```
