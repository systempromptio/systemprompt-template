# dw.customer.shoppercontext.ShopperContextException

## Overview
Exception thrown for errors during shopper context operations (set, get, remove); includes an `errorCode` property describing the reason.

## Description
Thrown by ShopperContextMgr methods when an error occurs while saving, retrieving, or deleting Shopper Context. The `errorCode` property contains one of the values from `ShopperContextErrorCodes` to indicate the failure reason.

## All Known Subclasses


```ts
declare class ShopperContextException extends APIException {
	/** Read-only error code describing the failure. */
	errorCode: string
}
```
