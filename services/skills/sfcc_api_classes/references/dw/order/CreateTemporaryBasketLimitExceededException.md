# dw.order.CreateTemporaryBasketLimitExceededException

## Overview
Exception thrown when the session's open temporary basket limit is reached and a new temporary basket cannot be created.

## Description
Thrown by `BasketMgr.createTemporaryBasket()` to indicate the open temporary basket limit for the current session customer has been reached.

```ts
declare class CreateTemporaryBasketLimitExceededException extends Error {
    /** No public constructor; thrown by the platform when temporary basket limit is reached. */
}
```
