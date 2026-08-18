# dw.svc.FTPServiceDefinition

## Overview
Represents an FTP or SFTP Service Definition with two configuration styles.

## Description
Two basic configuration styles are supported. First: implement `createRequest` to call setOperation on the Service, performing a single operation returned in `parseResponse`. Second: implement `execute` to perform operations using the serviceClient (FTPClient or SFTPClient), with the return value passed to `parseResponse`.

```ts
declare class FTPServiceDefinition extends ServiceDefinition {
	/**
	 * Status of whether the underlying FTP connection will be disconnected after the service call.
	 */
	autoDisconnect: boolean


	/**
	 * Returns the status of whether the underlying FTP connection will be disconnected after the service call.
	 */
	isAutoDisconnect(): boolean

	/**
	 * Sets the auto-disconnect flag (default: true). If true, connection disconnects after service call; if false, remains open.
	 */
	setAutoDisconnect(b: boolean): FTPServiceDefinition
}
```

**Deprecated:** This class is only used with the deprecated ServiceRegistry. Use LocalServiceRegistry instead, which allows configuration on FTPService directly.

**API Versioned:** No longer available as of version 19.10.
