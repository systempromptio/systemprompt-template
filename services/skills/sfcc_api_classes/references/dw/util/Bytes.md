# dw.util.Bytes

## Overview
Immutable class representing a binary data array, used for working with bytes in scripting contexts.

## Description
A simple immutable class representing an array of bytes, used for working with binary data in a scripting context. It acts as a view to ArrayBuffer. The buffer can be accessed through asUint8Array(). The size of the resulting byte representation is limited by the quota `api.jsArrayBufferSize` that defines the max size for an ArrayBuffer.

```ts
declare class Bytes  {
	/**
	 * The maximum number of bytes that a Bytes object can represent (10KB).
	 * @deprecated No longer used by the Bytes class.
	 */
	static MAX_BYTES: 10240

	/**
	 * The number of bytes represented by this object.
	 */
	readonly length: number

	/**
	 * Construct a Bytes object from the given ArrayBuffer or view.
	 * The bytes object acts as a view on the underlying ArrayBuffer. If a view is given that makes only a part of the storage array visible then this Bytes object will also make only the same part visible. The storage data is not copied.
	 * @param arrayBufferOrView - An ArrayBuffer or view to a buffer that is the storage.
	 */
	constructor(arrayBufferOrView: Object)

	/**
	 * Construct a Bytes object from the given string using the default encoding.
	 * Convenience for Bytes(string, "UTF-8").
	 * @param string - The string to encode into a Bytes object, must not be null.
	 * @throws IllegalArgumentException If the encoded byte sequence exceeds the maximum number of bytes.
	 */
	constructor(string: string)

	/**
	 * Construct a Bytes object from the given string using the given encoding.
	 * This method always replaces malformed input and unmappable character sequences with encoding defaults.
	 * @param string - The string to encode into a Bytes object, must not be null.
	 * @param encoding - The name of a supported encoding, or null in which case the default encoding (UTF-8) is used.
	 * @throws IllegalArgumentException If the named encoding is not supported or if the encoded byte sequence exceeds the maximum number of bytes.
	 */
	constructor(string: string, encoding: string)

	/**
	 * Returns a Uint8Array based on the ArrayBuffer used for this Bytes object.
	 * Changes to the returned ArrayBuffer will be visible in the Bytes object.
	 * @returns A newly created Uint8Array based on the existing ArrayBuffer.
	 */
	asUint8Array(): Object

	/**
	 * Returns the value of the byte at position index as an integer.
	 * The byte is interpreted as signed and so the value returned will always be between -128 and +127.
	 * @param index - The index of the byte.
	 * @returns The byte value at the specified index.
	 * @throws IndexOutOfBoundsException If the index argument is negative or not less than the length of this byte array.
	 */
	byteAt(index: number): number

	/**
	 * Return a new Bytes object containing the subsequence of this object's bytes specified by the index and length parameters.
	 * The returned object is a new view onto the same data, no data is copied.
	 * @param index - The initial index for the new view, inclusive.
	 * @param length - The number of bytes visible in the new view.
	 * @returns A new Bytes object representing a subsequence of this Bytes object.
	 * @throws ArrayIndexOutOfBoundsException If index < 0 or index > getLength() or index + length > getLength()
	 * @throws IllegalArgumentException If length < 0
	 */
	bytesAt(index: number, length: number): Bytes

	/**
	 * Returns the number of bytes represented by this object.
	 * @returns The number of bytes.
	 */
	getLength(): number

	/**
	 * Absolute get method for reading a signed integer value (32 bit) in network byte order (big endian).
	 * @param index - The byte index at which to read the number.
	 * @returns The read number.
	 * @throws IndexOutOfBoundsException If index is negative or not smaller than the number of bytes minus three.
	 */
	intAt(index: number): number

	/**
	 * Return a new Bytes object which has the same bytes as this one in reverse order.
	 * @returns A new Bytes object representing the reverse of this Bytes object.
	 */
	reverse(): Bytes

	/**
	 * Absolute get method for reading a signed short value (16 bit) in network byte order (big endian).
	 * @param index - The byte index at which to read the number.
	 * @returns The read number.
	 * @throws IndexOutOfBoundsException If index is negative or not smaller than the number of bytes minus one.
	 */
	shortAt(index: number): number

	/**
	 * Constructs a new String by decoding this array of bytes using the default encoding.
	 * Convenience for toString("UTF-8"). The method is protected by the quota `api.jsStringLength` that prevents creation of too long strings.
	 * @returns A String representing the decoded array of bytes.
	 */
	toString(): string

	/**
	 * Constructs a new String by decoding this array of bytes using the specified encoding.
	 * The method is protected by the quota `api.jsStringLength` that prevents creation of too long strings.
	 * @param encoding - The name of a supported encoding.
	 * @returns A String representing the decoded array of bytes.
	 * @throws IllegalArgumentException If the named encoding is not supported.
	 */
	toString(encoding: string): string
}
```
