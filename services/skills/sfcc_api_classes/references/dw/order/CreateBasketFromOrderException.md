# dw.order.CreateBasketFromOrderException

## Overview
APIException thrown when no Basket can be created from an Order via `BasketMgr.createBasketFromOrder`.

## Description
Indicates the platform could not create a Basket from the provided Order. Exposes a read-only `errorCode` describing the reason.

```ts
declare class CreateBasketFromOrderException extends Error {
    /** Reason code indicating why Basket creation failed. */
    errorCode: string
}
```
