# dw.value.Money

## Overview
Represents currency amounts in Commerce Cloud with a NOT_AVAILABLE sentinel and rich arithmetic helpers.

## Description
Stores the numeric value, ISO 4217 currency mnemonic, and a Decimal representation that is rounded using java.math.RoundingMode.HALF_UP; operations require comparable currencies and return new Money instances, while NOT_AVAILABLE signals the absence of a price and propagates through the helpers.

```ts
declare class Money  {
    /**
     * Sentinel Money instance representing the absence of a value.
     */
    static NOT_AVAILABLE: Money

    /**
     * @readonly
     * Indicates whether the instance contains both a value and a currency.
     */
    readonly available: boolean

    /**
     * @readonly
     * The ISO 4217 currency mnemonic ("USD", "EUR", or "N/A" when not available).
     */
    readonly currencyCode: string

    /**
     * @readonly
     * Decimal representation of the amount, or null when the instance is NOT_AVAILABLE.
     */
    readonly decimalValue: Decimal | null

    /**
     * @readonly
     * Numeric value of this money instance.
     */
    readonly value: number

    /**
     * @readonly
     * Numeric value or null when the instance is NOT_AVAILABLE.
     */
    readonly valueOrNull: number | null

    /**
     * Adds another Money of the same currency, resulting in NOT_AVAILABLE when either side is unavailable.
     */
    add(value: Money): Money

    /**
     * Adds a true percent value (10 means 10%) to this instance.
     */
    addPercent(percent: number): Money

    /**
     * Adds a rate such as a tax rate (0.05) to this instance.
     */
    addRate(value: number): Money

    /**
     * Compares two Money objects and throws if the currencies differ; treats NOT_AVAILABLE as 0.
     */
    compareTo(other: Money): number

    /**
     * Divides this Money by the provided divisor and returns a new instance.
     */
    divide(divisor: number): Money

    /**
     * Returns true when another object contains the same currency and value.
     */
    equals(other: Object): boolean

    /**
     * Returns the ISO 4217 currency mnemonic of this money.
     */
    getCurrencyCode(): string

    /**
     * Returns the Decimal representation or null when not available.
     */
    getDecimalValue(): Decimal | null

    /**
     * Returns the numeric value of this instance.
     */
    getValue(): number

    /**
     * Returns the numeric value or null when the instance is NOT_AVAILABLE.
     */
    getValueOrNull(): number | null

    /**
     * Computes a numeric hash code for the money value.
     */
    hashCode(): number

    /**
     * Returns true when the instance contains a value and currency.
     */
    isAvailable(): boolean

    /**
     * Returns true when the passed Money shares the same currency as this instance.
     */
    isOfSameCurrency(value: Money): boolean

    /**
     * Multiplies the money value by a scalar.
     */
    multiply(factor: number): Money

    /**
     * Multiplies the money value by a Quantity instance.
     */
    multiply(quantity: Quantity): Money

    /**
     * Returns a new Money instance with the same currency and the provided Decimal value.
     */
    newMoney(value: Decimal): Money

    /**
     * Returns how much percent less this value is compared to another Money or null when unavailable.
     */
    percentLessThan(value: Money): number | null

    /**
     * Returns what percent this value represents of another Money or null when unavailable.
     */
    percentOf(value: Money): number | null

    /**
     * Prorates the provided values across the specified discount.
     */
    static prorate(dist: Money, ...values: Money[]): Money[]

    /**
     * Subtracts another Money and returns the difference, propagating NOT_AVAILABLE when needed.
     */
    subtract(value: Money): Money

    /**
     * Subtracts a true percent value (10 means 10%) from this instance.
     */
    subtractPercent(percent: number): Money

    /**
     * Subtracts a rate such as a tax rate (0.05) from this instance.
     */
    subtractRate(value: number): Money

    /**
     * Returns the localized formatted string for this Money.
     */
    toFormattedString(): string

    /**
     * Returns the numeric value as a string using the platform default locale.
     */
    toNumberString(): string

    /**
     * Returns a general string representation of the Money instance.
     */
    toString(): string

    /**
     * Returns the primitive value portion according to the ECMA spec.
     */
    valueOf(): object
}
```