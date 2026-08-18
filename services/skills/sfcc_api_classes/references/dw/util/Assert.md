# dw.util.Assert

## Overview
Utility methods for assertion events.

## Description
The Assert class provides static utility methods for assertion events. All methods throw assertions when conditions are not met.

```ts
declare class Assert  {
	/**
	 * Propagates an assertion if the specified objects are not equal.
	 * @param arg1 - the first object to check
	 * @param arg2 - the second object to check
	 */
	static areEqual(arg1: Object, arg2: Object): void

	/**
	 * Propagates an assertion using the specified message if the specified objects are not equal.
	 * @param arg1 - the first object to check
	 * @param arg2 - the second object to check
	 * @param msg - the assertion message
	 */
	static areEqual(arg1: Object, arg2: Object, msg: string): void

	/**
	 * Propagates an assertion if the specified objects are not the same.
	 * @param arg1 - the first object to check
	 * @param arg2 - the second object to check
	 */
	static areSame(arg1: Object, arg2: Object): void

	/**
	 * Propagates an assertion using the specified message if the specified objects are not the same.
	 * @param arg1 - the first object to check
	 * @param arg2 - the second object to check
	 * @param msg - the assertion message
	 */
	static areSame(arg1: Object, arg2: Object, msg: string): void

	/**
	 * Propagates a failure assertion.
	 */
	static fail(): void

	/**
	 * Propagates a failure assertion using the specified message.
	 * @param msg - the assertion message
	 */
	static fail(msg: string): void

	/**
	 * Propagates an assertion if the specified check does not evaluate to an empty object.
	 * @param arg - the object to check
	 */
	static isEmpty(arg: Object): void

	/**
	 * Propagates an assertion using the specified message if the specified check does not evaluate to an empty object.
	 * @param arg - the object to check
	 * @param msg - the assertion message
	 */
	static isEmpty(arg: Object, msg: string): void

	/**
	 * Propagates an assertion if the specified check does not evaluate to false.
	 * @param check - the condition to check
	 */
	static isFalse(check: boolean): void

	/**
	 * Propagates an assertion using the specified message if the specified check does not evaluate to false.
	 * @param check - the condition to check
	 * @param msg - the assertion message
	 */
	static isFalse(check: boolean, msg: string): void

	/**
	 * Propagates an assertion if the specified object 'arg' is not an instance of the specified class 'clazz'.
	 * @param clazz - the class
	 * @param arg - the object to check
	 */
	static isInstanceOf(clazz: Object, arg: Object): void

	/**
	 * Propagates an assertion using the specified message if the specified object is not an instance of the specified class.
	 * @param clazz - the class
	 * @param arg - the object to check
	 * @param msg - the assertion message
	 */
	static isInstanceOf(clazz: Object, arg: Object, msg: string): void

	/**
	 * Propagates an assertion if the specified object is empty.
	 * @param arg - the object to check
	 */
	static isNotEmpty(arg: Object): void

	/**
	 * Propagates an assertion using the specified message if the specified object is empty.
	 * @param arg - the object to check
	 * @param msg - the assertion message
	 */
	static isNotEmpty(arg: Object, msg: string): void

	/**
	 * Propagates an assertion if the specified object is null.
	 * @param arg - the object to check
	 */
	static isNotNull(arg: Object): void

	/**
	 * Propagates an assertion using the specified message if the specified object is null.
	 * @param arg - the object to check
	 * @param msg - the assertion message
	 */
	static isNotNull(arg: Object, msg: string): void

	/**
	 * Propagates an assertion if the specified object is not null.
	 * @param arg - the object to check
	 */
	static isNull(arg: Object): void

	/**
	 * Propagates an assertion using the specified message if the specified object is not null.
	 * @param arg - the object to check
	 * @param msg - the assertion message
	 */
	static isNull(arg: Object, msg: string): void

	/**
	 * Propagates an assertion if the specified check does not evaluate to true.
	 * @param check - the condition to check
	 */
	static isTrue(check: boolean): void

	/**
	 * Propagates an assertion using the specified message if the specified check does not evaluate to true.
	 * @param check - the condition to check
	 * @param msg - the assertion message
	 */
	static isTrue(check: boolean, msg: string): void
}
```
