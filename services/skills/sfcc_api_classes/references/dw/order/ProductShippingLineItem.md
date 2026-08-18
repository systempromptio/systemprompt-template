# dw.order.ProductShippingLineItem

## Overview
Represents a specific line item in a shipment; defines line-item-specific shipping costs.

## Description
Represents a specific line item in a shipment. A ProductShippingLineItem defines lineitem-specific shipping costs.

```ts
declare class ProductShippingLineItem extends dw.order.LineItem {
    /** Reserved deprecated constant. */
    static PRODUCT_SHIPPING_ID: 'PRODUCT_SHIPPING'

    /** The gross price after product-shipping-level adjustments. */
    readonly adjustedGrossPrice: dw.value.Money

    /** The net price after product-shipping-level adjustments. */
    readonly adjustedNetPrice: dw.value.Money

    /** The adjusted price after product-shipping-level adjustments. */
    readonly adjustedPrice: dw.value.Money

    /** The tax after applying adjustments. */
    readonly adjustedTax: dw.value.Money

    /** An iterator of price adjustments applied to this product shipping line item. */
    readonly priceAdjustments: dw.util.Collection

    /** The parent product line item this shipping line item belongs to. */
    readonly productLineItem: dw.order.ProductLineItem

    /** The quantity of the shipping cost. */
    quantity: dw.value.Quantity

    /** The shipment this shipping line item belongs to. */
    readonly shipment: dw.order.Shipment

    /** The 'surcharge' flag. */
    surcharge: boolean

    getAdjustedGrossPrice(): dw.value.Money
    getAdjustedNetPrice(): dw.value.Money
    getAdjustedPrice(): dw.value.Money
    getAdjustedTax(): dw.value.Money
    getPriceAdjustments(): dw.util.Collection
    getProductLineItem(): dw.order.ProductLineItem
    getQuantity(): dw.value.Quantity
    getShipment(): dw.order.Shipment
    isSurcharge(): boolean
    setPriceValue(value: number): void
    setQuantity(quantity: dw.value.Quantity): void
    setSurcharge(flag: boolean): void
}
```
