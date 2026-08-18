# dw.order.CreateCouponLineItemException

## Overview
Exception indicating a coupon code provided to `LineItemCtnr.createCouponLineItem` is invalid or otherwise unacceptable.

## Description
Thrown when the provided coupon code is invalid. The `errorCode` property contains one of the values from `CouponStatusCodes` describing the cause (for example, COUPON_CODE_ALREADY_IN_BASKET, COUPON_DISABLED, NO_ACTIVE_PROMOTION, etc.).

```ts
declare class CreateCouponLineItemException extends Error {
    /** Read-only error code describing why the coupon creation failed. */
    errorCode: string
}
```
