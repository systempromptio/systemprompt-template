# dw.catalog.ProductSearchHit

## Overview
Represents a single search hit from the product search index. Encapsulates the presentation product, the set of actual products represented by the hit, price range info, and helper accessors.

## Description
A ProductSearchHit is a read-only view returned by product search. It provides the presentation product (product shown on a tile), the products that actually matched the query (represented products), pricing summary (min/max), promotions that qualify, and convenience checks such as whether the hit represents a price range. Use the `getRepresentedProducts()` family to access the actual matched products and `getProduct()`/`getProductID()` for the presentation product.

```ts
declare class ProductSearchHit  {
    /** HIT: simple product */
    static HIT_TYPE_SIMPLE: 'HIT_TYPE_SIMPLE'
    /** HIT: product master */
    static HIT_TYPE_PRODUCT_MASTER: 'HIT_TYPE_PRODUCT_MASTER'
    /** HIT: product bundle */
    static HIT_TYPE_PRODUCT_BUNDLE: 'HIT_TYPE_PRODUCT_BUNDLE'
    /** HIT: product set */
    static HIT_TYPE_PRODUCT_SET: 'HIT_TYPE_PRODUCT_SET'
    /** HIT: slicing group */
    static HIT_TYPE_SLICING_GROUP: 'HIT_TYPE_SLICING_GROUP'
    /** HIT: variation group */
    static HIT_TYPE_VARIATION_GROUP: 'HIT_TYPE_VARIATION_GROUP'

    /** The ID of the product that is first in sort order among represented products. */
    readonly firstRepresentedProductID: string
    /** The type of the product wrapped by this hit (one of the HIT_TYPE_* constants). */
    readonly hitType: string
    /** The product that is last in sort order among represented products. */
    readonly lastRepresentedProduct: Product
    /** The ID of the product that is last in sort order among represented products. */
    readonly lastRepresentedProductID: string
    /** Maximum price (Money) among represented products from the index. May be N/A. */
    readonly maxPrice: Money
    /** Maximum price per unit among represented products from the index. May be N/A. */
    readonly maxPricePerUnit: Money
    /** Minimum price (Money) among represented products from the index. May be N/A. */
    readonly minPrice: Money
    /** Minimum price per unit among represented products from the index. May be N/A. */
    readonly minPricePerUnit: Money
    /** True when represented products have differing prices. */
    readonly priceRange: boolean
    /** Presentation product for this hit (product shown on the tile). */
    readonly product: Product
    /** ID of the presentation product for this hit. */
    readonly productID: string
    /** IDs of promotions that qualify for at least one represented product (index-time data). */
    readonly qualifyingPromotionIDs: List<string>
    /** IDs of the actual products represented by this hit, ordered by sort rank. */
    readonly representedProductIDs: List<string>
    /** The actual products represented by this hit, ordered by sort rank. */
    readonly representedProducts: List<Product>
    /** Distinct variation values for represented variants for a given variation attribute. */
    readonly representedVariationValues: List<any>

    /** Returns the product with the highest sort rank among represented products. */
    getFirstRepresentedProduct(): Product
    /** Returns the ID of the product with the highest sort rank among represented products. */
    getFirstRepresentedProductID(): string
    /** Returns the hit type string (one of the HIT_TYPE_* constants). */
    getHitType(): string
    /** Returns the product with the lowest sort rank among represented products. */
    getLastRepresentedProduct(): Product
    /** Returns the ID of the product with the lowest sort rank among represented products. */
    getLastRepresentedProductID(): string
    /** Returns the maximum price among represented products (index price). */
    getMaxPrice(): Money
    /** Returns the maximum price per unit among represented products (index price). */
    getMaxPricePerUnit(): Money
    /** Returns the minimum price among represented products (index price). */
    getMinPrice(): Money
    /** Returns the minimum price per unit among represented products (index price). */
    getMinPricePerUnit(): Money
    /** Returns the presentation product for this hit. */
    getProduct(): Product
    /** Returns the presentation product ID for this hit. */
    getProductID(): string
    /** Returns the IDs of the actual products represented by this hit (ordered by rank). */
    getRepresentedProductIDs(): List<string>
    /** Returns the actual products represented by this hit (ordered by rank). */
    getRepresentedProducts(): List<Product>
    /**
     * Returns distinct variation attribute values for represented variants.
     * @param va ProductVariationAttribute or String (attribute ID)
     */
    getRepresentedVariationValues(va: Object): List<any>
    /** Returns true when the represented products form a price range. */
    isPriceRange(): boolean
}
```
