# dw.customer.OrderHistory

## Overview
Provides access to a customer's past orders in the current storefront site.

## Description
Retrieves order counts and iterators for a customer's historical orders. Some methods do not work with Salesforce Order Management orders. Returns resources such as `SeekableIterator` for results; callers should close iterators when not fully iterating.

```ts
declare class OrderHistory {
  /** Read-only: number of orders the customer placed in the store. */
  orderCount: number

  /** Read-only: iterator over the customer's orders for the current site. */
  orders: SeekableIterator

  /** Returns number of orders placed by the customer. */
  getOrderCount(): number

  /** Returns a SeekableIterator with the customer's orders (default ordering). */
  getOrders(): SeekableIterator

  /**
   * Returns a SeekableIterator with orders matching `query` and `sortString`.
   * @param query optional query expression (max 3 expressions)
   * @param sortString optional sort specification
   * @param params optional parameters for the query
   */
  getOrders(query: string, sortString: string, ...params: Object): SeekableIterator
}
```
