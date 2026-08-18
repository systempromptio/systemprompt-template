# dw.rpc.Stub

## Overview
Base class for all service stubs accessed through WebReference objects, providing access to WSDL operations with configurable timeout and authentication.

## Description
Provides timeout and security configuration for SOAP web service operations. Supports connection timeout (5-15 seconds), read timeout (default 15 minutes for jobs, 2 minutes otherwise), and authentication credentials. Use Business Manager Services module for timeout values rather than class methods for better analytics and timeout management. Deprecated; use webreferences2 and Port instead.

```ts
declare class Stub  {
	/**
	 * Connection timeout property name (milliseconds, 100-15000 range)
	 */
	static CONNECTION_TIMEOUT: string
	
	/**
	 * Standard property: target service endpoint address
	 */
	static ENDPOINT_ADDRESS_PROPERTY: string
	
	/**
	 * Standard property: password for authentication
	 */
	static PASSWORD_PROPERTY: string
	
	/**
	 * Standard property: session maintenance flag
	 */
	static SESSION_MAINTAIN_PROPERTY: string
	
	/**
	 * Standard property: user name for authentication
	 */
	static USERNAME_PROPERTY: string
	
	/**
	 * The password
	 */
	password: string
	
	/**
	 * The current read timeout value in milliseconds for this Stub
	 */
	timeout: number
	
	/**
	 * The user name (handles sensitive security-related data per PCI DSS v3)
	 */
	username: string
	
	/**
	 * Gets the value of a specific configuration property
	 * @param name - property name
	 * @returns property value
	 */
	_getProperty(name: string): Object
	
	/**
	 * Sets a named property to the passed value
	 * @param name - property name
	 * @param value - property value
	 */
	_setProperty(name: string, value: Object): void
	
	/**
	 * Returns the password
	 * @returns the password
	 */
	getPassword(): string
	
	/**
	 * Returns the current read timeout value in milliseconds
	 * @returns timeout in milliseconds
	 */
	getTimeout(): number
	
	/**
	 * Returns the user name
	 * @returns the user name
	 */
	getUsername(): string
	
	/**
	 * Sets the password for this Stub instance
	 * @param password - the password to set
	 */
	setPassword(password: string): void
	
	/**
	 * Sets the read timeout value in milliseconds for this Stub instance
	 * @param timeout - timeout in milliseconds (100 minimum, 900000 maximum)
	 */
	setTimeout(timeout: number): void
	
	/**
	 * Sets the user name for this Stub instance
	 * @param username - the user name to set
	 */
	setUsername(username: string): void
}
```
