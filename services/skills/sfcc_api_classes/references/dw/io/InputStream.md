# dw.io.InputStream

## Overview
Represents a byte stream source usable by stream readers (e.g., XMLStreamReader). The class itself provides lifecycle control but not direct read methods.

## Description
`InputStream` models an input byte stream. It cannot be instantiated directly. Use implementations provided by the platform (or wrappers) and chain with higher-level readers to consume bytes. Provides a `close()` method to release the stream.

## All Known Subclasses
No known subclasses.

```ts
declare class InputStream  {
    /** Close the input stream and release resources. */
    close(): void
}
```
