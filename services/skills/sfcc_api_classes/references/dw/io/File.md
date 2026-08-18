# dw.io.File

## Overview
Represents files and directories in the virtual file namespace and provides common filesystem operations (copy, list, delete, zip, gzip, metadata).

## Description
Construct `File` instances by absolute path or by root directory + relative path. Offers utilities for file/directory creation, listing, copying, hashing (MD5), compressing/uncompressing, and querying metadata (exists, isFile, isDirectory, length, lastModified).

## All Known Subclasses
No known subclasses.

```ts
declare class File  {
    /** Create a File from an absolute path (normalizes separators). */
    constructor(absPath: string)

    /** Create a File given a root directory File and a relative path. */
    constructor(rootDir: File, relPath: string)

    /** Copy file to target; directories cannot be copied. Throws IOException or FileAlreadyExistsException. */
    copyTo(file: File): File

    /** Create an empty file; returns true if created, false if already exists. */
    createNewFile(): boolean

    /** Returns true if file exists. */
    exists(): boolean

    /** Returns normalized full path for this File. */
    getFullPath(): string

    /** Returns just the file or directory name. */
    getName(): string

    /** Deprecated: returns the path relative to the root dir; use `getFullPath()`. */
    getPath(): string

    /** Returns a File representing a root directory for the given root type and args. */
    static getRootDirectory(rootDir: string, ...args: string[]): File

    /** Returns root directory type string (for example, "IMPEX"). */
    getRootDirectoryType(): string

    /** Unzips this file into the provided root directory (assumes this is a zip). */
    unzip(root: File): void

    /** Returns MD5 hash of file contents. Throws on directories. */
    md5(): string

    /** Creates a single directory. */
    mkdir(): boolean

    /** Creates directories, including parents. */
    mkdirs(): boolean

    /** Removes this file or empty directory. */
    remove(): boolean

    /** Rename this file to the provided File. */
    renameTo(file: File): boolean

    /** GZip this file into `outputZipFile`. */
    gzip(outputZipFile: File): void

    /** Gunzip this file into the provided root directory. */
    gunzip(root: File): void

    /** Returns true if this File denotes a directory. */
    isDirectory(): boolean

    /** Returns true if this File denotes a normal file. */
    isFile(): boolean

    /** Last modified time in milliseconds. */
    lastModified(): number

    /** Length in bytes. */
    length(): number

    /** Returns array of names in directory or null if not a directory. */
    list(): string[]

    /** Returns a List of File objects in directory, or null if not a directory. */
    listFiles(): List

    /** Returns a filtered List of File objects satisfying the JS function `filter`. */
    listFiles(filter: Function): List

    /** Returns MD5 hash of this file's contents. */
    md5(): string

    /** Zip this file or directory into `outputZipFile`. */
    zip(outputZipFile: File): void
}
```
