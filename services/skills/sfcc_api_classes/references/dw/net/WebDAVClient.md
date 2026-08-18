# dw.net.WebDAVClient

## Overview
WebDAV client for interacting with WebDAV servers: get/put files, directory listings, move, copy, propfind and other WebDAV operations.

## Description
Supports reading and writing text and binary content, creating collections, listing properties and file info, and basic response inspection and status checks.

```ts
declare class WebDAVClient  {
    /** Read content as string using default encoding. */
    get(path: string): string

    /** Read with specified encoding. */
    get(path: string, encoding: string): string

    /** Read and write to file, returning success. */
    get(path: string, encoding: string, file: File): boolean

    /** Read with file output and max size. */
    get(path: string, file: File, maxFileSize: number): boolean

    /** Read with encoding and maxGetSize. */
    get(path: string, encoding: string, maxGetSize: number): string

    /** Get a HashMap of all response headers. */
    getAllResponseHeaders(): HashMap

    /** Read binary into local file. */
    getBinary(path: string, file: File): boolean

    /** Read binary into local file with max size. */
    getBinary(path: string, file: File, maxFileSize: number): boolean

    /** Returns a response header value. */
    getResponseHeader(header: string): string

    /** Returns status code of last operation. */
    getStatusCode(): number

    /** Returns status text of last operation. */
    getStatusText(): string

    /** Create collection (directory) on remote server. */
    mkcol(path: string): boolean

    /** Move origin to destination; returns true on success. */
    move(origin: string, destination: string): boolean

    /** Move with overwrite flag. */
    move(origin: string, destination: string, overwrite: boolean): boolean

    /** Returns supported WebDAV methods for the given path. */
    options(path: string): string[]

    /** Returns WebDAVFileInfo listing (depth 1). */
    propfind(path: string): WebDAVFileInfo[]

    /** Returns WebDAVFileInfo listing with specified depth. */
    propfind(path: string, depth: number): WebDAVFileInfo[]

    /** Put text content to remote path. */
    put(path: string, content: string): boolean

    /** Put text with encoding. */
    put(path: string, content: string, encoding: string): boolean

    /** Put a local file to remote path (binary transfer). */
    put(path: string, file: File): boolean

    /** Returns true if last operation succeeded. */
    succeeded(): boolean
}
```
