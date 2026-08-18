## Overview

Simple class representing a file on a remote WebDAV location. Instances are returned by WebDAVClient.propfind(String) and expose read-only metadata about the remote file.

## Description

This class provides read-only attributes for files returned from a WebDAV server. It does not allow manipulation of the remote file. Use `WebDAVClient.propfind()` to obtain `WebDAVFileInfo` objects.

```ts
/**
 * Class dw.net.WebDAVFileInfo
 */
declare class WebDAVFileInfo {
    /** The content type of the file. */
    contentType: string;

    /** The creationDate of the file. */
    creationDate: Date;

    /** Identifies if the file is a directory. */
    directory: boolean;

    /** The name of the file. */
    name: string;

    /** The path of the file. */
    path: string;

    /** The size of the file. */
    size: number;

    /** Returns the content type of the file. */
    getContentType(): string;

    /** Returns the creationDate of the file. */
    getCreationDate(): Date;

    /** Returns the name of the file. */
    getName(): string;

    /** Returns the path of the file. */
    getPath(): string;

    /** Returns the size of the file. */
    getSize(): number;

    /** Identifies if the file is a directory. */
    isDirectory(): boolean;

    /** Returns the lastModified date of the file. */
    lastModified(): Date;
}
```
