# dw.catalog.Product

## Overview
Core product entity in Commerce Cloud Digital identified by unique SKU. Supports multiple product types: simple products, master products with variations, variants, option products, product sets, and bundles.

## Description
Represents a product in Commerce Cloud Digital. Products are identified by unique product IDs (SKUs). Types include simple products, master products (templates for variation sets), variants (orderable products mastered by a master), option products (additional purchasable options), product-sets (non-orderable collections), and product bundles (orderable collections with unified pricing and inventory).

Product price and availability information are accessible through `getPriceModel()` and `getAvailabilityModel()`. Attribute information is accessible through `getAttributeModel()`. Products may reference other products via recommendations or product links. Products belong to an owning catalog and are assigned to categories in other catalogs.

## All Known Subclasses
Variant, VariationGroup

```ts
declare class Product extends ExtensibleObject {
	/** Active data for this product for the current site. */
	readonly activeData: ProductActiveData

	/** All categories to which this product is assigned. */
	readonly allCategories: Collection

	/** All category assignments for this product in any catalog. */
	readonly allCategoryAssignments: Collection

	/** All incoming ProductLinks. */
	readonly allIncomingProductLinks: Collection

	/** All outgoing ProductLinks. */
	readonly allProductLinks: Collection

	/** True if product is assigned to the current site via site catalog. */
	readonly assignedToSiteCatalog: boolean

	/** Product's ProductAttributeModel for convenient attribute information access. */
	readonly attributeModel: ProductAttributeModel

	/** Availability model for determining product availability information. */
	readonly availabilityModel: ProductAvailabilityModel

	/** Availability status flag. @deprecated Use getAvailabilityModel().isInStock() instead */
	readonly available: boolean

	/** Availability status flag. @deprecated Use getAvailabilityModel() instead */
	availableFlag: boolean

	/** Product brand. */
	readonly brand: string

	/** True if this product instance is a product bundle. */
	readonly bundle: boolean

	/** True if this product is bundled within at least one product bundle. */
	readonly bundled: boolean

	/** All products that participate in the product bundle. */
	readonly bundledProducts: Collection

	/** All bundles in which this product is included (site-assigned only). */
	readonly bundles: Collection

	/** All categories to which this product is assigned and available through current site. */
	readonly categories: Collection

	/** True if product is bound to at least one catalog category. */
	readonly categorized: boolean

	/** Category assignments for this product in the current site catalog. */
	readonly categoryAssignments: Collection

	/** Classification category associated with this product. Defines the attribute set. */
	readonly classificationCategory: Category

	/** European Article Number of the product. */
	readonly EAN: string

	/** True if product is Facebook enabled. */
	readonly facebookEnabled: boolean

	/** ID of the product. */
	readonly ID: string

	/** Product's image. @deprecated Use getImages(String) and getImage(String, Number) instead */
	readonly image: MediaFile

	/** Incoming ProductLinks where source product is a site product. */
	readonly incomingProductLinks: Collection

	/** Product's long description in current locale. */
	readonly longDescription: MarkupText

	/** Name of the product manufacturer. */
	readonly manufacturerName: string

	/** Manufacturer's stock keeping unit value. */
	readonly manufacturerSKU: string

	/** True if this product instance is a product master. */
	readonly master: boolean

	/** Minimum order quantity for this product. */
	readonly minOrderQuantity: Quantity

	/** Product name in current locale. */
	readonly name: string

	/** Online status of the product calculated from online flag and onlineFrom/onlineTo dates. */
	readonly online: boolean

	/** All currently online categories to which product is assigned and available through current site. */
	readonly onlineCategories: Collection

	/** Online status flag of the product. */
	readonly onlineFlag: boolean

	/** Date from which the product is online or valid. */
	readonly onlineFrom: Date

	/** Date until which the product is online or valid. */
	readonly onlineTo: Date

	/** Product's option model with initialized option values. */
	readonly optionModel: ProductOptionModel

	/** True if the product has options. */
	readonly optionProduct: boolean

	/** Outgoing recommendations for this product with orderable target products only. */
	readonly orderableRecommendations: Collection

	/** Product's page description in default locale. */
	readonly pageDescription: string

	/** Product's page keywords in default locale. */
	readonly pageKeywords: string

	/** All page meta tags defined for this instance for which content can be generated. */
	readonly pageMetaTags: Array

	/** Product's page title in default locale. */
	readonly pageTitle: string

	/** Product's page URL in default locale. */
	readonly pageURL: string

	/** True if product is Pinterest enabled. */
	readonly pinterestEnabled: boolean

	/** Price model for retrieving price for this product. */
	readonly priceModel: ProductPriceModel

	/** Primary category of the product within current site catalog. */
	readonly primaryCategory: Category

	/** Category assignment to the primary category in current site catalog or null. */
	readonly primaryCategoryAssignment: CategoryAssignment

	/** True if instance represents a product, false if product set. */
	readonly product: boolean

	/** All outgoing ProductLinks where target product is available in current site (unsorted). */
	readonly productLinks: Collection

	/** True if instance represents a product set, otherwise false. */
	readonly productSet: boolean

	/** True if this product is part of any product set. */
	readonly productSetProduct: boolean

	/** All products assigned to this product and available through current site. */
	readonly productSetProducts: Collection

	/** All product sets in which this product is included (site-assigned only). */
	readonly productSets: Collection

	/** Outgoing recommendations for this product in site catalog, sorted by order. */
	readonly recommendations: Collection

	/** True if this product instance is part of a retail set. @deprecated Use isProductSet() instead */
	readonly retailSet: boolean

	/** True if product is searchable. */
	readonly searchable: boolean

	/** True if product is currently searchable. */
	readonly searchableFlag: boolean

	/** Searchable status of Product if unavailable. Null indicates value not set. */
	readonly searchableIfUnavailableFlag: boolean

	/** Product's search placement classification. Higher value indicates more relevance. */
	readonly searchPlacement: number

	/** Product's search rank. Higher value indicates more relevance. */
	readonly searchRank: number

	/** Product's short description in current locale. */
	readonly shortDescription: MarkupText

	/** Product's change frequency needed for sitemap creation. */
	readonly siteMapChangeFrequency: string

	/** Status if product is included in sitemap. */
	readonly siteMapIncluded: number

	/** Product's priority needed for sitemap creation. */
	readonly siteMapPriority: number

	/** True if product is assigned to current site via site catalog. @deprecated Use isAssignedToSiteCatalog() instead */
	readonly siteProduct: boolean

	/** Steps in which the order amount of product can be increased. */
	readonly stepQuantity: Quantity

	/** Store receipt name of product in current locale. */
	readonly storeReceiptName: string

	/** Store tax class ID (optional override for in-store tax calculation). */
	readonly storeTaxClass: string

	/** Product's tax class ID resolved by Global Preference setting. */
	readonly taxClassID: string

	/** Name of product's rendering template. */
	readonly template: string

	/** Product's thumbnail image. @deprecated Use getImages(String) and getImage(String, Number) instead */
	readonly thumbnail: MediaFile

	/** Product's sales unit. */
	readonly unit: string

	/** Product's unit quantity. */
	readonly unitQuantity: Quantity

	/** Universal Product Code of the product. */
	readonly UPC: string

	/** True if this product instance is mastered by a product master. */
	readonly variant: boolean

	/** All variants assigned to this variation master or variation group product. */
	readonly variants: Collection

	/** True if this product instance is a variation group product. */
	readonly variationGroup: boolean

	/** All variation groups assigned to this variation master product. */
	readonly variationGroups: Collection

	/** Variation model of this product with pre-selected attribute values if variant. */
	readonly variationModel: ProductVariationModel

	/** Checks if product is bound to specified catalog category. @deprecated Use isAssignedToCategory(Category) */
	assignedToCategory(category: Category): boolean

	/** Returns active data for this product for the current site. */
	getActiveData(): ProductActiveData

	/** Returns all categories to which this product is assigned. */
	getAllCategories(): Collection

	/** Returns all category assignments for this product in any catalog. */
	getAllCategoryAssignments(): Collection

	/** Returns all incoming ProductLinks. */
	getAllIncomingProductLinks(): Collection

	/** Returns all incoming ProductLinks of a specific type. */
	getAllIncomingProductLinks(type: number): Collection

	/** Returns all outgoing ProductLinks. */
	getAllProductLinks(): Collection

	/** Returns all outgoing ProductLinks of a specific type. */
	getAllProductLinks(type: number): Collection

	/** Returns outgoing recommendations for this product in specified catalog, sorted by order. */
	getAllRecommendations(catalog: Catalog): Collection

	/** Returns outgoing recommendations of specified type in specified catalog, sorted by order. */
	getAllRecommendations(catalog: Catalog, type: number): Collection

	/** Returns ProductAttributeModel for convenient product attribute information access. */
	getAttributeModel(): ProductAttributeModel

	/** Returns availability model for determining availability information. */
	getAvailabilityModel(): ProductAvailabilityModel

	/** Returns availability model of given inventory list. */
	getAvailabilityModel(list: ProductInventoryList): ProductAvailabilityModel

	/** Returns availability status flag. @deprecated Use getAvailabilityModel() instead */
	getAvailableFlag(): boolean

	/** Returns product brand. */
	getBrand(): string

	/** Returns quantity of specified product within bundle or 0 if not part of bundle. */
	getBundledProductQuantity(aProduct: Product): Quantity

	/** Returns all products that participate in the product bundle. */
	getBundledProducts(): Collection

	/** Returns all bundles in which this product is included (site-assigned only). */
	getBundles(): Collection

	/** Returns all categories to which product is assigned and available through current site. */
	getCategories(): Collection

	/** Returns category assignment for a specific category. */
	getCategoryAssignment(category: Category): CategoryAssignment

	/** Returns category assignments for this product in current site catalog. */
	getCategoryAssignments(): Collection

	/** Returns classification category associated with this product. */
	getClassificationCategory(): Category

	/** Returns European Article Number of product. */
	getEAN(): string

	/** Returns ID of the product. */
	getID(): string

	/** Returns product's image. @deprecated Use getImages(String) and getImage(String, Number) instead */
	getImage(): MediaFile

	/** Returns image at specific index for view type, or null if not available. */
	getImage(viewtype: string, index: number): MediaFile

	/** Returns first image for view type. */
	getImage(viewtype: string): MediaFile

	/** Returns all images assigned to product for specific view type. */
	getImages(viewtype: string): List

	/** Returns incoming ProductLinks where source product is a site product. */
	getIncomingProductLinks(): Collection

	/** Returns incoming ProductLinks of specific type where source product is a site product. */
	getIncomingProductLinks(type: number): Collection

	/** Returns product's long description in current locale. */
	getLongDescription(): MarkupText

	/** Returns name of product manufacturer. */
	getManufacturerName(): string

	/** Returns manufacturer's stock keeping unit value. */
	getManufacturerSKU(): string

	/** Returns minimum order quantity for this product. */
	getMinOrderQuantity(): Quantity

	/** Returns product name in current locale. */
	getName(): string

	/** Returns all currently online categories to which product is assigned and available through current site. */
	getOnlineCategories(): Collection

	/** Returns online status flag of product. */
	getOnlineFlag(): boolean

	/** Returns date from which product is online or valid. */
	getOnlineFrom(): Date

	/** Returns date until which product is online or valid. */
	getOnlineTo(): Date

	/** Returns product's option model with initialized option values. */
	getOptionModel(): ProductOptionModel

	/** Returns outgoing recommendations for this product with orderable target products only. */
	getOrderableRecommendations(): Collection

	/** Returns outgoing recommendations of specific type with orderable target products only. */
	getOrderableRecommendations(type: number): Collection

	/** Returns product's page description in default locale. */
	getPageDescription(): string

	/** Returns product's page keywords in default locale. */
	getPageKeywords(): string

	/** Returns page meta tag for specified id. */
	getPageMetaTag(id: string): PageMetaTag

	/** Returns all page meta tags defined for this instance for which content can be generated. */
	getPageMetaTags(): Array

	/** Returns product's page title in default locale. */
	getPageTitle(): string

	/** Returns product's page URL in default locale. */
	getPageURL(): string

	/** Returns price model for retrieving price for this product. */
	getPriceModel(): ProductPriceModel

	/** Returns price model based on specified option model. */
	getPriceModel(optionModel: ProductOptionModel): ProductPriceModel

	/** Returns primary category of product within current site catalog. */
	getPrimaryCategory(): Category

	/** Returns category assignment to primary category in current site catalog or null. */
	getPrimaryCategoryAssignment(): CategoryAssignment

	/** Returns all outgoing ProductLinks where target product is available in current site (unsorted). */
	getProductLinks(): Collection

	/** Returns all outgoing ProductLinks of specific type where target product is available in current site. */
	getProductLinks(type: number): Collection

	/** Returns all products assigned to this product and available through current site. */
	getProductSetProducts(): Collection

	/** Returns all product sets in which this product is included (site-assigned only). */
	getProductSets(): Collection

	/** Returns outgoing recommendations for this product in site catalog, sorted by order. */
	getRecommendations(): Collection

	/** Returns outgoing recommendations of specified type in site catalog, sorted by order. */
	getRecommendations(type: number): Collection

	/** Returns true if product is currently searchable. */
	getSearchableFlag(): boolean

	/** Returns searchable status of Product if unavailable. Null indicates value not set. */
	getSearchableIfUnavailableFlag(): boolean

	/** Returns product's search placement classification. Higher value indicates more relevance. */
	getSearchPlacement(): number

	/** Returns product's search rank. Higher value indicates more relevance. */
	getSearchRank(): number

	/** Returns product's short description in current locale. */
	getShortDescription(): MarkupText

	/** Returns product's change frequency needed for sitemap creation. */
	getSiteMapChangeFrequency(): string

	/** Returns status if product is included in sitemap. */
	getSiteMapIncluded(): number

	/** Returns product's priority needed for sitemap creation. */
	getSiteMapPriority(): number

	/** Returns steps in which order amount of product can be increased. */
	getStepQuantity(): Quantity

	/** Returns store receipt name of product in current locale. */
	getStoreReceiptName(): string

	/** Returns store tax class ID (optional override for in-store tax calculation). */
	getStoreTaxClass(): string

	/** Returns product's tax class ID resolved by Global Preference setting. */
	getTaxClassID(): string

	/** Returns name of product's rendering template. */
	getTemplate(): string

	/** Returns product's thumbnail image. @deprecated Use getImages(String) and getImage(String, Number) instead */
	getThumbnail(): MediaFile

	/** Returns product's sales unit. */
	getUnit(): string

	/** Returns product's unit quantity. */
	getUnitQuantity(): Quantity

	/** Returns Universal Product Code of product. */
	getUPC(): string

	/** Returns all variants assigned to this variation master or variation group product. */
	getVariants(): Collection

	/** Returns all variation groups assigned to this variation master product. */
	getVariationGroups(): Collection

	/** Returns variation model of this product with pre-selected attribute values if variant. */
	getVariationModel(): ProductVariationModel

	/** Checks if specified product participates in this product bundle. */
	includedInBundle(product: Product): boolean

	/** Returns true if item is assigned to specified category. */
	isAssignedToCategory(category: Category): boolean

	/** Returns true if product is assigned to current site via site catalog. */
	isAssignedToSiteCatalog(): boolean

	/** Checks if product is available. @deprecated Use getAvailabilityModel().isInStock() instead */
	isAvailable(): boolean

	/** Checks if this product instance is a product bundle. */
	isBundle(): boolean

	/** Checks if this product is bundled within at least one product bundle. */
	isBundled(): boolean

	/** Checks if product is bound to at least one catalog category. */
	isCategorized(): boolean

	/** Checks if product is Facebook enabled. */
	isFacebookEnabled(): boolean

	/** Checks if this product instance is a product master. */
	isMaster(): boolean

	/** Returns online status of product calculated from online flag and onlineFrom/onlineTo dates. */
	isOnline(): boolean

	/** Checks if product has options. */
	isOptionProduct(): boolean

	/** Checks if product is Pinterest enabled. */
	isPinterestEnabled(): boolean

	/** Returns true if instance represents a product, false if product set. */
	isProduct(): boolean

	/** Returns true if instance represents a product set. */
	isProductSet(): boolean

	/** Returns true if this product is part of any product set. */
	isProductSetProduct(): boolean

	/** Checks if this product instance is part of a retail set. @deprecated Use isProductSet() instead */
	isRetailSet(): boolean

	/** Checks if product is searchable. */
	isSearchable(): boolean

	/** Returns true if product is assigned to current site via site catalog. @deprecated Use isAssignedToSiteCatalog() instead */
	isSiteProduct(): boolean

	/** Checks if this product instance is mastered by a product master. */
	isVariant(): boolean

	/** Checks if this product instance is a variation group product. */
	isVariationGroup(): boolean

	/** Set availability status flag of product. @deprecated Use getAvailabilityModel() instead */
	setAvailableFlag(available: boolean): void

	/** Set online status flag of product for current site. */
	setOnlineFlag(online: boolean): void

	/** Set flag indicating whether product is searchable in context of current site. */
	setSearchableFlag(searchable: boolean): void

	/** Set product's search placement classification in context of current site. */
	setSearchPlacement(placement: number): void

	/** Set product's search rank in context of current site. */
	setSearchRank(rank: number): void
}
```
