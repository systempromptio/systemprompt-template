# dw.web.HttpParameter

## Overview
Represents an HTTP parameter with type conversion and multi-value support.

## Description
Represents an HTTP parameter.

```
Object
  dw.web.HttpParameter
```

```ts
declare class HttpParameter  {
	/**
	 * The value of the current HttpParameter attribute as a boolean. If there is more than one value defined, only the first one is returned. For an undefined attribute it returns null.
	 */
	readonly booleanValue: boolean

	/**
	 * The value of the current HttpParameter attribute as a date. If there is more than one value defined, only the first one is returned. For an undefined attribute and if attribute is not a date it returns null.
	 */
	readonly dateValue: Date

	/**
	 * The value of the current HttpParameter attribute as a number. If there is more than one value defined, only the first one is returned. For an undefined attribute it returns 0.0.
	 */
	readonly doubleValue: number

	/**
	 * Identifies if there is a value for the http parameter attribute and whether the value is empty. A value is treated as empty if it's not blank.
	 */
	readonly empty: boolean

	/**
	 * The value of the current HttpParameter attribute as int. If there is more than one value defined, only the first one is returned. For an undefined attribute it returns null.
	 */
	readonly intValue: number

	/**
	 * The raw value for this HttpParameter instance. The raw value is the not trimmed String value of this HTTP parameter. If there is more than one value defined, only the first one is returned. For an undefined attribute the method returns null.
	 */
	readonly rawValue: string

	/**
	 * A Collection of all raw values for this HTTP parameter. The raw value is the not trimmed String value of this HTTP parameter.
	 */
	readonly rawValues: Collection

	/**
	 * The value of the current HttpParameter attribute. If there is more than one value defined, only the first one is returned. For an undefined attribute the method returns null.
	 */
	readonly stringValue: string

	/**
	 * A Collection of all defined values for this HTTP parameter.
	 */
	readonly stringValues: Collection

	/**
	 * Identifies if the parameter was submitted. This is equivalent to the check whether the parameter has a value.
	 */
	readonly submitted: boolean

	/**
	 * The value of the current HttpParameter attribute. If there is more than one value defined, only the first one is returned. For an undefined attribute the method returns null.
	 */
	readonly value: string

	/**
	 * A Collection of all defined values for this current HTTP parameter.
	 */
	readonly values: Collection

	/**
	 * Identifies if the given value is part of the actual values.
	 * @param value - The value to check
	 * @returns True if the value is among the actual values, false otherwise
	 */
	containsStringValue(value: string): boolean

	/**
	 * Returns the value of the current HttpParameter attribute as a boolean. If there is more than one value defined, only the first one is returned. For an undefined attribute it returns null.
	 * @returns The actual value as a boolean or null if no value is available
	 */
	getBooleanValue(): boolean

	/**
	 * Returns the value of the current HttpParameter attribute as a boolean. If there is more than one value defined, only the first one is returned. For an undefined attribute it returns the given default value.
	 * @param defaultValue - The default value to use
	 * @returns The value of the parameter or the default value if empty
	 */
	getBooleanValue(defaultValue: boolean): boolean

	/**
	 * Returns the value of the current HttpParameter attribute as a date. If there is more than one value defined, only the first one is returned. For an undefined attribute and if attribute is not a date it returns null.
	 * @returns The actual value as date or null if empty
	 */
	getDateValue(): Date

	/**
	 * Returns the value of the current HttpParameter attribute as a date. If there is more than one value defined, only the first one is returned. For an undefined attribute it returns the given default value and if the attribute is not a date it returns null.
	 * @param defaultValue - The default value to use
	 * @returns The date value of the attribute or the default value if empty
	 */
	getDateValue(defaultValue: Date): Date

	/**
	 * Returns the value of the current HttpParameter attribute as a number. If there is more than one value defined, only the first one is returned. For an undefined attribute it returns 0.0.
	 * @returns The actual value as double or null if the parameter has no value
	 */
	getDoubleValue(): number

	/**
	 * Returns the value of the current HttpParameter attribute as a number. If there is more than one value defined, only the first one is returned. For an undefined attribute it returns the given default value.
	 * @param defaultValue - The default value to use
	 * @returns The actual value as double or the default value if empty
	 */
	getDoubleValue(defaultValue: number): number

	/**
	 * Returns the value of the current HttpParameter attribute as int. If there is more than one value defined, only the first one is returned. For an undefined attribute it returns null.
	 * @returns The actual value as an integer or null if no value is available
	 */
	getIntValue(): number

	/**
	 * Returns the value of the current HttpParameter attribute as an integer. If there is more than one value defined, only the first one is returned. For an undefined attribute it returns the given default value.
	 * @param defaultValue - The default value to use
	 * @returns The value of the parameter or the default value if empty
	 */
	getIntValue(defaultValue: number): number

	/**
	 * Returns the raw value for this HttpParameter instance. The raw value is the not trimmed String value of this HTTP parameter. If there is more than one value defined, only the first one is returned. For an undefined attribute the method returns null.
	 * @returns The actual value or null
	 */
	getRawValue(): string

	/**
	 * Returns a Collection of all raw values for this HTTP parameter. The raw value is the not trimmed String value of this HTTP parameter.
	 * @returns Collection of raw values
	 */
	getRawValues(): Collection

	/**
	 * Returns the value of the current HttpParameter attribute. If there is more than one value defined, only the first one is returned. For an undefined attribute the method returns null.
	 * @returns The actual value or null
	 */
	getStringValue(): string

	/**
	 * Returns the value of the current HttpParameter attribute. If there is more than one value defined, only the first one is returned. For an undefined attribute it returns the given default value.
	 * @param defaultValue - The default value to use
	 * @returns The value of the parameter or the default value if empty
	 */
	getStringValue(defaultValue: string): string

	/**
	 * Returns a Collection of all defined values for this HTTP parameter.
	 * @returns Collection of values
	 */
	getStringValues(): Collection

	/**
	 * Returns the value of the current HttpParameter attribute. If there is more than one value defined, only the first one is returned. For an undefined attribute the method returns null.
	 * @returns The actual value or null
	 */
	getValue(): string

	/**
	 * Returns a Collection of all defined values for this current HTTP parameter.
	 * @returns Collection of values
	 */
	getValues(): Collection

	/**
	 * Identifies if the given String is an actual value of this http parameter.
	 * @param value - The value to check
	 * @returns True if the value is checked, false otherwise
	 */
	isChecked(value: string): boolean

	/**
	 * Identifies if there is a value for the http parameter attribute and whether the value is empty. A value is treated as empty if it's not blank.
	 * @returns True if empty, false otherwise
	 */
	isEmpty(): boolean

	/**
	 * Identifies if the parameter was submitted. This is equivalent to the check whether the parameter has a value.
	 * @returns True if submitted, false otherwise
	 */
	isSubmitted(): boolean

	/**
	 * Returns the value of the current HttpParameter attribute. If there is more than one value defined, only the first one is returned. For an undefined attribute the method returns null.
	 * @returns The actual value or null
	 */
	toString(): string
}
```
