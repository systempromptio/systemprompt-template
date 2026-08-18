# dw.value.EnumValue

## Overview
Represents a single value for an Enumeration type with base value (String or Integer) and display value.

## Description
Represents a single value for an Enumeration type. Enumeration types can be configured through Business Manager for custom attributes. Some system attributes (e.g., order status) are also Enumeration types. Each EnumValue has a base value (String or Integer) and a display value. If the value of an Enumeration type object attribute is null, accessing that attribute returns an EnumValue with a null base value (not null itself), meaning empty(object.attribute) is false but empty(object.attribute.value) is true.

```ts
declare class EnumValue  {
	/**
	 * The display value of the enumeration value. If no display value is configured, returns the string representation of the value.
	 * @readonly
	 */
	readonly displayValue: string

	/**
	 * The value of the enumeration value. Either an integer or a string.
	 * @readonly
	 */
	readonly value: Object

	/**
	 * Returns the display value of the enumeration value. If no display value is configured, returns the string representation of the value.
	 * @returns the display value
	 */
	getDisplayValue(): string

	/**
	 * Returns the value of the enumeration value. Either an integer or a string.
	 * @returns the value
	 */
	getValue(): Object

	/**
	 * Same as getDisplayValue().
	 * @returns the display value
	 */
	toString(): string

	/**
	 * Returns the "natural" primitive value of this object (equivalent to getValue()).
	 * @returns the primitive value
	 */
	valueOf(): Object
}
```
