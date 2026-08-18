 # dw.catalog.ProductPriceModel

 ## Overview
 Provides price lookup and aggregated price information for a product, including per-unit pricing, price ranges for master/set products, price book lookups, and access to a product price table.

 ## Description
 Calculates effective storefront prices using configured price books and optionally an associated option model. Supports quantity-based lookups, price book-specific queries, and returns `ProductPriceInfo` objects with contextual metadata.

 ```ts
 declare class ProductPriceModel  {
    /** Returns active price for base quantity 1.00 (session currency) or MONEY.NOT_AVAILABLE. */
    getPrice(): Money

    /** Returns active price for specified quantity. */
    getPrice(quantity: Quantity): Money

    /** Returns active price of the product in the specified price book for quantity 1.00. */
    getPriceBookPrice(priceBookID: string): Money

    /** Returns active price in price book for specified quantity. */
    getPriceBookPrice(priceBookID: string, quantity: Quantity): Money

    /** Returns ProductPriceInfo for the specified price book (or null). */
    getPriceBookPriceInfo(priceBookID: string): ProductPriceInfo

    /** Returns ProductPriceInfo for the specified price book and quantity (or null). */
    getPriceBookPriceInfo(priceBookID: string, quantity: Quantity): ProductPriceInfo

    /** Returns active price info for base quantity 1.00 (or null). */
    getPriceInfo(): ProductPriceInfo

    /** Returns active price info for specified quantity (or null). */
    getPriceInfo(quantity: Quantity): ProductPriceInfo

    /** Returns collection of eligible ProductPriceInfo objects (may be empty). */
    getPriceInfos(): Collection

    /** Returns the product price table used for quantity-based pricing. */
    getPriceTable(): ProductPriceTable

    /** Returns the sales price per unit for base quantity 1.00 (or MONEY.N_A). */
    getPricePerUnit(): Money

    /** Returns the sales price per unit for the specified quantity. */
    getPricePerUnit(quantity: Quantity): Money

    /** Calculates percentage between two Money values (deprecated). */
    getPricePercentage(basePrice: Money, comparePrice: Money): number

    /** Returns the maximum/minimum price across variants or set-products. */
    getMaxPrice(): Money
    getMinPrice(): Money

    /** Returns max/min price per unit across variants or set-products. */
    getMaxPricePerUnit(): Money
    getMinPricePerUnit(): Money

    /** Returns max/min price (or per-unit) within a specific price book. */
    getMaxPriceBookPrice(priceBookID: string): Money
    getMinPriceBookPrice(priceBookID: string): Money
    getMaxPriceBookPricePerUnit(priceBookID: string): Money
    getMinPriceBookPricePerUnit(priceBookID: string): Money

    /** Returns true when the product has a range of prices (variants/set-products). */
    isPriceRange(): boolean
    isPriceRange(priceBookID: string): boolean
 }
 ```
