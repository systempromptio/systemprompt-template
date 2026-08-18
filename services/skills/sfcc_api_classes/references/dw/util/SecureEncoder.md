# dw.util.SecureEncoder

## Overview
Utility class for encoding untrusted data strings into RFC-compliant formats for various contexts (HTML, JavaScript, XML, URI, JSON).

## Description
SecureEncoder contains many methods for manipulating untrusted data Strings into RFC-Compliant Strings for a given context by encoding "bad" data into the proper format.

```ts
declare class SecureEncoder  {
  /**
   * Encodes a given input for use in a general HTML context (text content and text attributes). This method takes the UNION of allowed characters between the two context, so may be more imprecise that the more specific contexts.
   * @param input - untrusted input to be encoded, if necessary
   * @returns a properly encoded string for the given input
   */
  static forHtmlContent(input: string): string

  /**
   * Encodes a given input for use in an HTML Attribute guarded by a double quote.
   * @param input - untrusted input to be encoded, if necessary
   * @returns a properly encoded string for the given input
   */
  static forHtmlInDoubleQuoteAttribute(input: string): string

  /**
   * Encodes a given input for use in an HTML Attribute guarded by a single quote.
   * @param input - untrusted input to be encoded, if necessary
   * @returns a properly encoded string for the given input
   */
  static forHtmlInSingleQuoteAttribute(input: string): string

  /**
   * Encodes a given input for use in an HTML Attribute left unguarded.
   * @param input - untrusted input to be encoded, if necessary
   * @returns a properly encoded string for the given input
   */
  static forHtmlUnquotedAttribute(input: string): string

  /**
   * Encodes a given input for use in JavaScript inside an HTML attribute.
   * @param input - untrusted input to be encoded, if necessary
   * @returns a properly encoded string for the given input
   */
  static forJavaScriptInAttribute(input: string): string

  /**
   * Encodes a given input for use in JavaScript inside an HTML block.
   * @param input - untrusted input to be encoded, if necessary
   * @returns a properly encoded string for the given input
   */
  static forJavaScriptInBlock(input: string): string

  /**
   * Encodes a given input for use in JavaScript inside an HTML context. This method takes the UNION of allowed characters among the other contexts, so may be more imprecise that the more specific contexts.
   * @param input - untrusted input to be encoded, if necessary
   * @returns a properly encoded string for the given input
   */
  static forJavaScriptInHTML(input: string): string

  /**
   * Encodes a given input for use in JavaScript inside a JavaScript source file.
   * @param input - untrusted input to be encoded, if necessary
   * @returns a properly encoded string for the given input
   */
  static forJavaScriptInSource(input: string): string

  /**
   * Encodes a given input for use in a JSON Object Value to prevent escaping into a trusted context.
   * @param input - untrusted input to be encoded, if necessary
   * @returns a properly encoded string for the given input
   */
  static forJSONValue(input: string): string

  /**
   * Encodes a given input for use as a component of a URI. This is equivalent to javascript's encodeURIComponent and does a realistic job of encoding.
   * @param input - untrusted input to be encoded, if necessary
   * @returns a properly encoded string for the given input
   */
  static forUriComponent(input: string): string

  /**
   * Encodes a given input for use as a component of a URI. This is a strict encoder and fully complies with RFC3986.
   * @param input - untrusted input to be encoded, if necessary
   * @returns a properly encoded string for the given input
   */
  static forUriComponentStrict(input: string): string

  /**
   * Encodes a given input for use in an XML comments.
   * @param input - untrusted input to be encoded, if necessary
   * @returns a properly encoded string for the given input
   */
  static forXmlCommentContent(input: string): string

  /**
   * Encodes a given input for use in a general XML context (text content and text attributes). This method takes the UNION of allowed characters between the other contexts, so may be more imprecise that the more specific contexts.
   * @param input - untrusted input to be encoded, if necessary
   * @returns a properly encoded string for the given input
   */
  static forXmlContent(input: string): string

  /**
   * Encodes a given input for use in an XML attribute guarded by a double quote.
   * @param input - untrusted input to be encoded, if necessary
   * @returns a properly encoded string for the given input
   */
  static forXmlInDoubleQuoteAttribute(input: string): string

  /**
   * Encodes a given input for use in an XML attribute guarded by a single quote.
   * @param input - untrusted input to be encoded, if necessary
   * @returns a properly encoded string for the given input
   */
  static forXmlInSingleQuoteAttribute(input: string): string
}
```
