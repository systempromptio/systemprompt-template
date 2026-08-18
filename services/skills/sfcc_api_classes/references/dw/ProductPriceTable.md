 # dw.catalog.ProductPriceTable

 ## Overview
 A quantity-to-price map representing tiered product prices. The applicable price for an order quantity is the price associated with the largest quantity that does not exceed the order quantity.

 ## Description
 Exposes quantities stored in the price table and methods to query next quantity, price for a quantity, percentage off relative to minimum order quantity, and which price book defined the price.

 ```ts
 declare class ProductPriceTable  {
    /** Collection of all quantities stored in the price table. */
    readonly quantities: Collection

    /** Returns the quantity following the passed quantity in the price table, or null if last. */
    getNextQuantity(quantity: Quantity): Quantity

    /** Returns the percentage off value for the passed quantity relative to product minimum order quantity. */
    getPercentage(quantity: Quantity): number

    /** Returns the monetary price for the passed order quantity, or null if none defined. */
    getPrice(quantity: Quantity): Money

    /** Returns the PriceBook which defined the monetary price for the passed quantity, or null if none defined. */
    getPriceBook(quantity: Quantity): PriceBook

    /** Returns all quantities stored in the price table. */
    getQuantities(): Collection
 }
 ```
