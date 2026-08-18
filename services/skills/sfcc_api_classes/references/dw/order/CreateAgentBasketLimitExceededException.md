# dw.order.CreateAgentBasketLimitExceededException

## Overview
Exception thrown when creating an agent basket fails because the session's open agent basket limit is reached.

## Description
Thrown by `BasketMgr.createAgentBasket()` to indicate the open agent basket limit for the current session customer is already reached.

```ts
declare class CreateAgentBasketLimitExceededException extends Error {
    /** No public constructor; this exception is thrown by platform APIs. */
}
```
