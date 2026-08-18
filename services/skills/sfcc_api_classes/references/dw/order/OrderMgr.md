# dw.order.OrderMgr

## Overview
Manager for orders — lookup and retrieval helpers.

## Description
OrderMgr provides methods to locate and query orders, e.g., by orderNo, customer or query.

```ts
declare class OrderMgr  {
    /** Retrieves an order by order number. */
    static getOrder(orderNo: string): Order

    /** Returns an iterator or collection of orders matching query (signature varies). */
    static queryOrders(query: string): any
}
```
