/**
 * Server-Sent Events (SSE) Event Processor Hook
 *
 * Handles parsing and processing of SSE event stream data with proper
 * line-by-line parsing and event emission.
 *
 * @module hooks/sse/useSSEEventHandler
 */

import { useCallback } from 'react'

/**
 * Event handler callback for SSE events.
 * @param eventType - The type of event received
 * @param data - The event data payload
 */
export type SSEEventCallback = (eventType: string, data: string) => void

/**
 * Hook for processing SSE stream events.
 *
 * Provides utilities for parsing SSE protocol events from a stream
 * and invoking callbacks for each event. Handles proper SSE format:
 * ```
 * event: eventType
 * data: eventData
 * ```
 *
 * @returns Functions for processing SSE stream data
 *
 * @example
 * ```typescript
 * function SSEConsumer() {
 *   const { processSSELine, processSSEStream } = useSSEEventHandler(
 *     (eventType, data) => {
 *       console.log(`Received ${eventType}:`, data)
 *     }
 *   )
 *
 *   useEffect(() => {
 *     const reader = response.body.getReader()
 *     processSSEStream(reader)
 *   }, [processSSEStream])
 *
 *   return <div>Processing events...</div>
 * }
 * ```
 */
export function useSSEEventHandler(onMessage?: SSEEventCallback) {
  /**
   * Parse and process a single SSE event line pair.
   *
   * Validates that the line follows SSE format (event: type) and
   * that the next line contains the data payload. Emits event only
   * if both lines are present and valid.
   *
   * @param line - Current line (should start with "event: ")
   * @param nextLine - Following line (should start with "data: ")
   * @internal
   */
  const processSSELine = useCallback((line: string, nextLine: string | undefined): void => {
    if (!line.trim() || !line.startsWith('event: ')) return

    const eventType = line.substring(7).trim()
    if (nextLine?.startsWith('data: ')) {
      const data = nextLine.substring(6).trim()
      onMessage?.(eventType, data)
    }
  }, [onMessage])

  /**
   * Read and process complete SSE stream from a reader.
   *
   * Continuously reads from the provided stream until EOF, decoding
   * chunks and processing each event line pair. Handles stream
   * completion gracefully.
   *
   * @param reader - ReadableStreamDefaultReader from response.body
   * @throws {Error} If stream reading fails
   *
   * @example
   * ```typescript
   * const response = await fetch(url, { ...options })
   * await processSSEStream(response.body.getReader())
   * ```
   */
  const processSSEStream = useCallback(
    async (reader: ReadableStreamDefaultReader<Uint8Array>): Promise<void> => {
      const decoder = new TextDecoder()

      while (true) {
        const { done, value } = await reader.read()
        if (done) {
          break
        }

        const chunk = decoder.decode(value, { stream: true })
        const lines = chunk.split('\n')

        for (let i = 0; i < lines.length; i++) {
          processSSELine(lines[i], lines[i + 1])
        }
      }
    },
    [processSSELine]
  )

  return {
    processSSELine,
    processSSEStream,
  }
}
