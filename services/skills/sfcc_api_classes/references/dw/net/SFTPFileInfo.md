# dw.net.SFTPFileInfo

## Overview
Container with metadata about a remote SFTP file or directory: name, size, modification time and directory flag.

## Description
Used to return file/directory information from SFTP server listing operations. Contains read-only properties describing the remote entry.

```ts
declare class SFTPFileInfo  {
    /** True when the entry is a directory. */
    directory: boolean

    /** Last modification time as Date. */
    modificationTime: Date

    /** Name of the file or directory. */
    name: string

    /** Size of the file/directory. */
    size: number

    constructor(name: string, size: number, directory: boolean, mtime: number)

    /** Returns true if entry is a directory. */
    getDirectory(): boolean

    /** Returns last modification time. */
    getModificationTime(): Date

    /** Returns the entry name. */
    getName(): string

    /** Returns the entry size. */
    getSize(): number
}
```
