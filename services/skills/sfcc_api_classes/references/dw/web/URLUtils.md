# dw.web.URLUtils

## Overview
URL utility class for generating URLs used in Commerce Cloud Digital.

## Description
Provides methods for generating absolute and relative URLs. Site information is determined from the current HTTP request. Absolute URL methods create URLs with specified protocol or use protocol from request. Supports search-friendly URLs for products, categories, content pages, and folders.

Image transformation is performed via Dynamic Imaging Service (DIS) when available.

```ts
declare class URLUtils  {
	/**
	 * ID for a catalog context.
	 */
	static CONTEXT_CATALOG: 'ContextCatalog'

	/**
	 * ID for a library context.
	 */
	static CONTEXT_LIBRARY: 'ContextLibrary'

	/**
	 * ID for a site context (assigned cartridges).
	 */
	static CONTEXT_SITE: 'ContextSite'

	/**
	 * Return an absolute URL with protocol and host from the current request.
	 * @param action - URL action
	 * @param params - URL parameters
	 * @returns An absolute URL
	 */
	static abs(action: URLAction, ...params: URLParameter[]): URL

	/**
	 * Return an absolute URL with protocol and host from current request.
	 * @param action - The pipeline to invoke (e.g., 'Pipeline-StartNode')
	 * @param namesAndParams - Name=value pairs (e.g., 'pid', 'value1', 'cgid', 'value2')
	 * @returns An absolute URL
	 */
	static abs(action: string, ...namesAndParams: string[]): URL

	/**
	 * Returns absolute URL to static location of specified context with image transformation.
	 * @param context - Context identifier
	 * @param contextID - Context ID
	 * @param relPath - Relative path
	 * @param transform - Image transformation parameters
	 * @returns An absolute URL
	 */
	static absImage(context: string, contextID: string, relPath: string, transform: Object): URL

	/**
	 * Returns absolute static URL for resource in current site with image transformation.
	 * @param relPath - Relative path
	 * @param transform - Image transformation parameters
	 * @returns An absolute URL
	 */
	static absImage(relPath: string, transform: Object): URL

	/**
	 * Returns absolute URL to static location of specified context.
	 * @param context - Context identifier
	 * @param contextID - Context ID
	 * @param relPath - Relative path
	 * @returns An absolute URL
	 */
	static absStatic(context: string, contextID: string, relPath: string): URL

	/**
	 * Returns static URL for resource in current site.
	 * @param relPath - Relative path
	 * @returns An absolute URL
	 */
	static absStatic(relPath: string): URL

	/**
	 * Return a URL for use with an Interaction Continue Node to continue UI flow.
	 * @returns A continue URL
	 */
	static continueURL(): URL

	/**
	 * Generates hostname-only URL if alias is set, or URL to Home-Show pipeline using protocol of incoming request.
	 * @returns A home URL
	 */
	static home(): URL

	/**
	 * Return an absolute URL with HTTP protocol.
	 * @param action - URL action
	 * @param params - URL parameters
	 * @returns An absolute HTTP URL
	 */
	static http(action: URLAction, ...params: URLParameter[]): URL

	/**
	 * Return an absolute URL with HTTP protocol.
	 * @param action - The pipeline to invoke
	 * @param namesAndParams - Name=value pairs
	 * @returns An absolute HTTP URL
	 */
	static http(action: string, ...namesAndParams: string[]): URL

	/**
	 * Return a URL for use with an Interaction Continue Node using HTTP protocol.
	 * @returns An HTTP continue URL
	 */
	static httpContinue(): URL

	/**
	 * Generates hostname-only URL if alias is set, or URL to Home-Show pipeline using HTTP protocol.
	 * @returns An HTTP home URL
	 */
	static httpHome(): URL

	/**
	 * Returns absolute HTTP URL to static location with image transformation.
	 * @param context - Context identifier
	 * @param contextID - Context ID
	 * @param relPath - Relative path
	 * @param transform - Image transformation parameters
	 * @returns An HTTP URL
	 */
	static httpImage(context: string, contextID: string, relPath: string, transform: Object): URL

	/**
	 * Returns absolute HTTP URL to static location with image transformation.
	 * @param host - Host name
	 * @param context - Context identifier
	 * @param contextID - Context ID
	 * @param relPath - Relative path
	 * @param transform - Image transformation parameters
	 * @returns An HTTP URL
	 */
	static httpImage(host: string, context: string, contextID: string, relPath: string, transform: Object): URL

	/**
	 * Returns static HTTP URL for resource in current site with image transformation.
	 * @param relPath - Relative path
	 * @param transform - Image transformation parameters
	 * @returns An HTTP URL
	 */
	static httpImage(relPath: string, transform: Object): URL

	/**
	 * Returns static HTTP URL for resource in current site with image transformation.
	 * @param host - Host name
	 * @param relPath - Relative path
	 * @param transform - Image transformation parameters
	 * @returns An HTTP URL
	 */
	static httpImage(host: string, relPath: string, transform: Object): URL

	/**
	 * Returns absolute HTTP URL to static location of specified context.
	 * @param context - Context identifier
	 * @param contextID - Context ID
	 * @param relPath - Relative path
	 * @returns An HTTP URL
	 */
	static httpStatic(context: string, contextID: string, relPath: string): URL

	/**
	 * Returns absolute HTTP URL to static location of specified context.
	 * @param host - Host name
	 * @param context - Context identifier
	 * @param contextID - Context ID
	 * @param relPath - Relative path
	 * @returns An HTTP URL
	 */
	static httpStatic(host: string, context: string, contextID: string, relPath: string): URL

	/**
	 * Returns static HTTP URL for resource in current site.
	 * @param relPath - Relative path
	 * @returns An HTTP URL
	 */
	static httpStatic(relPath: string): URL

	/**
	 * Returns static HTTP URL for resource in current site.
	 * @param host - Host name
	 * @param relPath - Relative path
	 * @returns An HTTP URL
	 */
	static httpStatic(host: string, relPath: string): URL

	/**
	 * Return an absolute URL with HTTPS protocol.
	 * @param action - URL action
	 * @param params - URL parameters
	 * @returns An absolute HTTPS URL
	 */
	static https(action: URLAction, ...params: URLParameter[]): URL

	/**
	 * Return an absolute URL with HTTPS protocol.
	 * @param action - The pipeline to invoke
	 * @param namesAndParams - Name=value pairs
	 * @returns An absolute HTTPS URL
	 */
	static https(action: string, ...namesAndParams: string[]): URL

	/**
	 * Return a URL for use with an Interaction Continue Node using HTTPS protocol.
	 * @returns An HTTPS continue URL
	 */
	static httpsContinue(): URL

	/**
	 * Generates hostname-only URL if alias is set, or URL to Home-Show pipeline using HTTPS protocol.
	 * @returns An HTTPS home URL
	 */
	static httpsHome(): URL

	/**
	 * Returns absolute HTTPS URL to static location with image transformation.
	 * @param context - Context identifier
	 * @param contextID - Context ID
	 * @param relPath - Relative path
	 * @param transform - Image transformation parameters
	 * @returns An HTTPS URL
	 */
	static httpsImage(context: string, contextID: string, relPath: string, transform: Object): URL

	/**
	 * Returns absolute HTTPS URL to static location with image transformation.
	 * @param host - Host name
	 * @param context - Context identifier
	 * @param contextID - Context ID
	 * @param relPath - Relative path
	 * @param transform - Image transformation parameters
	 * @returns An HTTPS URL
	 */
	static httpsImage(host: string, context: string, contextID: string, relPath: string, transform: Object): URL

	/**
	 * Returns static HTTPS URL for resource in current site with image transformation.
	 * @param relPath - Relative path
	 * @param transform - Image transformation parameters
	 * @returns An HTTPS URL
	 */
	static httpsImage(relPath: string, transform: Object): URL

	/**
	 * Returns static HTTPS URL for resource in current site with image transformation.
	 * @param host - Host name
	 * @param relPath - Relative path
	 * @param transform - Image transformation parameters
	 * @returns An HTTPS URL
	 */
	static httpsImage(host: string, relPath: string, transform: Object): URL

	/**
	 * Returns absolute HTTPS URL to static location of specified context.
	 * @param context - Context identifier
	 * @param contextID - Context ID
	 * @param relPath - Relative path
	 * @returns An HTTPS URL
	 */
	static httpsStatic(context: string, contextID: string, relPath: string): URL

	/**
	 * Returns absolute HTTPS URL to static location of specified context.
	 * @param host - Host name
	 * @param context - Context identifier
	 * @param contextID - Context ID
	 * @param relPath - Relative path
	 * @returns An HTTPS URL
	 */
	static httpsStatic(host: string, context: string, contextID: string, relPath: string): URL

	/**
	 * Returns static HTTPS URL for resource in current site.
	 * @param relPath - Relative path
	 * @returns An HTTPS URL
	 */
	static httpsStatic(relPath: string): URL

	/**
	 * Returns static HTTPS URL for resource in current site.
	 * @param host - Host name
	 * @param relPath - Relative path
	 * @returns An HTTPS URL
	 */
	static httpsStatic(host: string, relPath: string): URL

	/**
	 * Returns relative URL to static location with image transformation.
	 * @param context - Context identifier
	 * @param contextID - Context ID
	 * @param relPath - Relative path
	 * @param transform - Image transformation parameters
	 * @returns A relative URL
	 */
	static imageURL(context: string, contextID: string, relPath: string, transform: Object): URL

	/**
	 * Returns static URL for resource in current site with image transformation.
	 * @param relPath - Relative path
	 * @param transform - Image transformation parameters
	 * @returns A relative URL
	 */
	static imageURL(relPath: string, transform: Object): URL

	/**
	 * Create URL that redirects to location in current site with another host name.
	 * @param host - Host name
	 * @param url - Target URL
	 * @returns A redirect URL
	 */
	static sessionRedirect(host: string, url: URL): URL

	/**
	 * Create URL that redirects to location in current site with another host name using HTTP only.
	 * @param host - Host name
	 * @param url - Target URL
	 * @returns A redirect URL
	 */
	static sessionRedirectHttpOnly(host: string, url: URL): URL

	/**
	 * Returns relative URL to static location of specified context.
	 * @param context - Context identifier
	 * @param contextID - Context ID
	 * @param relPath - Relative path
	 * @returns A relative URL
	 */
	static staticURL(context: string, contextID: string, relPath: string): URL

	/**
	 * Returns static URL for resource in current site.
	 * @param relPath - Relative path
	 * @returns A relative URL
	 */
	static staticURL(relPath: string): URL

	/**
	 * Return a relative URL.
	 * @param action - URL action
	 * @param params - URL parameters
	 * @returns A relative URL
	 */
	static url(action: URLAction, ...params: URLParameter[]): URL

	/**
	 * Return a relative URL.
	 * @param action - The pipeline to invoke
	 * @param namesAndParams - Name=value pairs
	 * @returns A relative URL
	 */
	static url(action: string, ...namesAndParams: string[]): URL
}
```
