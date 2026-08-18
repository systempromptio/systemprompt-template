# dw.catalog.PriceBookMgr

## Overview
Provides accessors and session helpers for price books configured in the organization and assigned to sites.

## Description
Manager class exposing collections of price books and operations to assign/unassign price books to sites and to control which price books are considered during product price lookups.

```ts
declare class PriceBookMgr  {
    /** All price books defined for the organization (read-only). */
    static readonly allPriceBooks: dw.util.Collection

    /** A collection of price books currently set in the user session (used for price lookup). */
    static applicablePriceBooks: dw.util.Collection

    /** All price books assigned to the current site (read-only). Does not include parent price books considered by price lookup). */
    static readonly sitePriceBooks: dw.util.Collection

    /**
     * Assigns a price book to a site. Requires a transaction (use Transaction.wrap).
     * @param priceBook PriceBook to assign
     * @param siteId ID of the storefront site (e.g., 'SiteGenesis')
     * @returns true if assigned; throws if price book or site is invalid
     */
    static assignPriceBookToSite(priceBook: dw.catalog.PriceBook, siteId: string): boolean

    /** Returns all price books defined for the organization. */
    static getAllPriceBooks(): dw.util.Collection

    /** Returns the collection of price books set in the current user session. */
    static getApplicablePriceBooks(): dw.util.Collection

    /**
     * Returns the price book matching the given ID in the current organization.
     * @param priceBookID The price book id
     * @returns PriceBook or null if not found
     */
    static getPriceBook(priceBookID: string): dw.catalog.PriceBook | null

    /** Returns all price books assigned to the current site. */
    static getSitePriceBooks(): dw.util.Collection

    /**
     * Sets one or more price books to be considered by product price lookup (stored in the user session).
     * @param priceBooks One or more PriceBook objects
     */
    static setApplicablePriceBooks(...priceBooks: dw.catalog.PriceBook[]): void

    /**
     * Unassigns a price book from all sites. Requires a transaction (use Transaction.wrap).
     * @param priceBook PriceBook to unassign
     * @returns true if unassigned; throws if price book doesn't exist
     */
    static unassignPriceBookFromAllSites(priceBook: dw.catalog.PriceBook): boolean

    /**
     * Unassigns a price book from a specific site. Requires a transaction (use Transaction.wrap).
     * @param priceBook PriceBook to unassign
     * @param siteId ID of the storefront site
     * @returns true if unassigned; throws on invalid inputs
     */
    static unassignPriceBookFromSite(priceBook: dw.catalog.PriceBook, siteId: string): boolean
}
```

## All Known Subclasses
None
