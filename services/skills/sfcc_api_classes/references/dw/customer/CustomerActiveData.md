# dw.customer.CustomerActiveData

## Overview
Active/customer analytics data: order counts/values, visits, viewed/ordered product SKUs and other recent activity metrics.

## Description
Provides read-only metrics and collections describing a customer's recent and lifetime activity. Values may be null if not set or stale. Contains helper getters for each metric.

```ts
declare class CustomerActiveData extends dw.object.PersistentObject {
    readonly avgOrderValue: number | null
    readonly discountValueWithCoupon: number | null
    readonly discountValueWithoutCoupon: number | null
    readonly giftOrders: number | null
    readonly giftUnits: number | null
    readonly lastOrderDate: Date | null
    readonly orders: number | null
    readonly orderValue: number | null
    readonly orderValueMonth: number | null
    readonly productMastersOrdered: string[]
    readonly productsAbandonedMonth: string[]
    readonly productsOrdered: string[]
    readonly productsViewedMonth: string[]
    readonly returns: number | null
    readonly returnValue: number | null
    readonly sourceCodeOrders: number | null
    readonly topCategoriesOrdered: string[]
    readonly visitsMonth: number | null
    readonly visitsWeek: number | null
    readonly visitsYear: number | null

    getAvgOrderValue(): number | null
    getDiscountValueWithCoupon(): number | null
    getDiscountValueWithoutCoupon(): number | null
    getGiftOrders(): number | null
    getGiftUnits(): number | null
    getLastOrderDate(): Date | null
    getOrders(): number | null
    getOrderValue(): number | null
    getOrderValueMonth(): number | null
    getProductMastersOrdered(): string[]
    getProductsAbandonedMonth(): string[]
    getProductsOrdered(): string[]
    getProductsViewedMonth(): string[]
    getReturns(): number | null
    getReturnValue(): number | null
    getSourceCodeOrders(): number | null
    getTopCategoriesOrdered(): string[]
    getVisitsMonth(): number | null
    getVisitsWeek(): number | null
    getVisitsYear(): number | null
}
```
