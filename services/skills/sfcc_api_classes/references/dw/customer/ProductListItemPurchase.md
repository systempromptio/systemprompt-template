# dw.customer.ProductListItemPurchase

## Overview
Represents a record of a purchase for an item in a ProductList.

## Description
Contains metadata about the purchase: purchaser name, order number, date and quantity. All primary fields are read-only.

```ts
declare class ProductListItemPurchase extends ExtensibleObject {
  /** Read-only: the `ProductListItem` that was purchased. */
  item: ProductListItem

  /** Read-only: order number for the purchase. */
  orderNo: string

  /** Read-only: purchase date. */
  purchaseDate: Date

  /** Read-only: purchaser name. */
  purchaserName: string

  /** Read-only: purchased quantity. */
  quantity: Quantity

  getItem(): ProductListItem
  getOrderNo(): string
  getPurchaseDate(): Date
  getPurchaserName(): string
  getQuantity(): Quantity
}
```
