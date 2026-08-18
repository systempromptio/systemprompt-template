# dw.order.ProductLineItem

## Overview
Represents a specific product line item.

## Description
Represents a specific product line item.

```ts
declare class ProductLineItem extends dw.order.LineItem {
    /** The gross price of the product line item after applying all product-level adjustments. */
    readonly adjustedGrossPrice: dw.value.Money

    /** The net price of the product line item after applying all product-level adjustments. */
    readonly adjustedNetPrice: dw.value.Money

    /** The price of the product line item after applying all product-level adjustments. */
    readonly adjustedPrice: dw.value.Money

    /** The tax of the unit after applying adjustments, in the purchase currency. */
    readonly adjustedTax: dw.value.Money

    /** The parent bonus discount line item of this line item. */
    readonly bonusDiscountLineItem: dw.order.BonusDiscountLineItem

    /** Identifies if the product line item represents a bonus line item. */
    readonly bonusProductLineItem: boolean

    /** Identifies if the product line item represents a bundled line item. */
    readonly bundledProductLineItem: boolean

    /** A collection containing the bundled product line items. */
    readonly bundledProductLineItems: dw.util.Collection

    /** Returns true if the product line item represents a catalog product. */
    readonly catalogProduct: boolean

    /** The category the product line item is associated with. */
    category: dw.catalog.Category

    /** The ID of the category the product line item is associated with. */
    categoryID: string

    /** The value set for the external line item status or null if no value set. */
    externalLineItemStatus: string

    /** The value set for the external line item text or null if no value set. */
    externalLineItemText: string

    /** Returns true if this line item represents a gift. */
    gift: boolean

    /** The value set for gift message or null if no value set. */
    giftMessage: string

    /** The name of the manufacturer of the product. */
    manufacturerName: string

    /** The manufacturer's SKU of this product line item. */
    manufacturerSKU: string

    /** The minimal order quantity allowed for the product represented by the ProductLineItem. */
    readonly minOrderQuantity: dw.value.Quantity

    /** Return the value portion of getMinOrderQuantity(). */
    readonly minOrderQuantityValue: number

    /** The ID of the product option this product line item represents. */
    readonly optionID: string

    /** The product option model for a product line item representing an option product. */
    readonly optionModel: dw.catalog.ProductOptionModel

    /** Identifies if the product line item represents an option line item. */
    readonly optionProductLineItem: boolean

    /** A collection containing option product line items. */
    readonly optionProductLineItems: dw.util.Collection

    /** The ID of the product option value this product line item represents. */
    readonly optionValueID: string

    /** The order-item extension for this item, or null. */
    readonly orderItem: dw.order.OrderItem

    /** The parent line item of this line item or null if independent. */
    readonly parent: ProductLineItem

    /** The position within the line item container. */
    position: number

    /** An iterator of price adjustments applied to this product line item. */
    readonly priceAdjustments: dw.util.Collection

    /** The product associated with the product line item. */
    readonly product: dw.catalog.Product

    /** The ID of the related product. */
    readonly productID: string

    /** The inventory list the product line item is associated with. */
    productInventoryList: dw.catalog.ProductInventoryList

    /** The ID of the inventory list the product line item is associated with. */
    productInventoryListID: string

    /** The associated ProductListItem. */
    readonly productListItem: dw.customer.ProductListItem

    /** The name of the product copied when added to the line item container. */
    productName: string

    /** The price of this product line item after prorating adjustments. */
    readonly proratedPrice: dw.value.Money

    /** A map of PriceAdjustment to Money instances for prorated prices. */
    readonly proratedPriceAdjustmentPrices: dw.util.Map

    /** The qualifying ProductLineItem for a bonus product, if any. */
    readonly qualifyingProductLineItemForBonusProduct: ProductLineItem

    /** The quantity of the product represented by this ProductLineItem. */
    readonly quantity: dw.value.Quantity

    /** The numeric value of quantity. */
    readonly quantityValue: number

    /** All bonus product line items for which this line item is a qualifying product. */
    readonly relatedBonusProductLineItems: dw.util.Collection

    /** Returns if the product line item is reserved. */
    readonly reserved: boolean

    /** The associated Shipment. */
    shipment: dw.order.Shipment

    /** The dependent shipping line item of this line item. */
    readonly shippingLineItem: dw.order.ProductShippingLineItem

    /** Returns step quantity allowed for the product. */
    readonly stepQuantity: dw.value.Quantity

    /** Return the numeric value of step quantity. */
    readonly stepQuantityValue: number

    /** Creates a product price adjustment. */
    createPriceAdjustment(promotionID: string): dw.order.PriceAdjustment

    /** Creates a product price adjustment representing a specific discount. */
    createPriceAdjustment(promotionID: string, discount: dw.campaign.Discount): dw.order.PriceAdjustment

    /** Creates the dependent shipping line item for this line item. */
    createShippingLineItem(): dw.order.ProductShippingLineItem

    /** Returns the gross adjusted price. */
    getAdjustedGrossPrice(): dw.value.Money

    /** Returns the net adjusted price. */
    getAdjustedNetPrice(): dw.value.Money

    /** Returns the adjusted price. */
    getAdjustedPrice(): dw.value.Money

    /** Returns adjusted price with optional order-level adjustments. */
    getAdjustedPrice(applyOrderLevelAdjustments: boolean): dw.value.Money

    /** Returns adjusted tax. */
    getAdjustedTax(): dw.value.Money

    /** Returns the parent bonus discount line item. */
    getBonusDiscountLineItem(): dw.order.BonusDiscountLineItem

    /** Returns bundled product line items. */
    getBundledProductLineItems(): dw.util.Collection

    /** Returns the category. */
    getCategory(): dw.catalog.Category

    /** Returns the category ID. */
    getCategoryID(): string

    /** Returns external line item status. */
    getExternalLineItemStatus(): string

    /** Returns external line item text. */
    getExternalLineItemText(): string

    /** Returns gift message. */
    getGiftMessage(): string

    /** Returns manufacturer name. */
    getManufacturerName(): string

    /** Returns manufacturer SKU. */
    getManufacturerSKU(): string

    /** Returns min order quantity. */
    getMinOrderQuantity(): dw.value.Quantity

    /** Returns min order quantity value. */
    getMinOrderQuantityValue(): number

    /** Returns option ID. */
    getOptionID(): string

    /** Returns option model. */
    getOptionModel(): dw.catalog.ProductOptionModel

    /** Returns option product line items. */
    getOptionProductLineItems(): dw.util.Collection

    /** Returns option value ID. */
    getOptionValueID(): string

    /** Returns order item extension. */
    getOrderItem(): dw.order.OrderItem

    /** Returns parent. */
    getParent(): ProductLineItem

    /** Returns position. */
    getPosition(): number

    /** Returns price adjustment by promotion ID. */
    getPriceAdjustmentByPromotionID(promotionID: string): dw.order.PriceAdjustment

    /** Returns price adjustment by promotion ID and coupon code. */
    getPriceAdjustmentByPromotionIDAndCouponCode(promotionID: string, couponCode: string): dw.order.PriceAdjustment

    /** Returns price adjustments. */
    getPriceAdjustments(): dw.util.Collection

    /** Returns price adjustments by promotion ID. */
    getPriceAdjustmentsByPromotionID(promotionID: string): dw.util.Collection

    /** Returns product. */
    getProduct(): dw.catalog.Product

    /** Returns product ID. */
    getProductID(): string

    /** Returns product inventory list. */
    getProductInventoryList(): dw.catalog.ProductInventoryList

    /** Returns product inventory list ID. */
    getProductInventoryListID(): string

    /** Returns product list item. */
    getProductListItem(): dw.customer.ProductListItem

    /** Returns product name. */
    getProductName(): string

    /** Returns prorated price. */
    getProratedPrice(): dw.value.Money

    /** Returns prorated price adjustment prices map. */
    getProratedPriceAdjustmentPrices(): dw.util.Map

    /** Returns qualifying product line item for bonus product. */
    getQualifyingProductLineItemForBonusProduct(): ProductLineItem

    /** Returns quantity. */
    getQuantity(): dw.value.Quantity

    /** Returns quantity value. */
    getQuantityValue(): number

    /** Returns related bonus product line items. */
    getRelatedBonusProductLineItems(): dw.util.Collection

    /** Returns reserved flag. */
    isReserved(): boolean

    /** Returns shipment. */
    getShipment(): dw.order.Shipment

    /** Returns shipping line item. */
    getShippingLineItem(): dw.order.ProductShippingLineItem

    /** Returns step quantity. */
    getStepQuantity(): dw.value.Quantity

    /** Returns step quantity value. */
    getStepQuantityValue(): number

    /** Identifies if this is a bonus product line item. */
    isBonusProductLineItem(): boolean

    /** Identifies if this is a bundled product line item. */
    isBundledProductLineItem(): boolean

    /** Identifies if this is a catalog product. */
    isCatalogProduct(): boolean

    /** Identifies if this is a gift. */
    isGift(): boolean

    /** Identifies if this is an option product line item. */
    isOptionProductLineItem(): boolean

    /** Removes the specified price adjustment. */
    removePriceAdjustment(priceAdjustmentLineItem: dw.order.PriceAdjustment): void

    /** Removes the dependent shipping line item. */
    removeShippingLineItem(): void

    /** Replaces the current product with a new product. */
    replaceProduct(newProduct: dw.catalog.Product): void

    /** Sets category. */
    setCategory(category: dw.catalog.Category): void

    /** Sets category ID. */
    setCategoryID(categoryID: string): void

    /** Sets external line item status. */
    setExternalLineItemStatus(status: string): void

    /** Sets external line item text. */
    setExternalLineItemText(text: string): void

    /** Controls if this line item is a gift. */
    setGift(isGift: boolean): void

    /** Sets gift message. */
    setGiftMessage(message: string): void

    /** Sets manufacturer name. */
    setManufacturerName(name: string): void

    /** Sets manufacturer SKU. */
    setManufacturerSKU(sku: string): void

    /** Sets min order quantity value. */
    setMinOrderQuantityValue(quantityValue: number): void

    /** Sets position. */
    setPosition(aValue: number): void

    /** Sets price value. */
    setPriceValue(value: number): void

    /** Sets product inventory list. */
    setProductInventoryList(productInventoryList: dw.catalog.ProductInventoryList): void

    /** Sets product inventory list ID. */
    setProductInventoryListID(productInventoryListID: string): void

    /** Sets product name. */
    setProductName(aValue: string): void

    /** Updates the quantity value. */
    setQuantityValue(quantityValue: number): void

    /** Associates product line item with specified shipment. */
    setShipment(shipment: dw.order.Shipment): void

    /** Sets step quantity value. */
    setStepQuantityValue(quantityValue: number): void

    /** Updates option price. */
    updateOptionPrice(): void

    /** Updates option value. */
    updateOptionValue(optionValue: dw.catalog.ProductOptionValue): void

    /** Updates price (deprecated note in source). */
    updatePrice(price: dw.value.Money): void

    /** Updates quantity and returns new quantity value. */
    updateQuantity(quantityValue: number): number
}
```
