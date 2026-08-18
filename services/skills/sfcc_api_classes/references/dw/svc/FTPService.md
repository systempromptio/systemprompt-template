# dw.svc.FTPService

## Overview
FTP-backed service abstraction used for file transfers between SFCC and remote servers.

## Description
Encapsulates FTP operations exposed by the service layer. Typically used via `LocalServiceRegistry`.

```ts
declare class FTPService {
    /**
     * Executes a file upload to the remote FTP endpoint.
     * @param localPath string
     * @param remotePath string
     */
    callUpload(localPath: string, remotePath: string): any

    /**
     * Executes a file download from the remote FTP endpoint.
     * @param remotePath string
     * @param localPath string
     */
    callDownload(remotePath: string, localPath: string): any
}
```
