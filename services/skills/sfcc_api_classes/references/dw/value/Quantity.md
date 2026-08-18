# dw.value.Quantity

## Overview
Represents the quantity of an item.

## Description
Represents the quantity of an item with a numeric value and unit of measure (e.g., 'inches', 'pounds').

```ts
declare class Quantity  {
	/**
	 * Identifies if the instance contains settings for value and unit.
	 */
	readonly available: boolean

	/**
	 * The quantity as Decimal, null is returned when the quantity is not available.
	 */
	readonly decimalValue: Decimal | null

	/**
	 * The unit of measure for the quantity (e.g., 'inches', 'pounds').
	 */
	readonly unit: string

	/**
	 * The quantity value.
	 */
	readonly value: number

	/**
	 * Creates a new quantity instance with the specified value and unit.
	 * @param value - The actual quantity, must not be null
	 * @param unit - The unit identifier for the quantity, must not be null
	 */
	constructor(value: number, unit: string)

	/**
	 * Add Quantity object to the current object. Only objects representing the same unit can be added.
	 * @param value - Quantity object
	 * @returns Quantity object representing the sum of the operands
	 */
	add(value: Quantity): Quantity

	/**
	 * Compares two Quantity values. An exception is thrown if the two Quantities values are of different unit. If one of the Quantity values represents the N/A value it is treated as 0.0.
	 * @param other - The other quantity to compare
	 * @returns The comparison result
	 */
	compareTo(other: Quantity): number

	/**
	 * Divide Quantity object by specified divisor.
	 * @param divisor - Divisor
	 * @returns Quantity object representing division result
	 */
	divide(divisor: number): Quantity

	/**
	 * Compares two decimal values whether they are equivalent.
	 * @param other - The object to compare against this quantity instance
	 * @returns True if equal, false otherwise
	 */
	equals(other: Object): boolean

	/**
	 * Returns the quantity as Decimal, null is returned when the quantity is not available.
	 * @returns The quantity as Decimal
	 */
	getDecimalValue(): Decimal | null

	/**
	 * Returns the unit of measure for the quantity.
	 * @returns The unit value
	 */
	getUnit(): string

	/**
	 * Returns the quantity value.
	 * @returns The quantity value
	 */
	getValue(): number

	/**
	 * Calculates the hash code for a decimal.
	 * @returns The hash code
	 */
	hashCode(): number

	/**
	 * Identifies if the instance contains settings for value and unit.
	 * @returns True if the instance is initialized with value and unit, false if the state is 'not available'
	 */
	isAvailable(): boolean

	/**
	 * Identifies if two Quantities have the same unit.
	 * @param value - The second quantity for the comparison
	 * @returns True if both quantities have the same unit, false otherwise
	 */
	isOfSameUnit(value: Quantity): boolean

	/**
	 * Multiply Quantity object by specified factor.
	 * @param factor - Multiplication factor
	 * @returns Quantity object representing multiplication result
	 */
	multiply(factor: number): Quantity

	/**
	 * Method returns a new instance of Quantity with the same unit but different value. An N/A instance is returned if value is null.
	 * @param value - Value as a decimal
	 * @returns New Quantity instance with same unit
	 */
	newQuantity(value: Decimal): Quantity

	/**
	 * Rounds the Quantity value to the number of specified decimal digits.
	 * @param precision - Number of decimal digits after the decimal point
	 * @returns The new rounded Quantity value
	 */
	round(precision: number): Quantity

	/**
	 * Subtract Quantity object from the current object. Only objects representing the same unit can be subtracted.
	 * @param value - Quantity object to subtract
	 * @returns Quantity object representing the result of subtraction
	 */
	subtract(value: Quantity): Quantity

	/**
	 * Returns a string representation of this quantity object.
	 * @returns A string representation of this quantity object
	 */
	toString(): string

	/**
	 * According to the ECMA spec returns the "natural" primitive value. Here the value portion of the Quantity is returned.
	 * @returns The value portion of the Quantity
	 */
	valueOf(): Object
}
```
