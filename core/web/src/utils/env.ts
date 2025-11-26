/**
 * Environment configuration utilities
 */

/**
 * Get the API base URL from environment variables
 * Falls back to current origin if not set
 */
export function getApiBaseUrl(): string {
  // In development, use the proxy (relative URLs)
  if (import.meta.env.DEV) {
    return '';
  }

  // In production, use VITE_API_BASE_URL or current origin
  return import.meta.env.VITE_API_BASE_URL || window.location.origin;
}

/**
 * Get full API endpoint URL
 */
export function getApiUrl(path: string): string {
  const baseUrl = getApiBaseUrl();
  const cleanPath = path.startsWith('/') ? path : `/${path}`;
  return `${baseUrl}${cleanPath}`;
}
