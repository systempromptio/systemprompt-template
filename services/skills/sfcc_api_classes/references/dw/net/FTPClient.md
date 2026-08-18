# dw.net.FTPClient

## Overview
Deprecated FTP client for basic file operations on FTP servers (use SFTPClient for secure transfers).

## Description
Provides connect/disconnect, file listing, retrieval and upload operations, and directory management for FTP servers. Note: many methods are deprecated in favor of SFTP.

```ts
declare class FTPClient  {
    /** Deletes the remote file at the given path. */
    del(path: string): boolean

    /** Logs out and disconnects from the server. */
    disconnect(): void

    /** Reads remote file content as string using ISO-8859-1 (bounded by size). */
    get(path: string): string | null

    /** Reads remote file content using specified encoding. */
    get(path: string, encoding: string): string | null

    /** Reads remote file into local File using encoding. */
    get(path: string, encoding: string, file: File): boolean

    /** Reads remote file into local file in binary mode. */
    getBinary(path: string, file: File): boolean

    /** Returns whether client is currently connected. */
    getConnected(): boolean

    /** Returns numeric reply code from last action. */
    getReplyCode(): number

    /** Returns textual reply message from last action. */
    getReplyMessage(): string

    /** Returns timeout in milliseconds for this client. */
    getTimeout(): number

    /** Returns list of FTPFileInfo objects for current directory. */
    list(): FTPFileInfo[]

    /** Returns list of FTPFileInfo objects for provided remote path. */
    list(path: string): FTPFileInfo[]

    /** Creates a directory at the given path. */
    mkdir(path: string): boolean

    /** Puts given string content to the remote path using ISO-8859-1. */
    put(path: string, content: string): boolean

    /** Puts string content with specified encoding. */
    put(path: string, content: string, encoding: string): boolean

    /** Uploads a local file to the remote path in binary mode. */
    putBinary(path: string, file: File): boolean

    /** Removes a remote directory (must be empty). */
    removeDirectory(path: string): boolean

    /** Renames a remote file from -> to. */
    rename(from: string, to: string): boolean

    /** Sets timeout for next connections (milliseconds). */
    setTimeout(timeoutMillis: number): void
}
```
