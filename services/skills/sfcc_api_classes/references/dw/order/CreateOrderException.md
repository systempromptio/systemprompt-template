# dw.order.CreateOrderException

## Overview
APIException thrown when `OrderMgr.createOrder` fails to create an Order from a Basket.

## Description
Indicates no Order could be created from the given Basket. Thrown by `OrderMgr.createOrder(Basket, String)`.

```ts
declare class CreateOrderException extends Error {
    /** No public constructor; thrown by platform APIs when order creation fails. */
}
```
