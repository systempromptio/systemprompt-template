# dw.catalog.ProductSearchModel

## Overview
Central interface to product search results and refinement with utility methods to generate search URLs.

## Description
The class is the central interface to a product search result and a product search refinement. It also provides utility methods to generate a search URL.

```ts
declare class ProductSearchModel extends SearchModel {
    /** URL Parameter for the category ID */
    static CATEGORYID_PARAMETER: 'cgid'
    /** URL Parameter for the inventory list IDs */
    static INVENTORY_LIST_IDS_PARAMETER: 'ilids'
    /** The maximum number of inventory list IDs that can be passed to setInventoryListIDs(List) */
    static MAXIMUM_INVENTORY_LIST_IDS: 10
    /** The maximum number of product IDs that can be passed to setProductIDs(List) */
    static MAXIMUM_PRODUCT_IDS: 30
    /** The maximum number of store inventory values for a store inventory filter that can be passed to setStoreInventoryFilter(StoreInventoryFilter) */
    static MAXIMUM_STORE_INVENTORY_FILTER_VALUES: 10
    /** URL Parameter for the maximum price */
    static PRICE_MAX_PARAMETER: 'pmax'
    /** URL Parameter for the minimum price */
    static PRICE_MIN_PARAMETER: 'pmin'
    /** URL Parameter for the product ID */
    static PRODUCTID_PARAMETER: 'pid'
    /** constant indicating that all related products should be returned for the next product search by promotion ID */
    static PROMOTION_PRODUCT_TYPE_ALL: 'all'
    /** constant indicating that only bonus products should be returned for the next product search by promotion ID */
    static PROMOTION_PRODUCT_TYPE_BONUS: 'bonus'
    /** constant indicating that only discounted products should be returned for the next product search by promotion ID */
    static PROMOTION_PRODUCT_TYPE_DISCOUNTED: 'discounted'
    /** constant indicating that only qualifying products should be returned for the next product search by promotion ID */
    static PROMOTION_PRODUCT_TYPE_QUALIFYING: 'qualifying'
    /** URL Parameter for the promotion product type */
    static PROMOTION_PRODUCT_TYPE_PARAMETER: 'pmpt'
    /** URL Parameter for the promotion ID */
    static PROMOTIONID_PARAMETER: 'pmid'
    /** URL Parameter prefix for a refinement name */
    static REFINE_NAME_PARAMETER_PREFIX: 'prefn'
    /** URL Parameter prefix for a refinement value */
    static REFINE_VALUE_PARAMETER_PREFIX: 'prefv'
    /** URL Parameter prefix for a refinement value */
    static SORT_BY_PARAMETER_PREFIX: 'psortb'
    /** URL Parameter prefix for a refinement value */
    static SORT_DIRECTION_PARAMETER_PREFIX: 'psortd'
    /** URL Parameter prefix for a sorting option */
    static SORTING_OPTION_PARAMETER: 'sopt'
    /** URL Parameter prefix for a sorting rule */
    static SORTING_RULE_PARAMETER: 'srule'

    /** The category object for the category id specified in the query. Returns null if category doesn't exist or is offline. */
    readonly category: Category
    /** The category id that was specified in the search query. */
    categoryID: string
    /** Returns true if this is a pure search for a category (category ID specified and no search phrase). */
    readonly categorySearch: boolean
    /** The deepest common category of all products in the search result. Returns root category for empty search result. */
    readonly deepestCommonCategory: Category
    /** The sorting rule used to order the products in the results, or null if no search executed yet. Respects explicit and implicit sorting rules and options. */
    readonly effectiveSortingRule: SortingRule
    /** A list of inventory IDs that were specified in the search query or an empty list if no inventory ID set. */
    readonly inventoryIDs: List
    /** Flag indicating whether unorderable products should be excluded when getProducts() is called. */
    orderableProductsOnly: boolean
    /** All page meta tags defined for this instance for which content can be generated. */
    readonly pageMetaTags: Array
    /** Indicates if the search result is ordered by a personalized sorting rule. */
    readonly personalizedSort: boolean
    /** The maximum price by which the search result is refined. */
    priceMax: Number
    /** The minimum price by which the search result is refined. */
    priceMin: Number
    /** @deprecated Use getProductIDs() instead. The product id that was specified in the search query. */
    productID: string
    /** A list of product IDs that were specified in the search query or an empty list if no product ID set. */
    productIDs: List
    /** @deprecated Use getProductSearchHits() instead. All products in the search result (excludes offline/removed products). */
    readonly products: Iterator
    /** The product search hits in the search result (includes offline/removed products). */
    readonly productSearchHits: Iterator
    /** The promotion id that was specified in the search query or null if no promotion id set. Returns only first id if multiple specified. */
    promotionID: string
    /** A list of promotion id's that were specified in the search query or an empty list if no promotion id set. */
    promotionIDs: List
    /** The promotion product type specified in the search query. */
    promotionProductType: string
    /** Flag that determines if the category search will be recursive. */
    recursiveCategorySearch: boolean
    /** Returns true if the search is refined by a category (category ID is specified). */
    readonly refinedByCategory: boolean
    /** Identifies if this search has been refined by price. */
    readonly refinedByPrice: boolean
    /** Identifies if this search has been refined by promotion. */
    readonly refinedByPromotion: boolean
    /** Identifies if this is a category search refined with further criteria (brand, attribute, etc.). */
    readonly refinedCategorySearch: boolean
    /** The category used to determine possible refinements for the search. */
    refinementCategory: Category
    /** The ProductSearchRefinements associated with this search and filtered by session currency. */
    readonly refinements: ProductSearchRefinements
    /** The URL of the endpoint where merchants should upload their image for visual search. */
    readonly searchableImageUploadURL: string
    /** Search phrase suggestions for the current search phrase (alternative phrases, corrected and completed terms). */
    readonly searchPhraseSuggestions: SearchPhraseSuggestions
    /** The sorting rule explicitly set on this model to order products, or null if not set. */
    sortingRule: SortingRule
    /** The StoreInventoryFilter which was specified for this search. */
    storeInventoryFilter: StoreInventoryFilter
    /** @deprecated Use getSearchPhraseSuggestions() instead. The suggested search phrase with the highest accuracy. */
    readonly suggestedSearchPhrase: string
    /** @deprecated Use getSearchPhraseSuggestions() instead. A list with up to 5 suggested search phrases. */
    readonly suggestedSearchPhrases: List
    /** Indicates if no-hits search should be tracked for predictive intelligence use. */
    readonly trackingEmptySearchesEnabled: boolean
    /** Returns true if this is a visual search (image UUID is specified). */
    readonly visualSearch: boolean

    /** Constructs a new ProductSearchModel. */
    constructor()

    /**
     * Set the only search hit types to be included from the search. Values accepted are the 'hit type' constants
     * exposed in the ProductSearchHit class. Overwrites any hit type refinements set from prior calls.
     */
    addHitTypeRefinement(...types: string): void

    /**
     * Set the search hit types to be excluded from the search. Values accepted are the 'hit type' constants exposed in
     * the ProductSearchHit class. Overwrites any hit type refinements set from prior calls.
     */
    excludeHitType(...types: string): void

    /** Returns the category object for the category id specified in the query. */
    getCategory(): Category

    /** Returns the category id that was specified in the search query. */
    getCategoryID(): string

    /** Returns the deepest common category of all products in the search result or root for empty search result. */
    getDeepestCommonCategory(): Category

    /** Returns the sorting rule used to order the products in the results of this query, or null if no search executed yet. */
    getEffectiveSortingRule(): SortingRule

    /** Returns a list of inventory IDs that were specified in the search query or an empty list if no inventory ID set. */
    getInventoryIDs(): List

    /** Get the flag indicating whether unorderable products should be excluded when the next call to getProducts() is made. */
    getOrderableProductsOnly(): boolean

    /**
     * Returns the page meta tag for the specified id. The meta tag content is generated based on the product listing page meta tag context and rule.
     * Returns null if the meta tag is undefined, no rule found, or rule resolves to empty string.
     */
    getPageMetaTag(id: string): PageMetaTag

    /** Returns all page meta tags defined for this instance for which content can be generated. */
    getPageMetaTags(): Array

    /** Returns the maximum price by which the search result is refined. */
    getPriceMax(): Number

    /** Returns the minimum price by which the search result is refined. */
    getPriceMin(): Number

    /** @deprecated Use getProductIDs() instead. Returns the product id that was specified in the search query. */
    getProductID(): string

    /** Returns a list of product IDs that were specified in the search query or an empty list if no product ID set. */
    getProductIDs(): List

    /** @deprecated Use getProductSearchHits() instead. Returns all products in the search result (excludes offline/removed products). */
    getProducts(): Iterator

    /** Returns the underlying ProductSearchHit for a product, or null if no ProductSearchHit found for this product. */
    getProductSearchHit(product: Product): ProductSearchHit

    /** Returns the product search hits in the search result (includes offline/removed products). */
    getProductSearchHits(): Iterator

    /** Returns the promotion id that was specified in the search query or null if no promotion id set. Returns only first id if multiple specified. */
    getPromotionID(): string

    /** Returns a list of promotion id's that were specified in the search query or an empty list if no promotion id set. */
    getPromotionIDs(): List

    /** Returns the promotion product type specified in the search query. */
    getPromotionProductType(): string

    /** Returns the category used to determine possible refinements for the search. */
    getRefinementCategory(): Category

    /** Returns the ProductSearchRefinements associated with this search and filtered by session currency. */
    getRefinements(): ProductSearchRefinements

    /** Returns the URL of the endpoint where the merchants should upload their image for visual search. */
    getSearchableImageUploadURL(): string

    /** Returns search phrase suggestions for the current search phrase. */
    getSearchPhraseSuggestions(): SearchPhraseSuggestions

    /** Returns the sorting rule explicitly set on this model to order the products, or null if no rule explicitly set. */
    getSortingRule(): SortingRule

    /** Returns the StoreInventoryFilter which was specified for this search. */
    getStoreInventoryFilter(): StoreInventoryFilter

    /** @deprecated Use getSearchPhraseSuggestions() instead. Returns the suggested search phrase with the highest accuracy. */
    getSuggestedSearchPhrase(): string

    /** @deprecated Use getSearchPhraseSuggestions() instead. Returns a list with up to 5 suggested search phrases. */
    getSuggestedSearchPhrases(): List

    /** Returns true if this is a pure search for a category (category ID specified and no search phrase). */
    isCategorySearch(): boolean

    /** Indicates if the search result is ordered by a personalized sorting rule. */
    isPersonalizedSort(): boolean

    /** Get the flag that determines if the category search will be recursive. */
    isRecursiveCategorySearch(): boolean

    /** Returns true if the search is refined by a category (category ID is specified). */
    isRefinedByCategory(): boolean

    /** Identifies if this search has been refined by price. */
    isRefinedByPrice(): boolean

    /** Identifies if this search has been refined by the given price range. Either range parameter may be null to represent open ranges. */
    isRefinedByPriceRange(priceMin: Number, priceMax: Number): boolean

    /** Identifies if this search has been refined by promotion. */
    isRefinedByPromotion(): boolean

    /** Identifies if this search has been refined by a given promotion. */
    isRefinedByPromotion(promotionID: string): boolean

    /** Identifies if this is a category search and is refined with further criteria (brand, attribute, etc.). */
    isRefinedCategorySearch(): boolean

    /** Indicates if no-hits search should be tracked for predictive intelligence use. */
    isTrackingEmptySearchesEnabled(): boolean

    /** Returns true if this is a visual search (image UUID is specified). */
    isVisualSearch(): boolean

    /**
     * Execute the search based on the configured search term, category and filter conditions and return the execution status.
     * Empty ProductSearchModel without any search term or filter criteria is not supported.
     */
    search(): SearchStatus

    /** Specifies the category id used for the search query. */
    setCategoryID(categoryID: string): void

    /** Set a flag indicating whether no-hits search should be tracked for predictive intelligence use. */
    setEnableTrackingEmptySearches(trackingEmptySearches: boolean): void

    /** Specifies multiple inventory list IDs used for the search query. Supports up to MAXIMUM_INVENTORY_LIST_IDS inventory IDs. */
    setInventoryListIDs(inventoryListIDs: List): void

    /** Set a flag indicating whether unorderable products should be excluded when the next call to getProducts() is made. */
    setOrderableProductsOnly(orderableOnly: boolean): void

    /** Sets the maximum price by which the search result is to be refined. */
    setPriceMax(priceMax: Number): void

    /** Sets the minimum price by which the search result is to be refined. */
    setPriceMin(priceMin: Number): void

    /** @deprecated Use setProductIDs(List) instead. Specifies the product id used for the search query. */
    setProductID(productID: string): void

    /** Specifies multiple product IDs used for the search query. Supports up to MAXIMUM_PRODUCT_IDS product IDs. */
    setProductIDs(productIDs: List): void

    /** Specifies the promotion id used for the search query. */
    setPromotionID(promotionID: string): void

    /** Specifies multiple promotion id's used for the search query. Supports up to 30 promotion id's. */
    setPromotionIDs(promotionIDs: List): void

    /** Specifies the promotion product type used for the search query. This value is only relevant for searches by promotion ID. */
    setPromotionProductType(promotionProductType: string): void

    /** Set a flag to indicate if the search in category should be recursive. */
    setRecursiveCategorySearch(recurse: boolean): void

    /** Sets an explicit category to be used when determining refinements. The category must be in the site's storefront catalog. */
    setRefinementCategory(refinementCategory: Category): void

    /** Sets product IDs retrieved from the image ID to the ProductSearchModel. If the image ID is invalid or expired, product IDs will not be set. */
    setSearchableImageID(imageID: string): void

    /** @deprecated Use setSortingRule(SortingRule) instead. Sets or removes a sorting condition for the specified attribute. */
    setSortingCondition(attributeID: string, direction: Number): void

    /** Sets the sorting option to be used to order the products in the results of this query. If a sorting rule is also set, the sorting option is ignored. */
    setSortingOption(option: SortingOption): void

    /** Sets the sorting rule to be used to order the products in the results of this query. Overrides default behavior. */
    setSortingRule(rule: SortingRule): void

    /** Filters the search result by one or more inventory list IDs provided by the StoreInventoryFilter class. */
    setStoreInventoryFilter(storeInventoryFilter: StoreInventoryFilter): void

    /** Constructs a URL that you can use to execute a query for a specific Category. Generated URL is absolute using current request protocol. */
    static urlForCategory(action: string, cgid: string): URL

    /** Constructs a URL that you can use to execute a query for a specific Category. Search parameters are appended to the provided URL. */
    static urlForCategory(url: URL, cgid: string): URL

    /** Constructs a URL that you can use to execute a query for a specific Product. Generated URL is absolute using current request protocol. */
    static urlForProduct(action: string, cgid: string, pid: string): URL

    /** Constructs a URL that you can use to execute a query for a specific Product. Search parameters are appended to the provided URL. */
    static urlForProduct(url: URL, cgid: string, pid: string): URL

    /** Constructs a URL that you can use to execute a query for a specific attribute name-value pair. Generated URL is absolute using current request protocol. */
    static urlForRefine(action: string, attributeID: string, value: string): URL

    /** Constructs a URL that you can use to execute a query for a specific attribute name-value pair. Search parameters are appended to the provided URL. */
    static urlForRefine(url: URL, attributeID: string, value: string): URL

    /** Constructs a URL to re-execute the query with a category refinement. Generated URL is absolute using current request protocol. */
    urlRefineCategory(action: string, refineCategoryID: string): URL

    /** Constructs a URL to re-execute the query with a category refinement. Search parameters are appended to the provided URL. */
    urlRefineCategory(url: URL, refineCategoryID: string): URL

    /** Constructs a URL to re-execute the query with an additional price filter. Generated URL is absolute using current request protocol. */
    urlRefinePrice(action: string, min: Number, max: Number): URL

    /** Constructs a URL to re-execute the query with an additional price filter. Search parameters are appended to the provided URL. */
    urlRefinePrice(url: URL, min: Number, max: Number): URL

    /** Constructs a URL to re-execute the query with a promotion refinement. Search parameters are appended to the provided URL. */
    urlRefinePromotion(url: URL, refinePromotionID: string): URL

    /** Constructs a URL to re-execute the query with a promotion refinement. Generated URL is absolute using current request protocol. */
    urlRefinePromotion(action: string, refinePromotionID: string): URL

    /** Constructs a URL to re-execute the query without any category refinement. Generated URL is absolute using current request protocol. */
    urlRelaxCategory(action: string): URL

    /** Constructs a URL to re-execute the query without any category refinement. Search parameters are appended to the provided URL. */
    urlRelaxCategory(url: URL): URL

    /** Constructs a URL to re-execute the query with no price filter. Generated URL is absolute using current request protocol. */
    urlRelaxPrice(action: string): URL

    /** Constructs a URL to re-execute the query with no price filter. Search parameters are appended to the provided URL. */
    urlRelaxPrice(url: URL): URL

    /** Constructs a URL to re-execute the query without any promotion refinement. Search parameters are appended to the provided URL. */
    urlRelaxPromotion(url: URL): URL

    /** Constructs a URL to re-execute the query without any promotion refinement. Generated URL is absolute using current request protocol. */
    urlRelaxPromotion(action: string): URL

    /** Constructs a URL to re-execute the query but sort the results by the given storefront sorting option. Generated URL is absolute using current request protocol. */
    urlSortingOption(action: string, option: SortingOption): URL

    /** Constructs a URL to re-execute the query but sort the results by the given storefront sorting option. Search parameters are appended to the provided URL. */
    urlSortingOption(url: URL, option: SortingOption): URL

    /** Constructs a URL to re-execute the query but sort the results by the given rule. Generated URL is absolute using current request protocol. */
    urlSortingRule(action: string, rule: SortingRule): URL

    /** Constructs a URL to re-execute the query but sort the results by the given rule. Search parameters are appended to the provided URL. */
    urlSortingRule(url: URL, rule: SortingRule): URL
}
```
