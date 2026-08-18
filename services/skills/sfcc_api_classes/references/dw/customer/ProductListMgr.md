# dw.customer.ProductListMgr

## Overview
Static manager with methods to create, retrieve, search and remove `ProductList` instances.

## Description
Provides factory and query methods for `ProductList` objects: create by customer and type, get by ID or profile, list retrieval by customer or address, and flexible query functions returning `SeekableIterator` or `Collection`.

```ts
declare class ProductListMgr {
  /** Creates a new ProductList for `customer` with the given `type`. */
  static createProductList(customer: Customer, type: number): ProductList

  /** Returns a ProductList by its ID, or null. */
  static getProductList(ID: string): ProductList | null

  /** Deprecated: returns first ProductList for `profile` and `type`. Use getProductLists instead. */
  static getProductList(profile: Profile, type: number): ProductList | null

  /** Returns unsorted collection of product lists for the specified customer and type. */
  static getProductLists(customer: Customer, type: number): Collection

  /** Returns product lists for customer filtered by eventType. */
  static getProductLists(customer: Customer, type: number, eventType: string): Collection

  /** Returns lists that use the specified address as shipping address. */
  static getProductLists(customerAddress: CustomerAddress): Collection

  /** Query product lists using a map of attributes, returns SeekableIterator. */
  static queryProductLists(queryAttributes: Map, sortString: string): SeekableIterator

  /** Query product lists using query string, returns SeekableIterator. */
  static queryProductLists(queryString: string, sortString: string, ...args: Object): SeekableIterator

  /** Removes the given ProductList from the system. */
  static removeProductList(productList: ProductList): void
}
```
