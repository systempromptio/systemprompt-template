# dw.order.TaxMgr

## Overview
Provides methods to access the tax table.

## Description
Provides methods to access the tax table.

```ts
declare class TaxMgr  {
    /**
     * Constant representing the gross taxation policy.
     */
    static TAX_POLICY_GROSS: 0

    /**
     * Constant representing the net taxation policy.
     */
    static TAX_POLICY_NET: 1

    /**
     * The ID of the tax class that represents items with a custom tax rate.
     * @readonly
     */
    customRateTaxClassID: string

    /**
     * The ID of the default tax class defined for the site. Returns null if none.
     * @readonly
     */
    defaultTaxClassID: string | null

    /**
     * The ID of the default tax jurisdiction defined for the site. Returns null if none.
     * @readonly
     */
    defaultTaxJurisdictionID: string | null

    /**
     * The taxation policy (net/gross) configured for the current site.
     * @readonly
     */
    taxationPolicy: number

    /**
     * The ID of the tax class that represents tax exempt items.
     * @readonly
     */
    taxExemptTaxClassID: string

    /**
     * Applies externally set tax rates to the given Basket.
     * @param {dw.order.Basket} basket - apply external taxation to this basket
     */
    static applyExternalTax(basket: unknown): void

    /**
     * Returns the ID of the tax class that represents items with a custom tax rate.
     * @returns {string}
     */
    static getCustomRateTaxClassID(): string

    /**
     * Returns the ID of the default tax class defined for the site.
     * @returns {string|null}
     */
    static getDefaultTaxClassID(): string | null

    /**
     * Returns the ID of the default tax jurisdiction defined for the site.
     * @returns {string|null}
     */
    static getDefaultTaxJurisdictionID(): string | null

    /**
     * Returns the taxation policy (net/gross) configured for the current site.
     * @returns {number}
     */
    static getTaxationPolicy(): number

    /**
     * Returns the ID of the tax class that represents tax exempt items.
     * @returns {string}
     */
    static getTaxExemptTaxClassID(): string

    /**
     * Returns the ID of the tax jurisdiction for the specified address.
     * @param {dw.order.ShippingLocation} location - The shipping location
     * @returns {string|null}
     */
    static getTaxJurisdictionID(location: unknown): string | null

    /**
     * Returns the tax rate defined for the specified combination of tax class and tax jurisdiction.
     * @param {string} taxClassID - ID of the tax class
     * @param {string} taxJurisdictionID - ID of tax jurisdiction
     * @returns {number|null} the tax rate or null if not defined
     */
    static getTaxRate(taxClassID: string, taxJurisdictionID: string): number | null
}
```
