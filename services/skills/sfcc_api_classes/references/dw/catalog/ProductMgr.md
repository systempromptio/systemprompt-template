# dw.catalog.ProductMgr

## Overview
Provides helper methods for getting products by ID or catalog assignment.

## Description
Provides helper methods for getting products based on Product ID or Catalog.

```
Object
  dw.catalog.ProductMgr
```

```ts
declare class ProductMgr  {
	/**
	 * Returns the product with the specified id.
	 * @param productID - The product identifier
	 * @returns Product for specified id or null
	 */
	static getProduct(productID: string): Product

	/**
	 * Returns all products assigned to the current site. A product is assigned to a site if it is assigned to at least one category of the site catalog or it is a variant and its master product is assigned to the current site. It is strongly recommended to call close() on the returned SeekableIterator if not all of its elements are being retrieved. This will ensure the proper cleanup of system resources.
	 * @returns Iterator of all site products
	 */
	static queryAllSiteProducts(): SeekableIterator

	/**
	 * Returns all products assigned to the current site. Works like queryAllSiteProducts(), but additionally sorts the result set by product ID. It is strongly recommended to call close() on the returned SeekableIterator if not all of its elements are being retrieved.
	 * @returns Iterator of all site products sorted by product ID
	 */
	static queryAllSiteProductsSorted(): SeekableIterator

	/**
	 * Returns all products assigned to the the specified catalog, where assignment has the same meaning as it does for queryAllSiteProducts(). It is strongly recommended to call close() on the returned SeekableIterator if not all of its elements are being retrieved.
	 * @param catalog - The catalog whose assigned products should be returned
	 * @returns Iterator of all products assigned to specified catalog
	 */
	static queryProductsInCatalog(catalog: Catalog): SeekableIterator

	/**
	 * Returns all products assigned to the the specified catalog. Works like queryProductsInCatalog(), but additionally sorts the result set by product ID. It is strongly recommended to call close() on the returned SeekableIterator if not all of its elements are being retrieved.
	 * @param catalog - The catalog whose assigned products should be returned
	 * @returns Iterator of all products assigned to specified catalog sorted by product ID
	 */
	static queryProductsInCatalogSorted(catalog: Catalog): SeekableIterator
}
```
