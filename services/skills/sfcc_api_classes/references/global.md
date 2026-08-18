# global

## Overview
Pre-defined object serving as a placeholder for JavaScript's global properties and functions, providing access to all predefined objects and functions.

## Description
The global object is a pre-defined object that serves as a placeholder for the global properties and functions of JavaScript. All other predefined objects, functions, and properties are accessible through the global object.

```ts
/**
 * Representation for Infinity as an Integer
 */
Infinity: Number

/**
 * Representation for Not a Number as an Integer
 */
NaN: Number

/**
 * Represents an error during pipelet execution
 */
PIPELET_ERROR: Number

/**
 * Represents the next pipelet to fire
 */
PIPELET_NEXT: Number

/**
 * Provides access to the SlotContent object. Available only in ISML templates that are defined as the Slot's template.
 */
slotcontent: Object

/**
 * Representation for undefined
 */
undefined: Object

/**
 * Provides access to WSDL definition files in a Cartridge's webreferences2 folder.
 */
webreferences2: Object

/**
 * The current customer or null if this request is not associated with any customer.
 */
readonly customer: dw.customer.Customer

/**
 * References the module.exports property of the current module. Available only in scripts loaded as CommonJS module using require().
 */
readonly exports: Object

/**
 * Provides access to the global scope object containing all built-in functions and classes.
 */
readonly globalThis: Object

/**
 * An object representing the current module. Available only in scripts loaded as CommonJS module using require().
 */
readonly module: Module

/**
 * The current request.
 */
readonly request: dw.system.Request

/**
 * The current response.
 */
readonly response: dw.system.Response

/**
 * The current session.
 */
readonly session: dw.system.Session

/**
 * Unescapes characters in a URI component.
 * @param uri - a string that contains an encoded URI or other text to be decoded
 * @returns A copy of uri with any hexadecimal escape sequences replaced with the characters they represent
 */
decodeURI(uri: String): String

/**
 * Unescapes characters in a URI component.
 * @param uriComponent - a string that contains an encoded URI component or other text to be decoded
 * @returns A copy of uriComponent with any hexadecimal escape sequences replaced with the characters they represent
 */
decodeURIComponent(uriComponent: String): String

/**
 * Tests whether the given object is empty. null, undefined, zero-length strings, arrays or collections with no elements are considered empty.
 * @param obj - the object to be tested
 * @returns true if the object is interpreted as being empty
 */
empty(obj: Object): boolean

/**
 * Escapes characters in a URI.
 * @param uri - a String that contains the URI or other text to be encoded
 * @returns a copy of uri with certain characters replaced by hexadecimal escape sequences
 */
encodeURI(uri: String): String

/**
 * Escapes characters in a URI component.
 * @param uriComponent - a String that contains a portion of a URI or other text to be encoded
 * @returns a copy of uriComponent with certain characters replaced by hexadecimal escape sequences
 */
encodeURIComponent(uriComponent: String): String

/**
 * Encodes a String.
 * @param s - the String to be encoded
 * @returns a copy of s where characters have been replace by hexadecimal escape sequences
 */
escape(s: String): String

/**
 * Execute JavaScript code from a String.
 * @deprecated The eval() function is deprecated, because it's potential security risk for server side code injection.
 * @param code - a String that contains the JavaScript expression to be evaluated or the statements to be executed
 * @returns the value of the executed call or null
 */
eval(code: String): Object

/**
 * Import the specified class and make it available at the top level. Equivalent in effect to the Java import declaration.
 * @param classPath - the fully qualified class path
 */
importClass(classPath: Object): void

/**
 * Import all the classes in the specified package available at the top level. Equivalent in effect to the Java import declaration.
 * @param packagePath - the fully qualified package path
 */
importPackage(packagePath: Object): void

/**
 * Imports all functions from the specified script. Variables are not imported and must be accessed through helper functions. Syntax: [cartridgename:]scriptname
 * @param scriptPath - the path to the script
 */
importScript(scriptPath: String): void

/**
 * Returns true if the specified Number is finite.
 * @param number - the Number to test
 * @returns true if the specified Number is finite, false otherwise
 */
isFinite(number: Number): boolean

/**
 * Test the specified value to determine if it is not a Number.
 * @param object - the Object to be tested as a number
 * @returns True if the object is not a number
 */
isNaN(object: Object): boolean

/**
 * Determines whether the specified string is a valid name for an XML element or attribute.
 * @param name - the String specified
 * @returns True if the string is a valid name
 */
isXMLName(name: String): boolean

/**
 * Parses a String into an float Number.
 * @param s - the String to parse
 * @returns Returns the float as a Number
 */
parseFloat(s: String): Number

/**
 * Parses a String into an integer Number using the specified radix.
 * @param s - the String to parse
 * @param radix - the radix to use
 * @returns Returns the integer as a Number
 */
parseInt(s: String, radix: Number): Number

/**
 * Parses a String into an integer Number with automatic radix determination.
 * @param s - the String to parse
 * @returns Returns the integer as a Number
 */
parseInt(s: String): Number

/**
 * The require() function supports loading of modules in JavaScript. Works similar to CommonJS require(). Supports relative paths (./, ../), cartridge-relative (~/) and site cartridge paths (*/).
	* @param path - the path to the JavaScript module
	* @returns an object with the exported functions and properties
	*/
require(path: String): Module

/**
 * Formats and prints the message using the specified params. The format message is a Java MessageFormat expression. Printing happens in the script log output.
 * @param msg - the message to format
 * @param params - one, or multiple parameters that are used to format the message
 */
trace(msg: String, ...params: Object): void

/**
 * Decode an escaped String.
 * @param string - the String to decode
 * @returns a copy of the String where hexadecimal character sequences are replace by Unicode characters
 */
unescape(string: String): String

```
