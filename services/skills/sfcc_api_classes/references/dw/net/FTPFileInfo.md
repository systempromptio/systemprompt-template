# dw.net.FTPFileInfo

## Overview
Holds metadata about a remote FTP file or directory.

## Description
Lightweight container for remote file name, size, timestamp and whether it is a directory.

```ts
declare class FTPFileInfo  {
    /** Read-only flag whether the entry is a directory. */
    directory: boolean

    /** Read-only file name. */
    name: string

    /** Read-only size in bytes. */
    size: number

    /** Read-only timestamp (Date). */
    timestamp: Date

    /** Constructs FTPFileInfo with name, size, directory flag and timestamp. */
    FTPFileInfo(name: string, size: number, directory: boolean, timestamp: Date): void

    /** Returns whether entry is a directory. */
    getDirectory(): boolean

    /** Returns file name. */
    getName(): string

    /** Returns size of the file. */
    getSize(): number

    /** Returns timestamp of the file. */
    getTimestamp(): Date
}
```
