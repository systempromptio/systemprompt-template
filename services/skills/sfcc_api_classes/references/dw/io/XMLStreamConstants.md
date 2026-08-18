# dw.io.XMLStreamConstants

## Overview
Useful numeric constants for working with XML stream events.

## Description
Provides integer codes used by XML stream parsers to identify event types (start element, characters, comment, etc.).

```ts
declare class XMLStreamConstants  {
    /** Represents the start of an element. */
    static START_ELEMENT: 1

    /** Represents the end of an element. */
    static END_ELEMENT: 2

    /** Represents a processing instruction. */
    static PROCESSING_INSTRUCTION: 3

    /** Represents character data. */
    static CHARACTERS: 4

    /** Represents a comment. */
    static COMMENT: 5

    /** Represents a space event. */
    static SPACE: 6

    /** Represents the start of the document. */
    static START_DOCUMENT: 7

    /** Represents the end of the document. */
    static END_DOCUMENT: 8

    /** Represents an entity reference. */
    static ENTITY_REFERENCE: 9

    /** Represents an attribute in an element. */
    static ATTRIBUTE: 10

    /** Represents a DTD section. */
    static DTD: 11

    /** Represents a CDATA section. */
    static CDATA: 12

    /** Represents a namespace declaration. */
    static NAMESPACE: 13

    /** Represents a notation declaration. */
    static NOTATION_DECLARATION: 14

    /** Represents an entity declaration. */
    static ENTITY_DECLARATION: 15

    /**
     * Default constructor.
     */
    XMLStreamConstants(): void
}
```
