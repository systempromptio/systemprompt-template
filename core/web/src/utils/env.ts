/**
 * Environment configuration utilities
 */

/**
 * Get the API base URL from environment variables
 * Falls back to current origin if not set
 *
 * Uses VITE_API_BASE_HOST for consistency across the codebase
 */
export function getApiBaseUrl(): string {
  // In development, use the proxy (relative URLs)
  if (import.meta.env.DEV) {
    return '';
  }

  // In production, use VITE_API_BASE_HOST or current origin
  return import.meta.env.VITE_API_BASE_HOST || window.location.origin;
}

/**
 * Get full API endpoint URL
 */
export function getApiUrl(path: string): string {
  const baseUrl = getApiBaseUrl();
  const cleanPath = path.startsWith('/') ? path : `/${path}`;
  return `${baseUrl}${cleanPath}`;
}
