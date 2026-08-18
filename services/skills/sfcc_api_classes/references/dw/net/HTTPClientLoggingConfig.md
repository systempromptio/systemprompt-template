# dw.net.HTTPClientLoggingConfig

## Overview
Configuration helper to control HTTP client logging, log level, and sensitive-field redaction for JSON, XML, headers, form body and text patterns.

## Description
Controls whether logging is enabled, the verbosity level, and which fields/patterns should be redacted to protect sensitive data during logging.

```ts
declare class HTTPClientLoggingConfig  {
    /** Whether logging is enabled. */
    enabled: boolean

    /** Current log level (DEBUG, INFO, WARN, ERROR). */
    level: string

    /** Sensitive body field names for form data. */
    sensitiveBodyFields: string[]

    /** Sensitive header names to redact. */
    sensitiveHeaders: string[]

    /** Sensitive JSON field names to redact. */
    sensitiveJsonFields: string[]

    /** Sensitive XML field names to redact. */
    sensitiveXmlFields: string[]

    constructor()

    /** Gets the current log level. */
    getLevel(): string

    /** Gets configured sensitive body fields. */
    getSensitiveBodyFields(): string[]

    /** Gets configured sensitive headers. */
    getSensitiveHeaders(): string[]

    /** Gets configured sensitive JSON fields. */
    getSensitiveJsonFields(): string[]

    /** Gets configured sensitive XML fields. */
    getSensitiveXmlFields(): string[]

    /** Returns whether logging is enabled. */
    isEnabled(): boolean

    /** Enable or disable logging. */
    setEnabled(enabled: boolean): void

    /** Set log level (DEBUG, INFO, WARN, ERROR). */
    setLevel(level: string): void

    /** Set sensitive body fields for form data redaction. */
    setSensitiveBodyFields(...fields: string[]): void

    /** Set sensitive headers for redaction. */
    setSensitiveHeaders(...headers: string[]): void

    /** Set sensitive JSON field names for redaction. */
    setSensitiveJsonFields(...fields: string[]): void

    /** Set sensitive text regex patterns for redaction. */
    setSensitiveTextPatterns(...patterns: string[]): void

    /** Set sensitive XML field names for redaction. */
    setSensitiveXmlFields(...fields: string[]): void
}
```
