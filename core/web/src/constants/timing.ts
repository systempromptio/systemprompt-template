/**
 * Timing Constants
 *
 * Centralized configuration for all timing values across the application.
 * These values are carefully tuned to balance UX and server load.
 */

export const TIMING = {
  // Debounce/Throttle Delays
  INPUT_DEBOUNCE: 300, // Standard input debounce for form fields
  SCROLL_DEBOUNCE: 100, // Quick visual updates during scroll
  SEARCH_DEBOUNCE: 300, // Search input debounce

  // Reconnection Strategy
  RECONNECT_INITIAL_DELAY: 2000, // Start with 2 seconds
  RECONNECT_MAX_ATTEMPTS: 5, // Balance between UX and server load
  RECONNECT_MAX_DELAY: 5000, // Cap exponential backoff at 5 seconds

  // Polling and Monitoring
  TOKEN_CHECK_INTERVAL: 5 * 60 * 1000, // Check token every 5 minutes
  TOKEN_REFRESH_THRESHOLD: 60 * 1000, // Refresh when 1 minute remaining
  AGENT_DISCOVERY_INTERVAL: 30 * 1000, // Refresh agents every 30 seconds
  CONTEXT_STATS_INTERVAL: 60 * 1000, // Update context stats every 1 minute

  // Artifact Loading
  ARTIFACT_FETCH_TIMEOUT: 30 * 1000, // 30 second timeout for artifact fetches
  DYNAMIC_OPTIONS_TIMEOUT: 30 * 1000, // 30 second timeout for dynamic options

  // Tool Execution
  TOOL_EXECUTION_TIMEOUT: 120 * 1000, // 2 minute timeout for tool execution

  // Stream and Connection
  SSE_RECONNECT_DELAY: 2000, // SSE reconnect delay
  SSE_MAX_RETRIES: 5, // Maximum SSE reconnection attempts
} as const

// Type-safe access to timing constants
export type TimingKey = keyof typeof TIMING
