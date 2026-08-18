# dw.net.SFTPClient

## Overview
Client for SFTP operations: connect, authenticate, list, get/put files, manage directories, and simple file info retrieval.

## Description
Provides SFTP actions such as connect/disconnect, file transfer (text/binary), directory listing, file info, and basic identity control for authentication.

```ts
declare class SFTPClient  {
    constructor()

    /** Adds a known host public key for the next connection attempt (non-persistent). */
    addKnownHostKey(type: string, key: string): void

    /** Change remote directory. */
    cd(path: string): boolean

    /** Connect and login with host, user and password. */
    connect(host: string, user: string, password: string): boolean

    /** Connect with host and port. */
    connect(host: string, port: number, user: string, password: string): boolean

    /** Delete remote file by path. */
    del(path: string): boolean

    /** Disconnect and logout. */
    disconnect(): void

    /** Read remote file content as string using ISO-8859-1 by default. */
    get(path: string): string

    /** Read remote file with specified encoding. */
    get(path: string, encoding: string): string

    /** Read remote file and write into local file using given encoding; returns true on success. */
    get(path: string, encoding: string, file: File): boolean

    /** Read remote file as binary into local file; returns true on success. */
    getBinary(path: string, file: File): boolean

    /** Returns whether the client is currently connected. */
    getConnected(): boolean

    /** Error message from last action. */
    getErrorMessage(): string

    /** Returns SFTPFileInfo for path or null. */
    getFileInfo(path: string): SFTPFileInfo

    /** Gets configured identity (private key) for next connection (KeyRef). */
    getIdentity(): KeyRef

    /** Gets configured timeout (ms). */
    getTimeout(): number

    /** Lists entries in current directory as SFTPFileInfo objects. */
    list(): SFTPFileInfo[]

    /** Lists entries in remote path. */
    list(path: string): SFTPFileInfo[]

    /** Create directory. */
    mkdir(path: string): boolean

    /** Put text content to remote path (ISO-8859-1). */
    put(path: string, content: string): boolean

    /** Put text with encoding. */
    put(path: string, content: string, encoding: string): boolean

    /** Put local file as binary to remote path. */
    putBinary(path: string, file: File): boolean

    /** Remove a remote directory (must be empty). */
    removeDirectory(path: string): boolean

    /** Rename a remote file. */
    rename(from: string, to: string): boolean

    /** Set identity (KeyRef) used for next connection attempt. */
    setIdentity(keyRef: KeyRef): void

    /** Set connection timeout (ms) for future connections. */
    setTimeout(timeoutMillis: number): void
}
```
