# dw.order.LineItemCtnr

## Overview
Container for line items (product, coupon, gift certificate, shipping) with shipments, payment instruments and price calculations (net, tax, gross, adjusted).

## Description
A container for line items (ProductLineItems, CouponLineItems, GiftCertificateLineItems) with shipments, shipping adjustments (promotions), and payment instruments. Provides methods for creating line items/adjustments and accessing price values. Net-based methods show amounts before tax; tax-based methods return tax amounts; gross-based methods show amounts after tax. Adjusted-based methods return values after promotions. Total-based methods (getTotalNetPrice, getTotalTax, getTotalGrossPrice) aggregate all items including order-level promotions. Note: merchandise methods exclude gift certificates.

## All Known Subclasses
Basket, Order

```ts
declare class LineItemCtnr extends dw.object.ExtensibleObject {
    /** constant for Business Type B2B */
    static BUSINESS_TYPE_B2B: 2

    /** constant for Business Type B2C */
    static BUSINESS_TYPE_B2C: 1

    /** constant for Channel Type CallCenter */
    static CHANNEL_TYPE_CALLCENTER: 2

    /** constant for Channel Type Customer Service Center */
    static CHANNEL_TYPE_CUSTOMERSERVICECENTER: 11

    /** constant for Channel Type DSS */
    static CHANNEL_TYPE_DSS: 4

    /** constant for Channel Type Facebook Ads */
    static CHANNEL_TYPE_FACEBOOKADS: 8

    /** constant for Channel Type Google */
    static CHANNEL_TYPE_GOOGLE: 13

    /** constant for Channel Type Instagram Commerce */
    static CHANNEL_TYPE_INSTAGRAMCOMMERCE: 12

    /** constant for Channel Type Marketplace */
    static CHANNEL_TYPE_MARKETPLACE: 3

    /** constant for Channel Type Online Reservation */
    static CHANNEL_TYPE_ONLINERESERVATION: 10

    /** constant for Channel Type Pinterest */
    static CHANNEL_TYPE_PINTEREST: 6

    /** constant for Channel Type Snapchat */
    static CHANNEL_TYPE_SNAPCHAT: 15

    /** constant for Channel Type Store */
    static CHANNEL_TYPE_STORE: 5

    /** constant for Channel Type Storefront */
    static CHANNEL_TYPE_STOREFRONT: 1

    /** constant for Channel Type Subscriptions */
    static CHANNEL_TYPE_SUBSCRIPTIONS: 9

    /** constant for Channel Type TikTok */
    static CHANNEL_TYPE_TIKTOK: 14

    /** constant for Channel Type Twitter */
    static CHANNEL_TYPE_TWITTER: 7

    /** constant for Channel Type WhatsApp */
    static CHANNEL_TYPE_WHATSAPP: 16

    /** constant for Channel Type YouTube */
    static CHANNEL_TYPE_YOUTUBE: 17

    /** Adjusted total gross price (including tax) after product-level and order-level adjustments. */
    readonly adjustedMerchandizeTotalGrossPrice: dw.value.Money

    /** Adjusted total net price (excluding tax) after product-level and order-level adjustments. */
    readonly adjustedMerchandizeTotalNetPrice: dw.value.Money

    /** Adjusted merchandize total (net or gross based on container mode) including product-level and order-level adjustments. */
    readonly adjustedMerchandizeTotalPrice: dw.value.Money

    /** Adjusted merchandize total tax after product-level and order-level adjustments. */
    readonly adjustedMerchandizeTotalTax: dw.value.Money

    /** Adjusted sum of all shipping line items including tax after shipping adjustments. */
    readonly adjustedShippingTotalGrossPrice: dw.value.Money

    /** Adjusted sum of all shipping line items excluding tax after shipping adjustments. */
    readonly adjustedShippingTotalNetPrice: dw.value.Money

    /** Adjusted shipping total (net or gross based on container mode) after shipping adjustments. */
    readonly adjustedShippingTotalPrice: dw.value.Money

    /** Tax of all shipping line items after shipping adjustments. */
    readonly adjustedShippingTotalTax: dw.value.Money

    /**
     * All gift certificate line items of the container.
     * @deprecated Use getGiftCertificateLineItems() instead.
     */
    readonly allGiftCertificateLineItems: dw.util.Collection

    /** All product, shipping, price adjustment, and gift certificate line items. */
    readonly allLineItems: dw.util.Collection

    /** All product line items (dependent and independent) including option, bundled, and bonus items. */
    readonly allProductLineItems: dw.util.Collection

    /** Hash map of all products to total quantities (includes option, excludes bundled). */
    readonly allProductQuantities: dw.util.HashMap

    /** Collection of all shipping price adjustments applied in the container. */
    readonly allShippingPriceAdjustments: dw.util.Collection

    /** Billing address for the container (null if not created yet). */
    readonly billingAddress: dw.order.OrderAddress

    /** Unsorted collection of bonus discount line items. */
    readonly bonusDiscountLineItems: dw.util.Collection

    /** Collection of product line items that are bonus items. */
    readonly bonusLineItems: dw.util.Collection

    /** Business type (BUSINESS_TYPE_B2C or BUSINESS_TYPE_B2B). */
    readonly businessType: dw.value.EnumValue

    /** Channel type (storefront, call center, marketplace, etc.). */
    readonly channelType: dw.value.EnumValue

    /** Sorted collection of coupon line items (ordered by addition). */
    readonly couponLineItems: dw.util.Collection

    /** Currency code (3-character mnemonic like 'USD' or 'EUR'). */
    readonly currencyCode: string

    /** Customer associated with this container. */
    readonly customer: dw.customer.Customer

    /** Email of the customer associated with this container. */
    customerEmail: string

    /** Name of the customer associated with this container. */
    customerName: string

    /** Customer number of the customer associated with this container. */
    readonly customerNo: string

    /** Default shipment (id "me") that cannot be removed. */
    readonly defaultShipment: dw.order.Shipment

    /** ETag hash representing overall container state including associated objects. */
    readonly etag: string

    /** Whether the container is calculated based on external tax tables. */
    readonly externallyTaxed: boolean

    /** All gift certificate line items of the container. */
    readonly giftCertificateLineItems: dw.util.Collection

    /** Unsorted collection of PaymentInstruments representing GiftCertificates. */
    readonly giftCertificatePaymentInstruments: dw.util.Collection

    /** Total gross price of all gift certificates (usually equal to net price). */
    readonly giftCertificateTotalGrossPrice: dw.value.Money

    /** Total net price of all gift certificates (usually equal to gross price). */
    readonly giftCertificateTotalNetPrice: dw.value.Money

    /** Gift certificate total price (net or gross based on container mode). */
    readonly giftCertificateTotalPrice: dw.value.Money

    /** Total tax of all gift certificates (usually 0.0). */
    readonly giftCertificateTotalTax: dw.value.Money

    /** Merchandize total gross price (including tax) before services/adjustments. */
    readonly merchandizeTotalGrossPrice: dw.value.Money

    /** Merchandize total net price (excluding tax) before services/adjustments. */
    readonly merchandizeTotalNetPrice: dw.value.Money

    /** Merchandize total price (net or gross based on container mode). */
    readonly merchandizeTotalPrice: dw.value.Money

    /** Merchandize total tax before services/adjustments. */
    readonly merchandizeTotalTax: dw.value.Money

    /** List of notes ordered by creation time (oldest to newest). */
    readonly notes: dw.util.List

    /**
     * Single payment instrument accessor (deprecated).
     * @deprecated Use getPaymentInstruments() or getGiftCertificatePaymentInstruments() instead.
     */
    readonly paymentInstrument: dw.order.OrderPaymentInstrument

    /** Unsorted collection of payment instruments. */
    readonly paymentInstruments: dw.util.Collection

    /** Collection of price adjustments applied to totals (sorted by application order). */
    readonly priceAdjustments: dw.util.Collection

    /** Product line items not dependent on other items (includes bonus, excludes option/bundled). */
    readonly productLineItems: dw.util.Collection

    /** Hash map of products to total quantities (excludes bundled/option/bonus where appropriate). */
    readonly productQuantities: dw.util.HashMap

    /** Total quantity of all product line items (excludes bundled and option items). */
    readonly productQuantityTotal: number

    /** All shipments (first is default "me", others sorted by ID). */
    readonly shipments: dw.util.Collection

    /** Collection of shipping price adjustments applied to the container. */
    readonly shippingPriceAdjustments: dw.util.Collection

    /** Sum of all shipping line items including tax before adjustments. */
    readonly shippingTotalGrossPrice: dw.value.Money

    /** Sum of all shipping line items excluding tax before adjustments. */
    readonly shippingTotalNetPrice: dw.value.Money

    /** Shipping total price (net or gross based on container mode). */
    readonly shippingTotalPrice: dw.value.Money

    /** Tax of all shipping line items before adjustments. */
    readonly shippingTotalTax: dw.value.Money

    /** Whether tax was rounded at group level. */
    readonly taxRoundedAtGroup: boolean

    /** SortedMap with Decimal tax rates as keys and Money total tax as values. */
    readonly taxTotalsPerTaxRate: dw.util.SortedMap

    /** Grand total gross price (including tax). */
    readonly totalGrossPrice: dw.value.Money

    /** Grand total net price (excluding tax). */
    readonly totalNetPrice: dw.value.Money

    /** Grand total tax. */
    readonly totalTax: dw.value.Money

    /**
     * Adds a note to the object.
     * @param subject Subject of the note.
     * @param text Text of the note (max 4000 characters).
     */
    addNote(subject: string, text: string): dw.object.Note

    /**
     * Creates a billing address for the LineItemCtnr (replaces existing if present).
     */
    createBillingAddress(): dw.order.OrderAddress

    /**
     * Creates a bonus product line item based on a BonusDiscountLineItem and Product.
     * @param bonusDiscountLineItem Line item representing applied BonusChoiceDiscount.
     * @param product Product to add (must be bonus product of the discount).
     * @param optionModel ProductOptionModel or null.
     * @param shipment Shipment to add bonus product to (null = default shipment).
     */
    createBonusProductLineItem(bonusDiscountLineItem: dw.order.BonusDiscountLineItem, product: dw.catalog.Product, optionModel: dw.catalog.ProductOptionModel, shipment: dw.order.Shipment): dw.order.ProductLineItem

    /**
     * Creates a CouponLineItem for the coupon code (campaign-based or custom).
     * @param couponCode Coupon code string.
     * @param campaignBased Whether based on B2C Commerce campaign system.
     */
    createCouponLineItem(couponCode: string, campaignBased?: boolean): dw.order.CouponLineItem

    /**
     * Creates a gift certificate line item.
     * @param amount Monetary amount for the gift certificate.
     * @param recipientEmail Recipient's email address.
     */
    createGiftCertificateLineItem(amount: number, recipientEmail: string): dw.order.GiftCertificateLineItem

    /**
     * Creates an OrderPaymentInstrument representing a Gift Certificate.
     * @param giftCertificateCode Gift certificate code.
     * @param amount Amount as Money.
     */
    createGiftCertificatePaymentInstrument(giftCertificateCode: string, amount: dw.value.Money): dw.order.OrderPaymentInstrument

    /**
     * Creates a payment instrument for the payment method and amount.
     * @param paymentMethodId Payment method identifier.
     * @param amount Amount as Money.
     */
    createPaymentInstrument(paymentMethodId: string, amount: dw.value.Money): dw.order.OrderPaymentInstrument

    /**
     * Creates a payment instrument from a wallet (customer payment instrument).
     * @param walletPaymentInstrument CustomerPaymentInstrument to use.
     * @param amount Amount as Money.
     */
    createPaymentInstrumentFromWallet(walletPaymentInstrument: dw.customer.CustomerPaymentInstrument, amount: dw.value.Money): dw.order.OrderPaymentInstrument

    /**
     * Creates an order-level price adjustment for a promotion ID (must not be an actual B2C Commerce promotion ID).
     * @param promotionID Synthetic promotion identifier.
     * @param discount Optional Discount instance.
     */
    createPriceAdjustment(promotionID: string, discount?: dw.campaign.Discount): dw.order.PriceAdjustment

    /**
     * Creates product line item (multiple overloads supported).
     * @deprecated Use overloads without Quantity parameter.
     */
    createProductLineItem(productID: string, quantity: dw.value.Quantity, shipment: dw.order.Shipment): dw.order.ProductLineItem
    createProductLineItem(productID: string, shipment: dw.order.Shipment): dw.order.ProductLineItem
    createProductLineItem(productListItem: dw.customer.ProductListItem, shipment: dw.order.Shipment): dw.order.ProductLineItem
    createProductLineItem(product: dw.catalog.Product, optionModel: dw.catalog.ProductOptionModel, shipment: dw.order.Shipment): dw.order.ProductLineItem

    /**
     * Creates a standard Shipment for the container.
     * @param id Shipment id.
     */
    createShipment(id: string): dw.order.Shipment

    /**
     * Creates a shipping PriceAdjustment for a promotion id.
     * @param promotionID Promotion id to associate.
     */
    createShippingPriceAdjustment(promotionID: string): dw.order.PriceAdjustment

    /**
     * Returns adjusted merchandizing total gross price (including tax).
     */
    getAdjustedMerchandizeTotalGrossPrice(): dw.value.Money

    /**
     * Returns adjusted merchandizing total net price (excluding tax).
     */
    getAdjustedMerchandizeTotalNetPrice(): dw.value.Money

    /**
     * Returns adjusted merchandize total price including order-level adjustments if requested.
     * @param applyOrderLevelAdjustments Whether to apply order-level adjustments (default true).
     */
    getAdjustedMerchandizeTotalPrice(applyOrderLevelAdjustments?: boolean): dw.value.Money

    /**
     * Returns adjusted merchandize total tax.
     */
    getAdjustedMerchandizeTotalTax(): dw.value.Money

    /**
     * Returns adjusted shipping total gross price (including tax) after adjustments.
     */
    getAdjustedShippingTotalGrossPrice(): dw.value.Money

    /**
     * Returns adjusted shipping total net price (excluding tax) after adjustments.
     */
    getAdjustedShippingTotalNetPrice(): dw.value.Money

    /**
     * Returns adjusted shipping total price (net or gross based on container mode).
     */
    getAdjustedShippingTotalPrice(): dw.value.Money

    /**
     * Returns adjusted shipping total tax.
     */
    getAdjustedShippingTotalTax(): dw.value.Money

    /**
     * Returns all gift certificate line items of the container.
     * @deprecated Use getGiftCertificateLineItems() instead.
     */
    getAllGiftCertificateLineItems(): dw.util.Collection

    /**
     * Returns all line items (product, shipping, price adjustment, gift certificate).
     */
    getAllLineItems(): dw.util.Collection

    /**
     * Returns all product line items (dependent and independent) or filtered by productID.
     * @param productID Optional product ID to filter by.
     */
    getAllProductLineItems(productID?: string): dw.util.Collection

    /**
     * Returns hash map of all products to total quantities.
     */
    getAllProductQuantities(): dw.util.HashMap

    /**
     * Returns collection of all shipping price adjustments in the container.
     */
    getAllShippingPriceAdjustments(): dw.util.Collection

    /**
     * Returns billing address (null if not created yet).
     */
    getBillingAddress(): dw.order.OrderAddress

    /**
     * Returns unsorted collection of bonus discount line items.
     */
    getBonusDiscountLineItems(): dw.util.Collection

    /**
     * Returns collection of product line items that are bonus items.
     */
    getBonusLineItems(): dw.util.Collection

    /**
     * Returns business type (BUSINESS_TYPE_B2C or BUSINESS_TYPE_B2B).
     */
    getBusinessType(): dw.value.EnumValue

    /**
     * Returns channel type (storefront, call center, marketplace, etc.).
     */
    getChannelType(): dw.value.EnumValue

    /**
     * Returns coupon line item for the specified coupon code.
     * @param couponCode Coupon code to find.
     */
    getCouponLineItem(couponCode: string): dw.order.CouponLineItem

    /**
     * Returns sorted collection of coupon line items.
     */
    getCouponLineItems(): dw.util.Collection

    /**
     * Returns currency code for this container.
     */
    getCurrencyCode(): string

    /**
     * Returns customer associated with this container.
     */
    getCustomer(): dw.customer.Customer

    /**
     * Returns customer email.
     */
    getCustomerEmail(): string

    /**
     * Returns customer name.
     */
    getCustomerName(): string

    /**
     * Returns customer number.
     */
    getCustomerNo(): string

    /**
     * Returns default shipment (id "me").
     */
    getDefaultShipment(): dw.order.Shipment

    /**
     * Returns ETag hash representing container state.
     */
    getEtag(): string

    /**
     * Returns all gift certificate line items or filtered by giftCertificateId.
     * @param giftCertificateId Optional gift certificate ID to filter by.
     */
    getGiftCertificateLineItems(giftCertificateId?: string): dw.util.Collection

    /**
     * Returns PaymentInstruments representing GiftCertificates or filtered by code.
     * @param giftCertificateCode Optional gift certificate code to filter by.
     */
    getGiftCertificatePaymentInstruments(giftCertificateCode?: string): dw.util.Collection

    /**
     * Returns total gross price of all gift certificates.
     */
    getGiftCertificateTotalGrossPrice(): dw.value.Money

    /**
     * Returns total net price of all gift certificates.
     */
    getGiftCertificateTotalNetPrice(): dw.value.Money

    /**
     * Returns gift certificate total price (net or gross based on container mode).
     */
    getGiftCertificateTotalPrice(): dw.value.Money

    /**
     * Returns total tax of all gift certificates.
     */
    getGiftCertificateTotalTax(): dw.value.Money

    /**
     * Returns merchandize total gross price (including tax) before services/adjustments.
     */
    getMerchandizeTotalGrossPrice(): dw.value.Money

    /**
     * Returns merchandize total net price (excluding tax) before services/adjustments.
     */
    getMerchandizeTotalNetPrice(): dw.value.Money

    /**
     * Returns merchandize total price (net or gross based on container mode).
     */
    getMerchandizeTotalPrice(): dw.value.Money

    /**
     * Returns merchandize total tax before services/adjustments.
     */
    getMerchandizeTotalTax(): dw.value.Money

    /**
     * Returns list of notes ordered by creation time (oldest to newest).
     */
    getNotes(): dw.util.List

    /**
     * Returns payment instrument or null.
     * @deprecated Use getPaymentInstruments() or getGiftCertificatePaymentInstruments() instead.
     */
    getPaymentInstrument(): dw.order.OrderPaymentInstrument

    /**
     * Returns unsorted collection of payment instruments or filtered by payment method ID.
     * @param paymentMethodID Optional payment method ID to filter by.
     */
    getPaymentInstruments(paymentMethodID?: string): dw.util.Collection

    /**
     * Returns price adjustment for the specified promotion ID.
     * @param promotionID Promotion ID to find.
     */
    getPriceAdjustmentByPromotionID(promotionID: string): dw.order.PriceAdjustment

    /**
     * Returns collection of price adjustments applied to totals.
     */
    getPriceAdjustments(): dw.util.Collection

    /**
     * Returns product line items not dependent on other items or filtered by productID.
     * @param productID Optional product ID to filter by.
     */
    getProductLineItems(productID?: string): dw.util.Collection

    /**
     * Returns hash map of products to total quantities.
     * @param includeBonusProducts Whether to include bonus products (default false).
     */
    getProductQuantities(includeBonusProducts?: boolean): dw.util.HashMap

    /**
     * Returns total quantity of all product line items.
     */
    getProductQuantityTotal(): number

    /**
     * Returns shipment for the specified ID or null if not found.
     * @param id Shipment ID.
     */
    getShipment(id: string): dw.order.Shipment

    /**
     * Returns all shipments (first is default "me", others sorted by ID).
     */
    getShipments(): dw.util.Collection

    /**
     * Returns shipping price adjustment for the specified promotion ID.
     * @param promotionID Promotion ID to find.
     */
    getShippingPriceAdjustmentByPromotionID(promotionID: string): dw.order.PriceAdjustment

    /**
     * Returns collection of shipping price adjustments applied to the container.
     */
    getShippingPriceAdjustments(): dw.util.Collection

    /**
     * Returns sum of all shipping line items including tax before adjustments.
     */
    getShippingTotalGrossPrice(): dw.value.Money

    /**
     * Returns sum of all shipping line items excluding tax before adjustments.
     */
    getShippingTotalNetPrice(): dw.value.Money

    /**
     * Returns shipping total price (net or gross based on container mode).
     */
    getShippingTotalPrice(): dw.value.Money

    /**
     * Returns tax of all shipping line items before adjustments.
     */
    getShippingTotalTax(): dw.value.Money

    /**
     * Returns SortedMap with Decimal tax rates as keys and Money total tax as values.
     */
    getTaxTotalsPerTaxRate(): dw.util.SortedMap

    /**
     * Returns grand total gross price (including tax).
     */
    getTotalGrossPrice(): dw.value.Money

    /**
     * Returns grand total net price (excluding tax).
     */
    getTotalNetPrice(): dw.value.Money

    /**
     * Returns grand total tax.
     */
    getTotalTax(): dw.value.Money

    /**
     * Checks whether the container is calculated based on external tax tables.
     */
    isExternallyTaxed(): boolean

    /**
     * Checks if tax was rounded at group level.
     */
    isTaxRoundedAtGroup(): boolean

    /**
     * Removes all payment instruments from this container.
     */
    removeAllPaymentInstruments(): void

    /**
     * Removes the specified bonus discount line item.
     * @param bonusDiscountLineItem Bonus discount line item to remove.
     */
    removeBonusDiscountLineItem(bonusDiscountLineItem: dw.order.BonusDiscountLineItem): void

    /**
     * Removes the specified coupon line item.
     * @param couponLineItem Coupon line item to remove.
     */
    removeCouponLineItem(couponLineItem: dw.order.CouponLineItem): void

    /**
     * Removes the specified gift certificate line item.
     * @param giftCertificateLineItem Gift certificate line item to remove.
     */
    removeGiftCertificateLineItem(giftCertificateLineItem: dw.order.GiftCertificateLineItem): void

    /**
     * Removes a note from this container and deletes it.
     * @param note Note to remove.
     */
    removeNote(note: dw.object.Note): void

    /**
     * Removes the specified payment instrument and deletes it.
     * @param pi Payment instrument to remove.
     */
    removePaymentInstrument(pi: dw.order.PaymentInstrument): void

    /**
     * Removes the specified price adjustment line item.
     * @param priceAdjustment Price adjustment to remove.
     */
    removePriceAdjustment(priceAdjustment: dw.order.PriceAdjustment): void

    /**
     * Removes the specified product line item.
     * @param productLineItem Product line item to remove.
     */
    removeProductLineItem(productLineItem: dw.order.ProductLineItem): void

    /**
     * Removes the specified shipment and all associated line items (throws exception if default shipment).
     * @param shipment Shipment to remove.
     */
    removeShipment(shipment: dw.order.Shipment): void

    /**
     * Removes the specified shipping price adjustment line item.
     * @param priceAdjustment Shipping price adjustment to remove.
     */
    removeShippingPriceAdjustment(priceAdjustment: dw.order.PriceAdjustment): void

    /**
     * Sets the customer email address.
     * @param aValue New email value.
     */
    setCustomerEmail(aValue: string): void

    /**
     * Sets the customer name.
     * @param aValue New name value.
     */
    setCustomerName(aValue: string): void

    /**
     * Calculates tax for shipping and order-level merchandise price adjustments.
     */
    updateOrderLevelPriceAdjustmentTax(): void

    /**
     * Recalculates the totals of the line item container.
     */
    updateTotals(): void

    /**
     * Verifies whether manual price adjustments exceed limits for current user and site.
     */
    verifyPriceAdjustmentLimits(): dw.system.Status
}
```
