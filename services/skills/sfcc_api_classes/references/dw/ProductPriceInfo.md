 # dw.catalog.ProductPriceInfo

 ## Overview
 Represents a single product price point with context such as price book, date range, percentage off, and arbitrary price metadata.

 ## Description
 Provides read-only access to the monetary price and metadata that explain where the price came from and when it is valid.

 ```ts
 declare class ProductPriceInfo  {
    /** Date from which the associated price point is valid (or null). */
    readonly onlineFrom: Date

    /** Date until which the price point is valid (or null). */
    readonly onlineTo: Date

    /** Percentage off relative to base price for minimum order quantity. */
    readonly percentage: number

    /** Monetary price for this price point. */
    readonly price: Money

    /** The PriceBook that defined this price point. */
    readonly priceBook: PriceBook

    /** Arbitrary merchant-defined string associated with the price entry. */
    readonly priceInfo: string

    /** Returns the date from which this price point is valid. */
    getOnlineFrom(): Date

    /** Returns the date until which this price point is valid. */
    getOnlineTo(): Date

    /** Returns the percentage off value for this price point. */
    getPercentage(): number

    /** Returns the monetary price for this price point. */
    getPrice(): Money

    /** Returns the PriceBook which defined this price point. */
    getPriceBook(): PriceBook

    /** Returns the priceInfo string associated with this price point. */
    getPriceInfo(): string
 }
 ```
