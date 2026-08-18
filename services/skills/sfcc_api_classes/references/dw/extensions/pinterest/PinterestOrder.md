# dw.extensions.pinterest.PinterestOrder

## Overview
Represents an order placed through Pinterest, including item ID, order number, status, and payment status constants.

## Description
Container class for Pinterest orders with several status constants (PAID, NOT_PAID, PART_PAID) and methods to get/set itemId, orderNo, paymentStatus, and status.

```ts
declare class PinterestOrder  {
  static PAYMENT_STATUS_NOT_PAID: 'NOT_PAID'
  static PAYMENT_STATUS_PAID: 'PAID'
  static PAYMENT_STATUS_PART_PAID: 'PART_PAID'
  static STATUS_BACKORDER: 'BACKORDER'
  static STATUS_CANCELLED: 'CANCELLED'
  static STATUS_DELIVERED: 'DELIVERED'
  static STATUS_IN_PROGRESS: 'IN_PROGRESS'
  static STATUS_NEW: 'NEW'
  static STATUS_RETURNED: 'RETURNED'
  static STATUS_SHIPPED: 'SHIPPED'

  /** The item ID for this Pinterest order. */
  itemId: string

  /** The order number for this Pinterest order (same as Demandware order). */
  orderNo: string | null

  /** The payment status for this Pinterest order. */
  paymentStatus: string

  /** The status for this Pinterest order. */
  status: string

  getItemId(): string
  getOrderNo(): string | null
  getPaymentStatus(): string
  getStatus(): string

  setItemId(itemId: string): void
  setPaymentStatus(status: string): void
  setStatus(status: string): void
}
```

All Known Subclasses

None
