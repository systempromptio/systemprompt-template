# dw.catalog.ProductActiveData

## Overview
Holds recent activity and analytics metrics for a product on a specific site (views, sales, revenue, margins, conversion, etc.).

## Description
Provides read-only metrics aggregated over different time windows (day, week, month, year) used for analytics and merchandising decisions.

```ts
declare class ProductActiveData  {
    /** The date the product became available on the site. */
    readonly availableDate: Date

    /** Returns average gross margin percent for the last day (or null). */
    getAvgGrossMarginPercentDay(): number | null
    /** Returns average gross margin percent for the last 7 days (or null). */
    getAvgGrossMarginPercentWeek(): number | null
    /** Returns average gross margin percent for the last 30 days (or null). */
    getAvgGrossMarginPercentMonth(): number | null
    /** Returns average gross margin percent for the last 365 days (or null). */
    getAvgGrossMarginPercentYear(): number | null

    /** Returns average gross margin value for the last day (or null). */
    getAvgGrossMarginValueDay(): number | null
    getAvgGrossMarginValueWeek(): number | null
    getAvgGrossMarginValueMonth(): number | null
    getAvgGrossMarginValueYear(): number | null

    /** Returns average sales price for the last day/week/month/year (or null). */
    getAvgSalesPriceDay(): number | null
    getAvgSalesPriceWeek(): number | null
    getAvgSalesPriceMonth(): number | null
    getAvgSalesPriceYear(): number | null

    /** Returns conversion rate for day/week/month/year (or null). */
    getConversionDay(): number | null
    getConversionWeek(): number | null
    getConversionMonth(): number | null
    getConversionYear(): number | null

    /** Returns cost price for the product on the site (or null). */
    getCostPrice(): number | null

    /** Number of days product has been available on the site. */
    getDaysAvailable(): number

    /** Impressions, orders, revenue, views, units, returns, sales velocity for day/week/month/year. */
    getImpressionsDay(): number | null
    getImpressionsWeek(): number | null
    getImpressionsMonth(): number | null
    getImpressionsYear(): number | null

    getOrdersDay(): number | null
    getOrdersWeek(): number | null
    getOrdersMonth(): number | null
    getOrdersYear(): number | null

    getReturnRate(): number | null

    getRevenueDay(): number | null
    getRevenueWeek(): number | null
    getRevenueMonth(): number | null
    getRevenueYear(): number | null

    getSalesVelocityDay(): number | null
    getSalesVelocityWeek(): number | null
    getSalesVelocityMonth(): number | null
    getSalesVelocityYear(): number | null

    getUnitsDay(): number | null
    getUnitsWeek(): number | null
    getUnitsMonth(): number | null
    getUnitsYear(): number | null

    getViewsDay(): number | null
    getViewsWeek(): number | null
    getViewsMonth(): number | null
    getViewsYear(): number | null
}
```

## All Known Subclasses
None


