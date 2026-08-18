# dw.object.PersistentObject

## Overview
Base class for objects that have an identity and can be stored and retrieved (UUID-based).

## Description
Provides creation and modification timestamps and a UUID for persisted entities. Many domain objects inherit from this class.

```ts
declare class PersistentObject  {
    /** The date this object was created. */
    creationDate: Date // (Read Only)

    /** The date this object was last modified. */
    lastModified: Date // (Read Only)

    /** The unique universal identifier (UUID). */
    UUID: string // (Read Only)

    /** Returns the creation date. */
    getCreationDate(): Date

    /** Returns the last modified date. */
    getLastModified(): Date

    /** Returns the UUID. */
    getUUID(): string
}
```

## All Known Subclasses
ABTest, ABTestSegment, ActiveData, Basket, BonusDiscountLineItem, Campaign, Catalog, Category, CategoryAssignment, Content, ContentSearchRefinementDefinition, Coupon, CouponLineItem, CustomerActiveData, CustomerAddress, CustomerGroup, CustomerPaymentInstrument, CustomObject, EncryptedObject, ExtensibleObject, Folder, GiftCertificate, GiftCertificateLineItem, Library, LineItem, LineItemCtnr, Order, OrderAddress, OrderPaymentInstrument, OrganizationPreferences, PaymentCard, PaymentInstrument, PaymentMethod, PaymentProcessor, PaymentTransaction, PriceAdjustment, PriceBook, Product, ProductActiveData, ProductInventoryList, ProductInventoryRecord, ProductLineItem, ProductList, ProductListItem, ProductListItemPurchase, ProductListRegistrant, ProductOption, ProductOptionValue, ProductSearchRefinementDefinition, ProductShippingLineItem, Profile, Promotion, Recommendation, SearchRefinementDefinition, ServiceConfig, ServiceCredential, ServiceProfile, Shipment, ShippingLineItem, ShippingMethod, SitePreferences, SortingOption, SortingRule, SourceCodeGroup, Store, StoreGroup, Variant, VariationGroup
