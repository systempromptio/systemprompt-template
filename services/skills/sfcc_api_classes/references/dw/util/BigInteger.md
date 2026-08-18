# dw.util.BigInteger

## Overview
Helper class representing arbitrary-length integer numbers. Deprecated as of version 22.7, replaced by BigInt.

## Description
Represents arbitrary-length integer numbers for web services requiring `xsd:integer`. Designed like a desktop calculator, allowing method chaining:
```javascript
var i = new BigInteger(10);
var result = i.add(2).subtract(3).get(); // returns 9
```
**Deprecated:** Replaced by BigInt. No longer available as of version 22.7.

```ts
declare class BigInteger  {
	/**
	 * Constructs a BigInteger with value 0.
	 */
	constructor()

	/**
	 * Constructs a BigInteger using the specified Number value.
	 * @param value - The value to use
	 */
	constructor(value: Number)

	/**
	 * Constructs a BigInteger using the specified string representation of a number.
	 * @param value - The value to use
	 */
	constructor(value: String)

	/**
	 * Returns the absolute value of this BigInteger.
	 */
	abs(): BigInteger

	/**
	 * Adds a Number value to this BigInteger.
	 * @param value - The value to add
	 */
	add(value: Number): BigInteger

	/**
	 * Adds a BigInteger value to this BigInteger.
	 * @param value - The value to add
	 */
	add(value: BigInteger): BigInteger

	/**
	 * Divides this BigInteger by the specified Number.
	 * @param value - The divisor
	 */
	divide(value: Number): BigInteger

	/**
	 * Divides this BigInteger by the specified BigInteger.
	 * @param value - The divisor
	 */
	divide(value: BigInteger): BigInteger

	/**
	 * Compares two BigInteger values for equivalence.
	 * @param other - The object to compare against
	 */
	equals(other: Object): boolean

	/**
	 * Returns the value of the BigInteger as a Number.
	 */
	get(): Number

	/**
	 * Calculates the hash code for this BigInteger.
	 */
	hashCode(): Number

	/**
	 * Multiplies the specified Number value with this BigInteger.
	 * @param value - The value to multiply with
	 */
	multiply(value: Number): BigInteger

	/**
	 * Multiplies the specified BigInteger value with this BigInteger.
	 * @param value - The value to multiply with
	 */
	multiply(value: BigInteger): BigInteger

	/**
	 * Returns the negated value of this BigInteger.
	 */
	negate(): BigInteger

	/**
	 * Subtracts the specified Number value from this BigInteger.
	 * @param value - The value to subtract
	 */
	subtract(value: Number): BigInteger

	/**
	 * Subtracts the specified BigInteger value from this BigInteger.
	 * @param value - The value to subtract
	 */
	subtract(value: BigInteger): BigInteger

	/**
	 * Returns a string representation of this object.
	 */
	toString(): String

	/**
	 * Returns the natural value of the object. Called by ECMAScript interpreter to enable expressions like `var x = 1 + bigIntInstance.add(2)`.
	 */
	valueOf(): Object
}
```
