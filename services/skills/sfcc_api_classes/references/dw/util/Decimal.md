# dw.util.Decimal

## Overview
A helper class for performing precise decimal arithmetic operations and representing decimal numbers with arbitrary length, avoiding floating-point arithmetic errors.

## Description
The Decimal class is a helper class to perform decimal arithmetic in scripts and to represent a decimal number with arbitrary length. The decimal class avoids arithmetic errors, which are typical for calculating with floating numbers, that are based on a binary mantissa.

The class is designed in a way that it can be used very similar to a desktop calculator.

```javascript
var d = new Decimal(10.0);
var result = d.add(2.0).sub(3.0).get();
```

The above code will return 9 as result.

```
Object
  dw.util.Decimal
```

```ts
declare class Decimal  {
	/**
	 * Constructs a new Decimal with the value 0.
	 */
	constructor()

	/**
	 * Constructs a new decimal using the specified Number value.
	 * @param value - the value to use
	 */
	constructor(value: number)

	/**
	 * Constructs a new decimal using the specified BigInt value.
	 * @param value - the value to use
	 */
	constructor(value: BigInt)

	/**
	 * Constructs a new Decimal using the specified string representation of a number.
	 * @param value - the value to use
	 */
	constructor(value: string)

	/**
	 * Returns a new Decimal with the absolute value of this Decimal.
	 * @returns the new Decimal
	 */
	abs(): Decimal

	/**
	 * Adds a Number value to this Decimal and returns the new Decimal.
	 * @param value - the value to add to this decimal
	 * @returns the new decimal with the value added
	 */
	add(value: number): Decimal

	/**
	 * Adds a Decimal value to this Decimal and returns the new Decimal.
	 * @param value - the value to add to this decimal
	 * @returns the new decimal with the value added
	 */
	add(value: Decimal): Decimal

	/**
	 * Adds a percentage value to the current value of the decimal. For example a value of 10 represent 10% or a value of 85 represents 85%.
	 * @param value - the value to add
	 * @returns a new decimal with the added percentage value
	 */
	addPercent(value: number): Decimal

	/**
	 * Adds a percentage value to the current value of the decimal. For example a value of 10 represent 10% or a value of 85 represents 85%.
	 * @param value - the value to add
	 * @returns a new decimal with the added percentage value
	 */
	addPercent(value: Decimal): Decimal

	/**
	 * Divides the specified Number value with this decimal and returns the new decimal. When performing the division, 34 digits precision and a rounding mode of HALF_EVEN is used to prevent quotients with nonterminating decimal expansions.
	 * @param value - the value to use to divide this decimal
	 * @returns the new decimal
	 */
	divide(value: number): Decimal

	/**
	 * Divides the specified Decimal value with this decimal and returns the new decimal. When performing the division, 34 digits precision and a rounding mode of HALF_EVEN is used to prevent quotients with nonterminating decimal expansions.
	 * @param value - the value to use to divide this decimal
	 * @returns the new decimal
	 */
	divide(value: Decimal): Decimal

	/**
	 * Compares two decimal values whether they are equivalent.
	 * @param other - the object to comapre against this decimal
	 * @returns true if the decimals are equivalent, false otherwise
	 */
	equals(other: Object): boolean

	/**
	 * Returns the value of the Decimal as a Number.
	 * @returns the value of the Decimal
	 */
	get(): number

	/**
	 * Calculates the hash code for this decimal.
	 * @returns the hash code
	 */
	hashCode(): number

	/**
	 * Multiples the specified Number value with this Decimal and returns the new Decimal.
	 * @param value - the value to multiply with this decimal
	 * @returns the new decimal
	 */
	multiply(value: number): Decimal

	/**
	 * Multiples the specified Decimal value with this Decimal and returns the new Decimal.
	 * @param value - the value to multiply with this decimal
	 * @returns the new decimal
	 */
	multiply(value: Decimal): Decimal

	/**
	 * Returns a new Decimal with the negated value of this Decimal.
	 * @returns the new Decimal
	 */
	negate(): Decimal

	/**
	 * Rounds the current value of the decimal using the specified number of decimals. The parameter specifies the number of digest after the decimal point.
	 * @param decimals - the number of decimals to use
	 * @returns the decimal that has been rounded
	 */
	round(decimals: number): Decimal

	/**
	 * Subtracts the specified Number value from this Decimal and returns the new Decimal.
	 * @param value - the value to add to this decimal
	 * @returns the new decimal with the value subtraced
	 */
	subtract(value: number): Decimal

	/**
	 * Subtracts the specified Decimal value from this Decimal and returns the new Decimal.
	 * @param value - the value to add to this decimal
	 * @returns the new decimal with the value subtraced
	 */
	subtract(value: Decimal): Decimal

	/**
	 * Subtracts a percentage value from the current value of the decimal. For example a value of 10 represent 10% or a value of 85 represents 85%.
	 * @param value - the value to subtract
	 * @returns a new decimal with the subtracted percentage value
	 */
	subtractPercent(value: number): Decimal

	/**
	 * Subtracts a percentage value from the current value of the decimal. For example a value of 10 represent 10% or a value of 85 represents 85%.
	 * @param value - the value to subtract
	 * @returns a new decimal with the subtracted percentage value
	 */
	subtractPercent(value: Decimal): Decimal

	/**
	 * Returns a string representation of this object.
	 * @returns a string representation of this object
	 */
	toString(): string

	/**
	 * The valueOf() method is called by the ECMAScript interpret to return the "natural" value of an object. The Decimal object returns its current value as number. With this behavior script snippets can be written like: var d = new Decimal(10.0); var x = 1.0 + d.add(2.0); where x will be at the end 13.0.
	 * @returns the value of this object
	 */
	valueOf(): Object
}
```
