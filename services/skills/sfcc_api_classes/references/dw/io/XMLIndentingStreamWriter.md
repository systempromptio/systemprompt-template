# dw.io.XMLIndentingStreamWriter

## Overview
XML writer that formats output with indentation and configurable newline characters for readability.

## Description
Extends `XMLStreamWriter` to emit human-readable, indented XML. Properties control the indent string and newline sequence.

```ts
declare class XMLIndentingStreamWriter extends XMLStreamWriter {
    /** The indent string used for nested elements. */
    indent: String

    /** The string used for new lines (default is standard newline). */
    newLine: String

    /** Constructs a new XMLIndentingStreamWriter wrapping the provided Writer. */
    XMLIndentingStreamWriter(writer: Writer): void

    /** Returns the current indent string. */
    getIndent(): String

    /** Returns the configured new-line string. */
    getNewLine(): String

    /** Sets the indent string used for formatting. */
    setIndent(indent: String): void

    /** Sets the new-line string used for formatting. */
    setNewLine(newLine: String): void
}
```
