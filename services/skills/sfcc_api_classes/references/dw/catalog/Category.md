# dw.catalog.Category

## Overview
Represents a category in a product catalog, supporting category hierarchy, product assignments, recommendations, and online/offline status. Provides access to category metadata, sorting, SEO, and display options.

## Description
A Category models a node in the catalog's category tree. It manages product assignments, subcategories, recommendations, and metadata such as display name, description, images, and SEO properties. Categories can be online or offline, have display modes, and support sorting and search configuration. Not directly instantiable.

## Inheritance
Object → PersistentObject → ExtensibleObject → Category

```ts
declare class Category extends ExtensibleObject {
	/**
	 * Constant for Variation Group Display Mode: individual (0).
	 */
	static DISPLAY_MODE_INDIVIDUAL: number
	/**
	 * Constant for Variation Group Display Mode: merged (1).
	 */
	static DISPLAY_MODE_MERGED: number

	/**
	 * All outgoing recommendations for this category, sorted by order.
	 */
	readonly allRecommendations: Collection
	/**
	 * Category assignments for this category.
	 */
	readonly categoryAssignments: Collection
	/**
	 * Default sorting rule for this category, or null if not set.
	 */
	readonly defaultSortingRule: SortingRule | null
	/**
	 * Description for the current locale.
	 */
	readonly description: string | null
	/**
	 * Variation Groups Display Mode or null if not defined.
	 */
	displayMode: number | null
	/**
	 * Display name for the current locale.
	 */
	readonly displayName: string | null
	/**
	 * Category ID.
	 */
	readonly ID: string
	/**
	 * Image reference for this category.
	 */
	readonly image: MediaFile | null
	/**
	 * Incoming CategoryLink objects where this is the target.
	 */
	readonly incomingCategoryLinks: Collection
	/**
	 * True if the category is currently online.
	 */
	readonly online: boolean
	/**
	 * Online category assignments (products currently online).
	 */
	readonly onlineCategoryAssignments: Collection
	/**
	 * Online status flag.
	 */
	readonly onlineFlag: boolean
	/**
	 * Date from which the category is online.
	 */
	readonly onlineFrom: Date | null
	/**
	 * Online incoming CategoryLink objects.
	 */
	readonly onlineIncomingCategoryLinks: Collection
	/**
	 * Online outgoing CategoryLink objects.
	 */
	readonly onlineOutgoingCategoryLinks: Collection
	/**
	 * Online products assigned to this category.
	 */
	readonly onlineProducts: Collection
	/**
	 * Online subcategories, sorted by position.
	 */
	readonly onlineSubCategories: Collection
	/**
	 * Date until which the category is online.
	 */
	readonly onlineTo: Date | null
	/**
	 * Outgoing recommendations for this category, orderable only.
	 */
	readonly orderableRecommendations: Collection
	/**
	 * Outgoing CategoryLink objects where this is the source.
	 */
	readonly outgoingCategoryLinks: Collection
	/**
	 * Page description for SEO.
	 */
	readonly pageDescription: string | null
	/**
	 * Page keywords for SEO.
	 */
	readonly pageKeywords: string | null
	/**
	 * Page title for SEO.
	 */
	readonly pageTitle: string | null
	/**
	 * Page URL for this category.
	 */
	readonly pageURL: string | null
	/**
	 * Parent category, or null if root.
	 */
	readonly parent: Category | null
	/**
	 * ProductAttributeModel for this category.
	 */
	readonly productAttributeModel: ProductAttributeModel
	/**
	 * All products assigned to this category.
	 */
	readonly products: Collection
	/**
	 * Outgoing recommendations for this category.
	 */
	readonly recommendations: Collection
	/**
	 * True if this is the root category.
	 */
	readonly root: boolean
	/**
	 * Search placement value or null.
	 */
	searchPlacement: number | null
	/**
	 * Search rank value or null.
	 */
	searchRank: number | null
	/**
	 * Sitemap change frequency.
	 */
	readonly siteMapChangeFrequency: string | null
	/**
	 * Sitemap inclusion value.
	 */
	readonly siteMapIncluded: number | null
	/**
	 * Sitemap priority value.
	 */
	readonly siteMapPriority: number | null
	/**
	 * All subcategories, sorted by position.
	 */
	readonly subCategories: Collection
	/**
	 * Template file name for this category.
	 */
	readonly template: string | null
	/**
	 * Thumbnail image reference.
	 */
	readonly thumbnail: MediaFile | null
	/**
	 * True if this is a top-level category (not root).
	 */
	readonly topLevel: boolean

	/**
	 * Returns all outgoing recommendations for this category.
	 */
	getAllRecommendations(): Collection
	/**
	 * Returns all outgoing recommendations of the specified type.
	 * @param type Recommendation type
	 */
	getAllRecommendations(type: number): Collection
	/**
	 * Returns category assignments for this category.
	 */
	getCategoryAssignments(): Collection
	/**
	 * Returns the default sorting rule or null.
	 */
	getDefaultSortingRule(): SortingRule | null
	/**
	 * Returns the description for the current locale.
	 */
	getDescription(): string | null
	/**
	 * Returns the display mode or null.
	 */
	getDisplayMode(): number | null
	/**
	 * Returns the display name for the current locale.
	 */
	getDisplayName(): string | null
	/**
	 * Returns the category ID.
	 */
	getID(): string
	/**
	 * Returns the image reference.
	 */
	getImage(): MediaFile | null
	/**
	 * Returns incoming CategoryLink objects.
	 */
	getIncomingCategoryLinks(): Collection
	/**
	 * Returns incoming CategoryLink objects of the specified type.
	 * @param type Link type
	 */
	getIncomingCategoryLinks(type: number): Collection
	/**
	 * Returns online category assignments.
	 */
	getOnlineCategoryAssignments(): Collection
	/**
	 * Returns the online status flag.
	 */
	getOnlineFlag(): boolean
	/**
	 * Returns the online from date.
	 */
	getOnlineFrom(): Date | null
	/**
	 * Returns online incoming CategoryLink objects.
	 */
	getOnlineIncomingCategoryLinks(): Collection
	/**
	 * Returns online outgoing CategoryLink objects.
	 */
	getOnlineOutgoingCategoryLinks(): Collection
	/**
	 * Returns online products assigned to this category.
	 */
	getOnlineProducts(): Collection
	/**
	 * Returns online subcategories.
	 */
	getOnlineSubCategories(): Collection
	/**
	 * Returns the online to date.
	 */
	getOnlineTo(): Date | null
	/**
	 * Returns orderable recommendations for this category.
	 */
	getOrderableRecommendations(): Collection
	/**
	 * Returns orderable recommendations of the specified type.
	 * @param type Recommendation type
	 */
	getOrderableRecommendations(type: number): Collection
	/**
	 * Returns outgoing CategoryLink objects.
	 */
	getOutgoingCategoryLinks(): Collection
	/**
	 * Returns outgoing CategoryLink objects of the specified type.
	 * @param type Link type
	 */
	getOutgoingCategoryLinks(type: number): Collection
	/**
	 * Returns the page description.
	 */
	getPageDescription(): string | null
	/**
	 * Returns the page keywords.
	 */
	getPageKeywords(): string | null
	/**
	 * Returns the page title.
	 */
	getPageTitle(): string | null
	/**
	 * Returns the page URL.
	 */
	getPageURL(): string | null
	/**
	 * Returns the parent category or null.
	 */
	getParent(): Category | null
	/**
	 * Returns the ProductAttributeModel for this category.
	 */
	getProductAttributeModel(): ProductAttributeModel
	/**
	 * Returns all products assigned to this category.
	 */
	getProducts(): Collection
	/**
	 * Returns outgoing recommendations for this category.
	 */
	getRecommendations(): Collection
	/**
	 * Returns outgoing recommendations of the specified type.
	 * @param type Recommendation type
	 */
	getRecommendations(type: number): Collection
	/**
	 * Returns the search placement value or null.
	 */
	getSearchPlacement(): number | null
	/**
	 * Returns the search rank value or null.
	 */
	getSearchRank(): number | null
	/**
	 * Returns the sitemap change frequency.
	 */
	getSiteMapChangeFrequency(): string | null
	/**
	 * Returns the sitemap inclusion value.
	 */
	getSiteMapIncluded(): number | null
	/**
	 * Returns the sitemap priority value.
	 */
	getSiteMapPriority(): number | null
	/**
	 * Returns all subcategories.
	 */
	getSubCategories(): Collection
	/**
	 * Returns the template file name.
	 */
	getTemplate(): string | null
	/**
	 * Returns the thumbnail image reference.
	 */
	getThumbnail(): MediaFile | null
	/**
	 * Returns true if this category has any online products.
	 */
	hasOnlineProducts(): boolean
	/**
	 * Returns true if this category has any online subcategories.
	 */
	hasOnlineSubCategories(): boolean
	/**
	 * Returns true if this is a direct subcategory of the given parent.
	 * @param parent Parent category
	 */
	isDirectSubCategoryOf(parent: Category): boolean
	/**
	 * Returns true if the category is currently online.
	 */
	isOnline(): boolean
	/**
	 * Returns true if this is the root category.
	 */
	isRoot(): boolean
	/**
	 * Returns true if this is a subcategory of the given ancestor.
	 * @param ancestor Ancestor category
	 */
	isSubCategoryOf(ancestor: Category): boolean
	/**
	 * Returns true if this is a top-level category (not root).
	 */
	isTopLevel(): boolean
	/**
	 * Sets the display mode for this category.
	 * @param displayMode Display mode value
	 */
	setDisplayMode(displayMode: number | null): void
	/**
	 * Sets the search placement value.
	 * @param placement Search placement value
	 */
	setSearchPlacement(placement: number): void
	/**
	 * Sets the search rank value.
	 * @param rank Search rank value
	 */
	setSearchRank(rank: number): void
}
```
