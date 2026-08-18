# dw.customer.ProductListItem

## Overview
Represents a single entry in a ProductList (product or gift certificate) with quantity, priority, and purchase history.

## Description
Provides access to the referenced product, quantities, purchases and visibility flags. Includes constants for item types and methods to create purchase records and update item fields.

```ts
declare class ProductListItem extends ExtensibleObject {
  /** Constant: product item type. */
  static TYPE_PRODUCT: 1

  /** Constant: gift certificate item type. */
  static TYPE_GIFT_CERTIFICATE: 2

  /** Read-only: unique ID. */
  ID: string

  /** Read-only: owning ProductList. */
  list: ProductList

  /** Priority level (lower = higher priority). */
  priority: number

  /** Referenced `Product` (may be null if missing). */
  product: Product | null

  /** Read-only: productID string. */
  productID: string

  /** Product option model, or null. */
  productOptionModel: ProductOptionModel | null

  /** Visibility flag for customers other than owner. */
  public: boolean

  /** Read-only: total purchased quantity (Quantity object). */
  purchasedQuantity: Quantity

  /** Read-only: numeric value of purchased quantity. */
  purchasedQuantityValue: number

  /** Purchases collection. */
  purchases: Collection

  /** Quantity object for this item. */
  quantity: Quantity

  /** Numeric quantity value. */
  quantityValue: number

  /** Read-only: item type code. */
  type: number

  /** Create a purchase record for this item. */
  createPurchase(quantity: number, purchaserName: string): ProductListItemPurchase

  /** Getters and setters. */
  getID(): string
  getList(): ProductList
  getPriority(): number
  getProduct(): Product | null
  getProductID(): string
  getProductOptionModel(): ProductOptionModel | null
  getPurchasedQuantity(): Quantity
  getPurchasedQuantityValue(): number
  getPurchases(): Collection
  getQuantity(): Quantity
  getQuantityValue(): number
  getType(): number
  isPublic(): boolean
  setPriority(priority: number): void
  setProduct(product: Product | null): void
  setProductOptionModel(productOptionModel: ProductOptionModel): void
  setPublic(flag: boolean): void
  setQuantity(value: Quantity): void
  setQuantityValue(value: number): void
}
```
