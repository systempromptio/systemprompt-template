# dw.util.SecureFilter

## Overview
Utility class for filtering untrusted data strings into RFC-compliant formats for various contexts by removing illegal characters.

## Description
SecureFilter contains many methods for manipulating untrusted data Strings into RFC-Compliant Strings for a given context by removing "bad" data from the untrusted data.

```ts
declare class SecureFilter  {
  /**
   * Filters illegal characters from a given input for use in a general HTML context (text content and text attributes). This method takes the UNION of allowed characters among all contexts.
   * @param input - untrusted input to be filtered, if necessary
   * @returns a properly filtered string for the given input
   */
  static forHtmlContent(input: string): string

  /**
   * Filters illegal characters from a given input for use in an HTML Attribute guarded by a double quote.
   * @param input - untrusted input to be filtered, if necessary
   * @returns a properly filtered string for the given input
   */
  static forHtmlInDoubleQuoteAttribute(input: string): string

  /**
   * Filters illegal characters from a given input for use in an HTML Attribute guarded by a single quote.
   * @param input - untrusted input to be filtered, if necessary
   * @returns a properly filtered string for the given input
   */
  static forHtmlInSingleQuoteAttribute(input: string): string

  /**
   * Filters illegal characters from a given input for use in an HTML Attribute left unguarded.
   * @param input - untrusted input to be filtered, if necessary
   * @returns a properly filtered string for the given input
   */
  static forHtmlUnquotedAttribute(input: string): string

  /**
   * Filters illegal characters from a given input for use in JavaScript inside an HTML attribute.
   * @param input - untrusted input to be filtered, if necessary
   * @returns a properly filtered string for the given input
   */
  static forJavaScriptInAttribute(input: string): string

  /**
   * Filters illegal characters from a given input for use in JavaScript inside an HTML block.
   * @param input - untrusted input to be filtered, if necessary
   * @returns a properly filtered string for the given input
   */
  static forJavaScriptInBlock(input: string): string

  /**
   * Filters illegal characters from a given input for use in JavaScript inside an HTML context.
   * @param input - untrusted input to be filtered, if necessary
   * @returns a properly filtered string for the given input
   */
  static forJavaScriptInHTML(input: string): string

  /**
   * Filters illegal characters from a given input for use in JavaScript inside a JavaScript source file.
   * @param input - untrusted input to be filtered, if necessary
   * @returns a properly filtered string for the given input
   */
  static forJavaScriptInSource(input: string): string

  /**
   * Filters illegal characters from a given input for use in a JSON Object Value to prevent escaping into a trusted context.
   * @param input - untrusted input to be filtered, if necessary
   * @returns a properly filtered string for the given input
   */
  static forJSONValue(input: string): string

  /**
   * Filters illegal characters from a given input for use as a component of a URI.
   * @param input - untrusted input to be filtered, if necessary
   * @returns a properly filtered string for the given input
   */
  static forUriComponent(input: string): string

  /**
   * Filters illegal characters from a given input for use as a component of a URI (strict version).
   * @param input - untrusted input to be filtered, if necessary
   * @returns a properly filtered string for the given input
   */
  static forUriComponentStrict(input: string): string

  /**
   * Filters illegal characters from a given input for use in an XML comments.
   * @param input - untrusted input to be filtered, if necessary
   * @returns a properly filtered string for the given input
   */
  static forXmlCommentContent(input: string): string

  /**
   * Filters illegal characters from a given input for use in a general XML context.
   * @param input - untrusted input to be filtered, if necessary
   * @returns a properly filtered string for the given input
   */
  static forXmlContent(input: string): string

  /**
   * Filters illegal characters from a given input for use in an XML attribute guarded by a double quote.
   * @param input - untrusted input to be filtered, if necessary
   * @returns a properly filtered string for the given input
   */
  static forXmlInDoubleQuoteAttribute(input: string): string

  /**
   * Filters illegal characters from a given input for use in an XML attribute guarded by a single quote.
   * @param input - untrusted input to be filtered, if necessary
   * @returns a properly filtered string for the given input
   */
  static forXmlInSingleQuoteAttribute(input: string): string
}
```
