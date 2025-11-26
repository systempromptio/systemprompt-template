/**
 * UNIVERSAL CONTEXT NOTIFICATION STREAM (Refactored)
 *
 * Receives real-time updates for ALL contexts (active + inactive).
 * Works alongside A2A streams - stores handle deduplication.
 *
 * Architecture:
 * - useSSEConnection: Connection lifecycle management
 * - useStreamEventProcessor: Event routing and deduplication
 * - streamEventHandlers: Individual event type handlers
 *
 * Event Sources:
 * - Direct MCP calls (no A2A stream)
 * - Background agent tasks (inactive contexts)
 * - Context lifecycle events (create/update/delete)
 *
 * Deduplication Strategy:
 * - Stores use Map<id, entity> - naturally idempotent
 * - Newer events overwrite older (timestamp-based)
 * - A2A streaming updates take precedence (in-flight only)
 *
 * This is NOT part of A2A spec - it's our custom coordination layer.
 * Event-driven (no polling) - 100ms backend poll interval.
 */

import { useCallback } from 'react'
import { useSSEConnection } from './useSSEConnection'
import { useStreamEventProcessor } from './useStreamEventProcessor'

/**
 * Main SSE stream hook - coordinates connection and event processing
 *
 * Brings together connection management and event processing into a single
 * unified interface. This is the hook to use in components.
 *
 * @example
 * ```typescript
 * const { connect, disconnect } = useContextStream()
 *
 * // Auto-connects on auth, auto-disconnects on unmount
 * ```
 */
export function useContextStream() {
  const url = `${window.location.origin}/api/v1/stream/contexts`
  const { processEvent } = useStreamEventProcessor()

  const handleMessage = useCallback((eventType: string, data: string) => {
    processEvent(eventType, data)
  }, [processEvent])

  const { connect, disconnect, isConnected, error } = useSSEConnection({
    url,
    onMessage: handleMessage,
  })

  return {
    connect,
    disconnect,
    isConnected,
    error,
    reconnect: () => {
      disconnect()
      connect()
    }
  }
}
