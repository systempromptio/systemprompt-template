# dw.io.XMLStreamWriter

## Overview
Writer API for producing XML output with namespace and element handling.

## Description
Supports writing elements, attributes, namespaces, DTD, CDATA, comments and raw text to an underlying `Writer`.

```ts
declare class XMLStreamWriter  {
    /** Creates an XMLStreamWriter for the provided Writer. */
    XMLStreamWriter(writer: Writer): void

    /** Close this writer and free resources (does not close underlying writer). */
    close(): void

    /** Flushes cached data to the underlying output. */
    flush(): void

    /** Returns current default namespace. */
    getDefaultNamespace(): string | null

    /** Gets prefix bound to a URI or null. */
    getPrefix(uri: string): string | null

    /** Binds a URI to the default namespace. */
    setDefaultNamespace(uri: string | null): void

    /** Binds a prefix to a URI in current scope. */
    setPrefix(prefix: string, uri: string | null): void

    /** Writes attribute without prefix. */
    writeAttribute(localName: string, value: string): void

    /** Writes attribute with prefix and namespace. */
    writeAttribute(prefix: string, namespaceURI: string, localName: string, value: string): void

    /** Writes attribute with namespace. */
    writeAttribute(namespaceURI: string, localName: string, value: string): void

    /** Writes CDATA section. */
    writeCData(data: string): void

    /** Writes characters (text). */
    writeCharacters(text: string): void

    /** Writes a comment. */
    writeComment(data: string | null): void

    /** Writes the default namespace declaration. */
    writeDefaultNamespace(namespaceURI: string): void

    /** Writes DTD string. */
    writeDTD(dtd: string): void

    /** Writes an empty element tag. */
    writeEmptyElement(namespaceURI: string, localName: string): void

    /** Writes an empty element with prefix and namespace. */
    writeEmptyElement(prefix: string, localName: string, namespaceURI: string): void

    /** Writes an empty element with local name. */
    writeEmptyElement(localName: string): void

    /** Writes end document (closes open tags). */
    writeEndDocument(): void

    /** Writes an end tag relying on internal state. */
    writeEndElement(): void

    /** Writes an entity reference. */
    writeEntityRef(name: string): void

    /** Writes namespace declaration. */
    writeNamespace(prefix: string | null, namespaceURI: string): void

    /** Writes a processing instruction. */
    writeProcessingInstruction(target: string, data?: string): void

    /** Writes raw string directly (no XML checks). */
    writeRaw(raw: string): void

    /** Writes XML declaration. */
    writeStartDocument(encoding?: string, version?: string): void

    /** Writes a start element (localName). */
    writeStartElement(localName: string): void

    /** Writes a start element with namespaceURI and localName. */
    writeStartElement(namespaceURI: string, localName: string): void

    /** Writes a start element with prefix, localName and namespaceURI. */
    writeStartElement(prefix: string, localName: string, namespaceURI: string): void
}
```
