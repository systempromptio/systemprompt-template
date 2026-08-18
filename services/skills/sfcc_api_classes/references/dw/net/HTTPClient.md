# dw.net.HTTPClient

## Overview
HTTP client for performing HTTP requests (GET, POST, PUT, PATCH, DELETE, OPTIONS, HEAD). Supports headers, timeouts, multipart uploads, file transfers, and basic response inspection.

## Description
Opens connections, sends requests, and reads responses. Handles text, binary and file payloads, multipart parts, redirects, logging configuration and client identity for mTLS.

```ts
declare class HTTPClient  {
    /** Gets the logging configuration for this HTTP client. */
    getLoggingConfig(): HTTPClientLoggingConfig

    /** Returns a specific response header from the last operation or null. */
    getResponseHeader(header: string): string

    /** Returns all values for a response header as a list. */
    getResponseHeaders(name: string): List

    /** Returns all response headers as a Map of name -> List(values). */
    getResponseHeaders(): Map

    /** Returns the status code of the last HTTP operation. */
    getStatusCode(): number

    /** Returns the status message of the last HTTP operation. */
    getStatusMessage(): string

    /** Returns the response body as text (for 2xx responses). */
    getText(): string

    /** Returns the response body as text using given encoding. */
    getText(encoding: string): string

    /** Returns the configured timeout (ms) for the client. */
    getTimeout(): number

    /** Opens the given URL using specified HTTP method. */
    open(method: string, url: string): void

    /** Deprecated open variant. */
    open(method: string, url: string, async: boolean, user: string, password: string): void

    /** Opens the URL using HTTP Basic auth credentials. */
    open(method: string, url: string, user: string, password: string): void

    /** Sends a prepared HTTP request. */
    send(): void

    /** Sends a text body as request. */
    send(text: string): void

    /** Sends text with encoding. */
    send(text: string, encoding: string): void

    /** Sends a file as request body. */
    send(file: File): void

    /** Sends request and writes response to file; returns true for positive status. */
    sendAndReceiveToFile(file: File): boolean

    /** Sends text and writes response to outFile; returns true for positive status. */
    sendAndReceiveToFile(text: string, outFile: File): boolean

    /** Sends text with encoding and writes response to outFile; returns true for positive status. */
    sendAndReceiveToFile(text: string, encoding: string, outFile: File): boolean

    /** Sends Bytes as request body. */
    sendBytes(body: Bytes): void

    /** Sends bytes and writes response to outFile; returns true for positive status. */
    sendBytesAndReceiveToFile(body: Bytes, outFile: File): boolean

    /** Sends a multipart request constructed from HTTPRequestPart objects. */
    sendMultiPart(...parts: HTTPRequestPart[]): boolean

    /** Enable or disable automatic redirect handling. */
    setAllowRedirect(allowRedirect: boolean): void

    /** Enable/disable certificate host name verification. */
    setHostNameVerification(enable: boolean): void

    /** Sets the client identity (private key) used for mTLS. */
    setIdentity(keyRef: KeyRef): void

    /** Sets the logging configuration for this client. */
    setLoggingConfig(config: HTTPClientLoggingConfig): void

    /** Sets a request header for the next HTTP operation. */
    setRequestHeader(key: string, value: string): void

    /** Sets the timeout (ms) for future connections. */
    setTimeout(timeoutMillis: number): void
}
```
