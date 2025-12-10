/**
 * Server-Sent Events (SSE) Event Processor Hook
 *
 * Handles parsing and processing of SSE event stream data using
 * the eventsource-parser library for proper cross-chunk handling.
 *
 * @module hooks/sse/useSSEEventHandler
 */

import { useCallback } from 'react'
import { EventSourceParserStream } from 'eventsource-parser/stream'

/**
 * Event handler callback for SSE events.
 * @param eventType - The type of event received
 * @param data - The event data payload
 */
export type SSEEventCallback = (eventType: string, data: string) => void

/**
 * Hook for processing SSE stream events.
 *
 * Uses eventsource-parser library to properly handle SSE events that
 * may be split across multiple network chunks.
 *
 * @returns Functions for processing SSE stream data
 */
export function useSSEEventHandler(onMessage?: SSEEventCallback) {
  const processSSEStream = useCallback(
    async (reader: ReadableStreamDefaultReader<Uint8Array>): Promise<void> => {
      const stream = new ReadableStream({
        async start(controller) {
          while (true) {
            const { done, value } = await reader.read()
            if (done) {
              controller.close()
              break
            }
            controller.enqueue(value)
          }
        }
      })

      const eventStream = stream
        .pipeThrough(new TextDecoderStream())
        .pipeThrough(new EventSourceParserStream())

      const eventReader = eventStream.getReader()

      while (true) {
        const { done, value } = await eventReader.read()
        if (done) break

        if (value.event && value.data) {
          console.log(`[SSE] Event received: ${value.event}`, { timestamp: new Date().toISOString(), dataPreview: value.data.substring(0, 200) })
          onMessage?.(value.event, value.data)
        }
      }
    },
    [onMessage]
  )

  return {
    processSSEStream,
  }
}
