# dw.io.XMLStreamReader

## Overview
Cursor-style XML stream reader for incremental parsing of XML documents.

## Description
Provides methods to navigate XML stream events, inspect names, namespaces, text, and to read element text or subtrees as objects.

```ts
declare class XMLStreamReader  {
    /** Returns the column number where the current event ends or -1. */
    getColumnNumber(): number

    /** Returns the encoding of the input or null if unknown. */
    getEncoding(): string | null

    /** Returns the integer code of the current event. */
    getEventType(): number

    /** Returns the line number where the current event ends or -1. */
    getLineNumber(): number

    /** Returns the local name for START_ELEMENT/END_ELEMENT or ENTITY_REFERENCE. */
    getLocalName(): string

    /** Returns the number of namespace declarations on the current element. */
    getNamespaceCount(): number

    /** Returns the prefix for the namespace at the given index. */
    getNamespacePrefix(index: number): string | null

    /** Returns namespace URI for the given prefix. */
    getNamespaceURI(prefix: string): string | null

    /** Returns namespace URI for the namespace declared at index. */
    getNamespaceURI(index: number): string

    /** Returns URI of the element's prefix or null. */
    getNamespaceURI(): string | null

    /** Returns data part of a processing instruction or null. */
    getPIData(): string | null

    /** Returns target of a processing instruction or null. */
    getPITarget(): string | null

    /** Returns prefix of the current event or null. */
    getPrefix(): string | null

    /** Returns text for text-bearing events or null. */
    getText(): string | null

    /** Returns length of current text event. */
    getTextLength(): number

    /** Returns offset where current text begins. */
    getTextStart(): number

    /** Returns XML version declared or null. */
    getVersion(): string | null

    /** Reads content of a text-only element (deprecated). */
    getElementText(): string

    /** Reads a subtree and returns it as an object (deprecated alias). */
    getXMLObject(): Object

    /** Checks if current event has a name (START/END element). */
    hasName(): boolean

    /** Returns true if there are more parsing events. */
    hasNext(): boolean

    /** Indicates if current event has text. */
    hasText(): boolean

    /** Identifies if specified attribute was created by default. */
    isAttributeSpecified(index: number): boolean

    /** True if current event is character data. */
    isCharacters(): boolean

    /** True if cursor points to an end tag. */
    isEndElement(): boolean

    /** True if standalone was set in declaration. */
    isStandalone(): boolean

    /** True if cursor points to a start tag. */
    isStartElement(): boolean

    /** True if cursor points to whitespace-only text. */
    isWhiteSpace(): boolean

    /** Advances to next parsing event and returns its integer code. */
    next(): number

    /** Skips to next START_ELEMENT/END_ELEMENT, returns its event type. */
    nextTag(): number

    /** Reads coalesced element text content. */
    readElementText(): string

    /** Reads a subtree and parses it as an XML object. */
    readXMLObject(): Object

    /** Verifies current event matches type/namespace/name, throws if not. */
    require(type: number, namespaceURI: string | null, localName: string | null): void

    /** Returns whether standalone was set. */
    standaloneSet(): boolean
}
```
