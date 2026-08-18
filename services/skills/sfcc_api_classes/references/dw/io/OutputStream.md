# dw.io.OutputStream

## Overview
Represents a byte-oriented output stream used by the platform for writing data. It is typically chained with higher-level writers (for example `XMLStreamWriter`).

## Description
The class represents a stream of bytes that can be written from the application. It has no constructor and must be provided by the platform. Be careful when persisting sensitive data.

```ts
declare class OutputStream  {
    /** Closes the output stream. */
    close(): void
}
```
