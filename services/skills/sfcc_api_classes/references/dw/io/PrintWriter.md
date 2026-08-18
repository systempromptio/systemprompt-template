# dw.io.PrintWriter

## Overview
Template output stream writer available in template scripting context as `out`. Cannot be instantiated by user scripts.

## Description
Used by templates to print text to the template output stream. Provided by the system as variable `out`. Take care with sensitive data.

```ts
declare class PrintWriter extends Writer {
    /** Prints the given string into the output stream. */
    print(str: string): void

    /** Prints the given string followed by a line break into the output stream. */
    println(str: string): void

    /** Prints a line break into the output stream. */
    println(): void
}
```
